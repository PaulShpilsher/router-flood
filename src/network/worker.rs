//! High-performance packet generation worker
//!
//! This worker uses buffer reuse, batched stats updates, and burst-mode sending
//! for improved performance under high load.

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

/// Number of packets to generate in each burst before sleeping
/// This significantly reduces tokio::time::sleep overhead for high packet rates
const BURST_SIZE: usize = 100;

/// Minimum sleep duration in microseconds
/// Below this threshold, we use yield_now() instead to avoid sleep overhead
const MIN_SLEEP_MICROS: u64 = 50;

use crate::stats::{Stats, BatchStats};
use crate::network::target::PortTarget;
use crate::packet::{PacketBuilder, PacketType};
use crate::config::ProtocolMix;
use crate::packet::PacketSizeRange;
use crate::error::Result;
use crate::transport::{WorkerChannels, ChannelType};

/// Configuration for Worker
pub struct WorkerConfig {
    pub packet_rate: u64,
    pub packet_size_range: PacketSizeRange,
    pub protocol_mix: ProtocolMix,
    pub randomize_timing: bool,
    pub dry_run: bool,
    pub perfect_simulation: bool,
}

/// Worker with performance optimizations
pub struct Worker {
    local_stats: BatchStats,
    target_port: Arc<PortTarget>,
    target_ip: IpAddr,
    packet_builder: PacketBuilder,
    // Pre-allocated buffer for zero-copy
    buffer: Vec<u8>,
    // Pre-calculated packet types for efficiency
    packet_types: Vec<PacketType>,
    packet_type_index: usize,
    base_delay: Duration,
    randomize_timing: bool,
    dry_run: bool,
    perfect_simulation: bool,
    // Transport channels for actual packet sending (None in dry-run mode)
    channels: Option<WorkerChannels>,
}

impl Worker {
    pub fn new(
        stats: Arc<Stats>,
        target_ip: IpAddr,
        target_port: Arc<PortTarget>,
        config: WorkerConfig,
        channels: Option<WorkerChannels>,
    ) -> Self {
        let packet_rate = config.packet_rate;
        let packet_size_range = config.packet_size_range;
        let protocol_mix = config.protocol_mix;
        let randomize_timing = config.randomize_timing;
        let dry_run = config.dry_run;
        let perfect_simulation = config.perfect_simulation;
        // Create local stats with batching (flush every 50 packets)
        let local_stats = BatchStats::new(stats.clone(), 50);
        let packet_builder = PacketBuilder::new(packet_size_range, protocol_mix.clone());
        let base_delay = Duration::from_nanos(1_000_000_000 / packet_rate.max(1));
        
        // Pre-calculate packet type distribution based on protocol mix
        let packet_types = Self::generate_packet_types(&protocol_mix);

        // Pre-allocate buffer for zero-copy operations
        // Size = max payload + largest possible headers (IPv6 + UDP = 48 bytes)
        const MAX_HEADER_SIZE: usize = 48;  // IPv6 (40) + UDP (8)
        let buffer = vec![0u8; packet_size_range.max + MAX_HEADER_SIZE];
        
        Self {
            local_stats,
            target_port,
            target_ip,
            packet_builder,
            buffer,
            packet_types,
            packet_type_index: 0,
            base_delay,
            randomize_timing,
            dry_run,
            perfect_simulation,
            channels,
        }
    }
    
    pub async fn run(&mut self, running: Arc<AtomicBool>) {
        while running.load(Ordering::Relaxed) {
            // Process packets in bursts to reduce sleep overhead
            for _ in 0..BURST_SIZE {
                if !running.load(Ordering::Relaxed) {
                    break;
                }

                // Process single packet
                if self.process_packet().await.is_err() {
                    self.local_stats.increment_failed();
                }
            }

            // Apply rate limiting once per burst instead of per packet
            self.apply_burst_rate_limiting().await;
        }

        // Ensure final flush of batched stats
        self.local_stats.flush();
    }
    
