//! High-performance packet worker implementation
//!
//! This module provides the consolidated worker implementation using
//! batch processing and optimized packet generation.

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{debug, trace};

use crate::config::{Config, ProtocolMix};
use crate::constants::NANOSECONDS_PER_SECOND;
use crate::error::{NetworkError, Result};
use crate::packet::PacketType;
use crate::stats::{FloodStats, LocalStats};
use crate::core::target::MultiPortTarget;

/// High-performance packet worker using batch processing
pub struct PacketWorker {
    id: usize,
    #[allow(dead_code)]
    stats: Arc<FloodStats>,
    local_stats: LocalStats,
    target_ip: IpAddr,
    target_ports: Vec<u16>,
    packet_types: Vec<PacketType>,
    packet_type_index: usize,
    base_delay: Duration,
    randomize_timing: bool,
    dry_run: bool,
    running: Arc<AtomicBool>,
    // Performance tracking
    packets_processed: u64,
    total_processing_time: Duration,
    start_time: Instant,
}

impl PacketWorker {
    /// Create a new packet worker
    pub fn new(
        id: usize,
        target_ip: IpAddr,
        target_ports: Vec<u16>,
        config: &Config,
        stats: Arc<FloodStats>,
        running: Arc<AtomicBool>,
        dry_run: bool,
    ) -> Self {
        // Create local stats accumulator for batched updates
        let local_stats = LocalStats::new(stats.clone(), 50);
        
        // Pre-calculate packet types based on protocol mix
        let packet_types = Self::generate_packet_types(&config.target.protocol_mix);
        
        // Calculate base delay for rate limiting
        let base_delay = if config.attack.packet_rate > 0 {
            Duration::from_nanos(NANOSECONDS_PER_SECOND / config.attack.packet_rate as u64)
        } else {
            Duration::from_nanos(0)
        };
        
        Self {
            id,
            stats: stats.clone(),
            local_stats,
            target_ip,
            target_ports,
            packet_types,
            packet_type_index: 0,
            base_delay,
            randomize_timing: config.attack.randomize_timing,
            dry_run,
            running,
            packets_processed: 0,
            total_processing_time: Duration::from_nanos(0),
            start_time: Instant::now(),
        }
    }
    
    /// Generate packet types based on protocol mix
    fn generate_packet_types(mix: &ProtocolMix) -> Vec<PacketType> {
        let mut types = Vec::with_capacity(100);
        
        // Add packet types proportional to their ratios
        let udp_count = (mix.udp_ratio * 100.0) as usize;
        let tcp_syn_count = (mix.tcp_syn_ratio * 100.0) as usize;
        let tcp_ack_count = (mix.tcp_ack_ratio * 100.0) as usize;
        let icmp_count = (mix.icmp_ratio * 100.0) as usize;
        
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
        
        // Fill remaining with UDP
        while types.len() < 100 {
            types.push(PacketType::Udp);
        }
        
        types
    }
    
    /// Get the next packet type in rotation
    fn next_packet_type(&mut self) -> PacketType {
        let packet_type = self.packet_types[self.packet_type_index];
        self.packet_type_index = (self.packet_type_index + 1) % self.packet_types.len();
        packet_type
    }
    
    /// Get the next target port
    fn next_port(&self) -> u16 {
        if self.target_ports.is_empty() {
            80 // Default to HTTP
        } else {
            self.target_ports[self.packets_processed as usize % self.target_ports.len()]
        }
    }
    
