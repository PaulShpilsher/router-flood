//! High-performance batch worker with optimizations
//!
//! This worker uses batch processing, buffer reuse, and batched stats updates
//! for improved performance under high load.

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use crate::stats::{Stats, BatchStats};
use crate::core::target::MultiPortTarget;
use crate::packet::{PacketBuilder, PacketType};
use crate::config::ProtocolMix;
use crate::error::Result;

/// Configuration for Worker
pub struct WorkerConfig {
    pub packet_rate: u64,
    pub packet_size_range: (usize, usize),
    pub protocol_mix: ProtocolMix,
    pub randomize_timing: bool,
    pub dry_run: bool,
    pub perfect_simulation: bool,
}

/// Worker with performance optimizations and batch processing
pub struct Worker {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    stats: Arc<Stats>,
    local_stats: BatchStats,
    target: Arc<MultiPortTarget>,
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
}

impl Worker {
    pub fn new(
        id: usize,
        stats: Arc<Stats>,
        target_ip: IpAddr,
        target: Arc<MultiPortTarget>,
        config: WorkerConfig,
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
        let buffer = vec![0u8; packet_size_range.1];
        
        Self {
            id,
            stats,
            local_stats,
            target,
            target_ip,
            packet_builder,
            buffer,
            packet_types,
            packet_type_index: 0,
            base_delay,
            randomize_timing,
            dry_run,
            perfect_simulation,
        }
    }
    
    pub async fn run(&mut self, running: Arc<AtomicBool>) {
        while running.load(Ordering::Relaxed) {
            // Process packet
            if let Err(_) = self.process_packet_batch().await {
                self.local_stats.increment_failed();
            }
            
            // Apply rate limiting
            self.apply_rate_limiting().await;
        }
        
        // Ensure final flush of batched stats
        self.local_stats.flush();
    }
    
    async fn process_packet_batch(&mut self) -> Result<()> {
        let port = self.target.next_port();
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
            // In real mode, mark as sent
            self.local_stats.increment_sent(size as u64, protocol);
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
        let icmp_count = (mix.icmp_ratio * 100.0) as usize;
        let ipv6_count = (mix.ipv6_ratio * 100.0) as usize;
        let arp_count = (mix.arp_ratio * 100.0) as usize;
        
        for _ in 0..udp_count {
            types.push(PacketType::Udp);
        }
        for _ in 0..tcp_syn_count {
            types.push(PacketType::TcpSyn);
        }
        for _ in 0..tcp_ack_count {
            types.push(PacketType::TcpAck);
        }
        for _ in 0..icmp_count {
            types.push(PacketType::Icmp);
        }
        for _ in 0..ipv6_count {
            types.push(PacketType::Ipv6Udp);
        }
        for _ in 0..arp_count {
            types.push(PacketType::Arp);
        }
        
        // Fill remainder with UDP if needed
        while types.len() < 100 {
            types.push(PacketType::Udp);
        }
        
        types
    }
    
    async fn apply_rate_limiting(&mut self) {
        let delay = if self.randomize_timing {
            let jitter = self.packet_builder.rng_gen_range(0.8..1.2);
            Duration::from_nanos((self.base_delay.as_nanos() as f64 * jitter) as u64)
        } else {
            self.base_delay
        };
        
        time::sleep(delay).await;
    }
}