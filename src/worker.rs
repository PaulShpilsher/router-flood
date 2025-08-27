//! Worker thread management and packet sending logic
//!
//! This module handles the spawning and management of worker threads
//! that generate and send packets according to the configured parameters.

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{debug, trace};

use crate::buffer_pool::WorkerBufferPool;
use crate::config::{Config, ProtocolMix};
use crate::constants::{stats, timing, NANOSECONDS_PER_SECOND};
use crate::error::{NetworkError, Result};
use crate::packet::{PacketBuilder, PacketType};
use crate::stats::FloodStats;
use crate::stats_original::LocalStats;
use crate::target::MultiPortTarget;
use crate::transport::{WorkerChannels, ChannelFactory};
use crate::transport_original::ChannelType;
use crate::adapters::ChannelTypeAdapter;

/// Manages the lifecycle of worker threads
pub struct WorkerManager {
    handles: Vec<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl WorkerManager {
    /// Create a new worker manager and spawn worker threads
    pub fn new(
        config: &Config,
        stats: Arc<FloodStats>,
        multi_port_target: Arc<MultiPortTarget>,
        target_ip: IpAddr,
        interface: Option<&pnet::datalink::NetworkInterface>,
        dry_run: bool,
    ) -> Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let handles = Self::spawn_workers(
            config,
            stats,
            running.clone(),
            multi_port_target,
            target_ip,
            interface,
            dry_run,
        )?;

        Ok(Self { handles, running })
    }

    /// Spawn worker threads based on configuration
    fn spawn_workers(
        config: &Config,
        stats: Arc<FloodStats>,
        running: Arc<AtomicBool>,
        multi_port_target: Arc<MultiPortTarget>,
        target_ip: IpAddr,
        interface: Option<&pnet::datalink::NetworkInterface>,
        dry_run: bool,
    ) -> Result<Vec<JoinHandle<()>>> {
        let mut handles = Vec::with_capacity(config.attack.threads);
        
        // Create per-worker channels to eliminate contention
        let worker_channels = ChannelFactory::create_worker_channels(
            config.attack.threads,
            interface,
            dry_run,
        )?;

        for (task_id, channels) in worker_channels.into_iter().enumerate() {
            let worker = Worker::new(
                task_id,
                stats.clone(),
                running.clone(),
                multi_port_target.clone(),
                target_ip,
                channels,
                config.attack.packet_rate,
                config.attack.packet_size_range,
                config.target.protocol_mix.clone(),
                config.attack.randomize_timing,
                dry_run,
            );

            let handle = tokio::spawn(async move {
                worker.run().await;
            });

            handles.push(handle);
        }

        Ok(handles)
    }

    /// Stop all worker threads gracefully
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Wait for all worker threads to complete
    pub async fn join_all(self) -> Result<()> {
        for handle in self.handles {
            handle.await.map_err(|e| NetworkError::PacketSend(format!("Worker join error: {}", e)))?;
        }
        Ok(())
    }

    /// Check if workers are still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

/// Individual worker thread that sends packets
struct Worker {
    task_id: usize,
    local_stats: LocalStats,
    running: Arc<AtomicBool>,
    multi_port_target: Arc<MultiPortTarget>,
    target_ip: IpAddr,
    channels: WorkerChannels,
    packet_builder: PacketBuilder,
    buffer_pool: WorkerBufferPool,
    base_delay: StdDuration,
    randomize_timing: bool,
    dry_run: bool,
}

impl Worker {
    #[allow(clippy::too_many_arguments)]
    fn new(
        task_id: usize,
        stats: Arc<FloodStats>,
        running: Arc<AtomicBool>,
        multi_port_target: Arc<MultiPortTarget>,
        target_ip: IpAddr,
        channels: WorkerChannels,
        packet_rate: u64,
        packet_size_range: (usize, usize),
        protocol_mix: ProtocolMix,
        randomize_timing: bool,
        dry_run: bool,
    ) -> Self {
        let packet_builder = PacketBuilder::new(packet_size_range, protocol_mix);
        let base_delay = StdDuration::from_nanos(NANOSECONDS_PER_SECOND / packet_rate);
        
        // Create local stats with batch size based on packet rate
        let stats_batch_size = (packet_rate / 20).max(10) as usize; // Batch every ~50ms, min 10 packets
        let local_stats = LocalStats::new(stats.clone(), stats_batch_size);
        
        // Create buffer pool for this worker (1400 bytes max packet size)
        let buffer_pool = WorkerBufferPool::new(1400, 5, 10); // 5 initial, max 10 buffers

        Self {
            task_id,
            local_stats,
            running,
            multi_port_target,
            target_ip,
            channels,
            packet_builder,
            buffer_pool,
            base_delay,
            randomize_timing,
            dry_run,
        }
    }