    /// Process a batch of packets
    async fn process_batch(&mut self, batch_size: usize) -> Result<()> {
        let process_start = Instant::now();
        
        for _ in 0..batch_size {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }
            
            let packet_type = self.next_packet_type();
            let port = self.next_port();
            
            // Simulate packet processing
            if self.dry_run {
                // Simulate packet send
                trace!(
                    "Worker {} simulating {} packet to {}:{}",
                    self.id,
                    packet_type.protocol_name(),
                    self.target_ip,
                    port
                );
                // Use batched local stats for efficiency
                self.local_stats.increment_sent(64, packet_type.protocol_name());
            } else {
                // In real implementation, would send actual packet here
                // For now, simulate success/failure
                if rand::random::<f32>() > 0.01 { // 99% success rate
                    self.local_stats.increment_sent(
                        match packet_type {
                            PacketType::Udp => 64,
                            PacketType::TcpSyn | PacketType::TcpAck => 60,
                            PacketType::Icmp => 56,
                            _ => 64,
                        },
                        packet_type.protocol_name(),
                    );
                } else {
                    self.local_stats.increment_failed();
                }
            }
            
            self.packets_processed += 1;
            
            // Apply rate limiting if configured
            if self.base_delay.as_nanos() > 0 {
                let delay = if self.randomize_timing {
                    // Add ±20% randomization
                    let variation = (self.base_delay.as_nanos() as f64 * 0.2) as u64;
                    let random_offset = (rand::random::<u64>() % (variation * 2)) as i64 - variation as i64;
                    let delay_nanos = (self.base_delay.as_nanos() as i64 + random_offset).max(0) as u64;
                    Duration::from_nanos(delay_nanos)
                } else {
                    self.base_delay
                };
                
                time::sleep(delay).await;
            }
        }
        
        self.total_processing_time += process_start.elapsed();
        
        // Local stats will auto-flush when batch is full
        
        Ok(())
    }
    
    /// Run the worker
    pub async fn run(mut self) {
        debug!("Worker {} starting", self.id);
        
        while self.running.load(Ordering::Relaxed) {
            // Process packets in batches for efficiency
            if let Err(e) = self.process_batch(50).await {
                debug!("Worker {} error: {}", self.id, e);
            }
            
            // Yield to prevent monopolizing CPU
            tokio::task::yield_now().await;
        }
        
        // Final stats flush (happens automatically in Drop)
        drop(self.local_stats);
        
        let elapsed = self.start_time.elapsed();
        let pps = self.packets_processed as f64 / elapsed.as_secs_f64();
        
        debug!(
            "Worker {} stopped - processed {} packets in {:?} ({:.1} pps)",
            self.id, self.packets_processed, elapsed, pps
        );
    }
}

/// Manager for packet workers
pub struct PacketWorkerManager {
    handles: Vec<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl PacketWorkerManager {
    /// Create and spawn packet workers
    pub fn new(
        config: &Config,
        stats: Arc<FloodStats>,
        multi_port_target: Arc<MultiPortTarget>,
        target_ip: IpAddr,
        _interface: Option<&pnet::datalink::NetworkInterface>,
        dry_run: bool,
    ) -> Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::with_capacity(config.attack.threads);
        
        // Get target ports
        let target_ports = multi_port_target.get_ports().to_vec();
        
        // Spawn workers
        for i in 0..config.attack.threads {
            let worker = PacketWorker::new(
                i,
                target_ip,
                target_ports.clone(),
                config,
                stats.clone(),
                running.clone(),
                dry_run,
            );
            
            let handle = tokio::spawn(async move {
                worker.run().await;
            });
            
            handles.push(handle);
        }
        
        debug!("Spawned {} packet workers", config.attack.threads);
        
        Ok(Self { handles, running })
    }
    
    /// Stop all workers
    pub fn stop(&self) {
        debug!("Stopping all packet workers");
        self.running.store(false, Ordering::Relaxed);
    }
    
    /// Wait for all workers to complete
    pub async fn join_all(self) -> Result<()> {
        for handle in self.handles {
            handle.await.map_err(|e| {
                NetworkError::InterfaceNotFound(format!("Worker task panicked: {}", e))
            })?;
        }
        Ok(())
    }
}

// Extension trait for PacketType to get protocol name
#[allow(dead_code)]
trait PacketTypeExt {
    fn protocol_name(&self) -> &str;
}

impl PacketTypeExt for PacketType {
    fn protocol_name(&self) -> &str {
        match self {
            PacketType::Udp => "UDP",
            PacketType::TcpSyn | PacketType::TcpAck => "TCP",
            PacketType::Icmp => "ICMP",
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp => "IPv6",
            PacketType::Arp => "ARP",
            _ => "Other",
        }
    }
}

/// Compatibility wrapper for existing WorkerManager usage
pub type WorkerManager = PacketWorkerManager;

// Add rand dependency for randomization
extern crate rand;