    async fn process_packet(&mut self) -> Result<()> {
        let port = self.target_port.next_port();
        let packet_type = self.next_packet_type();
        
        // Try zero-copy build first
        match self.packet_builder.build_packet_into_buffer(
            &mut self.buffer,
            packet_type,
            self.target_ip,
            port
        ) {
            Ok((size, protocol)) => {
                self.simulate_or_send(size, protocol);
            }
            Err(_) => {
                // Fallback to regular build
                match self.packet_builder.build_packet(packet_type, self.target_ip, port) {
                    Ok((packet_data, protocol)) => {
                        let size = packet_data.len();
                        self.simulate_or_send(size, protocol);
                    }
                    Err(_) => {
                        self.local_stats.increment_failed();
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn simulate_or_send(&mut self, size: usize, protocol: &str) {
        if self.dry_run {
            // Dry-run simulation mode
            let success = if self.perfect_simulation {
                true
            } else {
                self.packet_builder.rng_gen_bool(0.98)
            };

            if success {
                self.local_stats.increment_sent(size as u64, protocol);
            } else {
                self.local_stats.increment_failed();
            }
        } else {
            // Real packet sending mode
            if let Some(ref mut channels) = self.channels {
                // Determine channel type based on target IP
                let channel_type = match self.target_ip {
                    IpAddr::V4(_) => ChannelType::IPv4,
                    IpAddr::V6(_) => ChannelType::IPv6,
                };

                // Send the packet using the buffer (already contains packet data)
                match channels.send_packet(&self.buffer[..size], self.target_ip, channel_type) {
                    Ok(()) => {
                        self.local_stats.increment_sent(size as u64, protocol);
                    }
                    Err(_) => {
                        self.local_stats.increment_failed();
                    }
                }
            } else {
                // No channels available - this shouldn't happen in non-dry-run mode
                self.local_stats.increment_failed();
            }
        }
    }
    
    fn next_packet_type(&mut self) -> PacketType {
        let packet_type = self.packet_types[self.packet_type_index];
        self.packet_type_index = (self.packet_type_index + 1) % self.packet_types.len();
        packet_type
    }
    
    fn generate_packet_types(mix: &ProtocolMix) -> Vec<PacketType> {
        let mut types = Vec::with_capacity(100);
        
        // Generate 100 packet types based on ratios
        let udp_count = (mix.udp_ratio * 100.0) as usize;
        let tcp_syn_count = (mix.tcp_syn_ratio * 100.0) as usize;
        let tcp_ack_count = (mix.tcp_ack_ratio * 100.0) as usize;
        let tcp_fin_count = (mix.tcp_fin_ratio * 100.0) as usize;
        let tcp_rst_count = (mix.tcp_rst_ratio * 100.0) as usize;
        let icmp_count = (mix.icmp_ratio * 100.0) as usize;
        // Removed ipv6 and arp for simplification
        
        for _ in 0..udp_count {
            types.push(PacketType::Udp);
        }
        for _ in 0..tcp_syn_count {
            types.push(PacketType::TcpSyn);
        }
        for _ in 0..tcp_ack_count {
            types.push(PacketType::TcpAck);
        }
        for _ in 0..tcp_fin_count {
            types.push(PacketType::TcpFin);
        }
        for _ in 0..tcp_rst_count {
            types.push(PacketType::TcpRst);
        }
        for _ in 0..icmp_count {
            types.push(PacketType::Icmp);
        }
        // Use UDP for custom ratio since we simplified packet types
        let custom_count = (mix.custom_ratio * 100.0) as usize;
        for _ in 0..custom_count {
            types.push(PacketType::Udp);
        }
        
        // Fill remainder with UDP if needed
        while types.len() < 100 {
            types.push(PacketType::Udp);
        }
        
        types
    }
    
    /// Apply rate limiting for an entire burst of packets
    /// This replaces per-packet sleep with per-burst sleep for much better performance
    async fn apply_burst_rate_limiting(&mut self) {
        // Calculate delay for entire burst
        let burst_delay = self.base_delay.saturating_mul(BURST_SIZE as u32);

        // Apply jitter if randomization is enabled
        let delay = if self.randomize_timing {
            let jitter = self.packet_builder.rng_gen_range(0.8..1.2);
            Duration::from_nanos((burst_delay.as_nanos() as f64 * jitter) as u64)
        } else {
            burst_delay
        };

        // For very high rates, use yield instead of sleep to avoid overhead
        if delay.as_micros() < MIN_SLEEP_MICROS as u128 {
            tokio::task::yield_now().await;
        } else {
            time::sleep(delay).await;
        }
    }
}