    /// Main worker loop
    async fn run(mut self) {
        while self.running.load(Ordering::Relaxed) {
            if let Err(e) = self.process_single_packet().await {
                if self.task_id == 0 {
                    debug!("Packet processing error: {}", e);
                }
                self.local_stats.increment_failed();
            }

            self.apply_rate_limiting().await;
        }
        
        // Ensure final flush when worker terminates
        self.local_stats.flush();
    }

    /// Process a single packet (build and send)
    async fn process_single_packet(&mut self) -> Result<()> {
        let current_port = self.multi_port_target.next_port();
        let packet_type = self.packet_builder.next_packet_type_for_ip(self.target_ip);

        // Use zero-copy packet building with buffer pool
        let mut buffer = self.buffer_pool.get_buffer();
        let buffer_slice = buffer.as_mut_slice();
        
        match self.packet_builder.build_packet_into_buffer(buffer_slice, packet_type, self.target_ip, current_port) {
            Ok((packet_size, protocol_name)) => {
                // Use only the portion of buffer that contains the packet
                let packet_data = &buffer_slice[..packet_size];
                
                if self.dry_run {
                    self.simulate_packet_send(packet_data, protocol_name).await;
                } else {
                    self.send_packet(packet_type, packet_data, protocol_name).await?;
                }
                
                // Return buffer to pool for reuse
                self.buffer_pool.return_buffer(buffer);
            },
            Err(_e) => {
                // Return buffer and fall back to normal allocation
                self.buffer_pool.return_buffer(buffer);
                self.fallback_packet_build_and_send(current_port, packet_type).await?;
            }
        }

        Ok(())
    }

    /// Fallback to normal packet building when buffer pool is unavailable
    async fn fallback_packet_build_and_send(&mut self, current_port: u16, packet_type: PacketType) -> Result<()> {
        let (packet_data, protocol_name) = self.packet_builder
            .build_packet(packet_type, self.target_ip, current_port)
            .map_err(|e| NetworkError::PacketSend(format!("Packet build failed: {}", e)))?;

        if self.dry_run {
            self.simulate_packet_send(&packet_data, protocol_name).await;
        } else {
            self.send_packet(packet_type, &packet_data, protocol_name).await?;
        }
        
        Ok(())
    }

    /// Simulate packet sending in dry-run mode
    async fn simulate_packet_send(&mut self, packet_data: &[u8], protocol_name: &str) {
        let simulate_success = self.packet_builder.rng_gen_bool(stats::SUCCESS_RATE_SIMULATION);
        
        if simulate_success {
            self.local_stats.increment_sent(packet_data.len() as u64, protocol_name);
            // Note: For logging, we'd need access to global stats, but this is for performance optimization
        } else {
            self.local_stats.increment_failed();
        }
    }

    /// Send packet via appropriate transport channel
    async fn send_packet(
        &mut self,
        packet_type: PacketType,
        packet_data: &[u8],
        protocol_name: &str,
    ) -> Result<()> {
        let new_channel_type = match packet_type {
            PacketType::Udp | PacketType::TcpSyn | PacketType::TcpAck | PacketType::Icmp => {
                crate::transport::layer::ChannelType::IPv4
            }
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp => {
                crate::transport::layer::ChannelType::IPv6
            }
            PacketType::Arp => {
                crate::transport::layer::ChannelType::Layer2
            }
        };
        let channel_type = ChannelTypeAdapter::to_original(new_channel_type);

        match self.channels.send_packet(packet_data, self.target_ip, channel_type) {
            Ok(_) => {
                self.local_stats.increment_sent(packet_data.len() as u64, protocol_name);
                Ok(())
            }
            Err(e) => {
                if self.task_id == 0 {
                    trace!("Failed to send packet: {}", e);
                }
                self.local_stats.increment_failed();
                Err(e)
            }
        }
    }


    /// Apply rate limiting using high-resolution token bucket
    async fn apply_rate_limiting(&mut self) {
        let target_nanos = if self.randomize_timing {
            let jitter = self.packet_builder.rng_gen_range(timing::JITTER_MIN..timing::JITTER_MAX);
            (self.base_delay.as_nanos() as f64 * jitter) as u64
        } else {
            self.base_delay.as_nanos() as u64
        };
        
        // Always use tokio::time::sleep for better CPU efficiency
        time::sleep(StdDuration::from_nanos(target_nanos)).await;
    }
}
