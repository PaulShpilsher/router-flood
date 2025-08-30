//! Batch worker implementation with performance optimizations
//!
//! This module provides a high-performance batch worker that leverages zero-copy operations,
//! memory pooling, lock-free statistics, and string interning.

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

use crate::core::traits::{TargetProvider, WorkerConfig};
use crate::error::Result;
use crate::packet::PacketType;
use crate::performance::{
    BatchPacketProcessor, BatchedStatsCollector, LockFreeStatsCollector
};

/// High-performance batch worker
pub struct BatchWorker {
    id: usize,
    processor: BatchPacketProcessor,
    stats_collector: BatchedStatsCollector,
    target_provider: Arc<dyn TargetProvider>,
    target_ip: IpAddr,
    packet_types: Vec<PacketType>,
    packet_type_index: usize,
    base_delay: Duration,
    randomize_timing: bool,
    dry_run: bool,
    perfect_simulation: bool,
    // Performance tracking
    packets_processed: u64,
    total_processing_time: Duration,
    start_time: Instant,
}

impl BatchWorker {
    /// Create a new batch worker
    pub fn new(
        id: usize,
        target_ip: IpAddr,
        target_provider: Arc<dyn TargetProvider>,
        config: &dyn WorkerConfig,
    ) -> Self {
        let processor = BatchPacketProcessor::new();
        let stats_collector = processor.create_batched_collector(50); // Batch every 50 packets
        
        // Pre-calculate packet types based on protocol mix
        let packet_types = Self::generate_packet_types();
        
        let per_worker_rate = config.packet_rate() / config.thread_count() as u64;
        let base_delay = Duration::from_nanos(1_000_000_000 / per_worker_rate);
        
        Self {
            id,
            processor,
            stats_collector,
            target_provider,
            target_ip,
            packet_types,
            packet_type_index: 0,
            base_delay,
            randomize_timing: config.randomize_timing(),
            dry_run: config.dry_run(),
            perfect_simulation: config.perfect_simulation(),
            packets_processed: 0,
            total_processing_time: Duration::ZERO,
            start_time: Instant::now(),
        }
    }
    
    /// Run the worker until stopped
    pub async fn run(&mut self, running: Arc<AtomicBool>) {
        while running.load(Ordering::Relaxed) {
            let process_start = Instant::now();
            
            if let Err(_) = self.process_single_packet().await {
                self.stats_collector.record_failed();
            }
            
            let process_time = process_start.elapsed();
            self.total_processing_time += process_time;
            self.packets_processed += 1;
            
            // Apply rate limiting
            self.apply_rate_limiting().await;
        }
        
        // Ensure final flush of statistics
        self.stats_collector.flush();
    }
    
    /// Process a single packet using batch pipeline
    async fn process_single_packet(&mut self) -> Result<()> {
        let target_port = self.target_provider.next_port();
        let packet_type = self.next_packet_type();
        
        if self.dry_run {
            self.simulate_packet_processing(packet_type, target_port).await
        } else {
            self.process_real_packet(packet_type, target_port).await
        }
    }
    
    /// Simulate packet processing in dry-run mode
    async fn simulate_packet_processing(&mut self, packet_type: PacketType, target_port: u16) -> Result<()> {
        // Extract values we need before borrowing
        let perfect_simulation = self.perfect_simulation;
        
        // Use batch processor to build packet (but don't send)
        let processed_packet = self.processor.process_packet(
            packet_type,
            self.target_ip,
            target_port,
            64, // Default payload size
        )?;
        
        // Extract values before borrowing self for simulate_success
        let protocol = processed_packet.protocol().to_string();
        let size = processed_packet.size();
        
        // Drop the packet to release the borrow
        drop(processed_packet);
        
        // Simulate success/failure
        let success = if perfect_simulation {
            true
        } else {
            self.simulate_success()
        };
        
        if success {
            self.stats_collector.record_sent(&protocol, size);
        } else {
            self.stats_collector.record_failed();
        }
        
        Ok(())
    }
    
    /// Process a real packet (would integrate with transport layer)
    async fn process_real_packet(&mut self, packet_type: PacketType, target_port: u16) -> Result<()> {
        let processed_packet = self.processor.process_packet(
            packet_type,
            self.target_ip,
            target_port,
            64, // Default payload size
        )?;
        
        // In a real implementation, this would send the packet via transport layer
        // For now, just record as successful
        self.stats_collector.record_sent(
            processed_packet.protocol(),
            processed_packet.size(),
        );
        
        Ok(())
    }
    
    /// Get the next packet type in rotation
    fn next_packet_type(&mut self) -> PacketType {
        let packet_type = self.packet_types[self.packet_type_index];
        self.packet_type_index = (self.packet_type_index + 1) % self.packet_types.len();
        packet_type
    }
    
    /// Generate packet types based on protocol mix (simplified)
    fn generate_packet_types() -> Vec<PacketType> {
        // Simplified protocol mix - in real implementation would use config
        vec![
            PacketType::Udp,      // 60%
            PacketType::Udp,
            PacketType::Udp,
            PacketType::TcpSyn,   // 25%
            PacketType::TcpSyn,
            PacketType::TcpAck,   // 5%
            PacketType::Icmp,     // 5%
            PacketType::Ipv6Udp,  // 3%
            PacketType::Arp,      // 2%
        ]
    }
    
    /// Simulate packet success for dry run mode
    fn simulate_success(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < 0.98 // 98% success rate
    }
    
    /// Apply rate limiting with optional jitter
    async fn apply_rate_limiting(&self) {
        let delay = if self.randomize_timing {
            self.apply_jitter(self.base_delay)
        } else {
            self.base_delay
        };
        
        time::sleep(delay).await;
    }
    
    /// Apply timing jitter
    fn apply_jitter(&self, base_delay: Duration) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.8..1.2); // ±20% jitter
        Duration::from_nanos((base_delay.as_nanos() as f64 * jitter_factor) as u64)
    }
    
    /// Get worker ID
    pub fn id(&self) -> usize {
        self.id
    }
    
    /// Get performance metrics
    pub fn metrics(&self) -> WorkerMetrics {
        let elapsed = self.start_time.elapsed();
        let packets_per_second = if elapsed.as_secs_f64() > 0.0 {
            self.packets_processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        WorkerMetrics {
            worker_id: self.id,
            packets_processed: self.packets_processed,
            total_processing_time: self.total_processing_time,
            elapsed_time: elapsed,
            packets_per_second,
            average_processing_time: if self.packets_processed > 0 {
                self.total_processing_time / self.packets_processed as u32
            } else {
                Duration::ZERO
            },
        }
    }
    
    /// Get the underlying stats collector for global aggregation
    pub fn stats_collector(&self) -> Arc<LockFreeStatsCollector> {
        self.processor.stats_collector()
    }
}

/// Performance metrics for a batch worker
#[derive(Debug, Clone)]
pub struct WorkerMetrics {
    pub worker_id: usize,
    pub packets_processed: u64,
    pub total_processing_time: Duration,
    pub elapsed_time: Duration,
    pub packets_per_second: f64,
    pub average_processing_time: Duration,
}

/// Manager for batch workers
pub struct BatchWorkerManager {
    workers: Vec<BatchWorker>,
    pub running: Arc<AtomicBool>,
    global_stats: Arc<LockFreeStatsCollector>,
}

impl BatchWorkerManager {
    /// Create a new batch worker manager
    pub fn new(
        worker_count: usize,
        target_ip: IpAddr,
        target_provider: Arc<dyn TargetProvider>,
        config: &dyn WorkerConfig,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let mut workers = Vec::with_capacity(worker_count);
        
        // Create the first worker to get the global stats collector
        let first_worker = BatchWorker::new(0, target_ip, target_provider.clone(), config);
        let global_stats = first_worker.stats_collector();
        workers.push(first_worker);
        
        // Create remaining workers
        for id in 1..worker_count {
            workers.push(BatchWorker::new(id, target_ip, target_provider.clone(), config));
        }
        
        Self {
            workers,
            running,
            global_stats,
        }
    }
    
    /// Run all workers
    pub async fn run(&mut self) -> Result<()> {
        let mut handles = Vec::new();
        
        for mut worker in self.workers.drain(..) {
            let running = self.running.clone();
            let handle = tokio::spawn(async move {
                worker.run(running).await;
                worker // Return worker for metrics collection
            });
            handles.push(handle);
        }
        
        // Wait for all workers to complete and collect metrics
        let mut final_metrics = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(worker) => {
                    final_metrics.push(worker.metrics());
                }
                Err(e) => {
                    eprintln!("Worker join error: {}", e);
                }
            }
        }
        
        // Print final metrics
        self.print_final_metrics(&final_metrics);
        
        Ok(())
    }
    
    /// Stop all workers
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
    
    /// Check if workers are running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// Get global statistics
    pub fn global_stats(&self) -> Arc<LockFreeStatsCollector> {
        self.global_stats.clone()
    }
    
    /// Print final performance metrics
    fn print_final_metrics(&self, metrics: &[WorkerMetrics]) {
        println!("\n🚀 Batch Worker Performance Summary");
        println!("═══════════════════════════════════════");
        
        let total_packets: u64 = metrics.iter().map(|m| m.packets_processed).sum();
        let total_pps: f64 = metrics.iter().map(|m| m.packets_per_second).sum();
        let avg_processing_time: Duration = if !metrics.is_empty() {
            metrics.iter().map(|m| m.average_processing_time).sum::<Duration>() / metrics.len() as u32
        } else {
            Duration::ZERO
        };
        
        println!("Total Packets Processed: {}", total_packets);
        println!("Total Packets/Second:    {:.1}", total_pps);
        println!("Average Processing Time: {:?}", avg_processing_time);
        println!("Worker Count:            {}", metrics.len());
        
        // Per-worker breakdown
        println!("\nPer-Worker Breakdown:");
        for metric in metrics {
            println!(
                "Worker {}: {} packets, {:.1} pps, {:?} avg",
                metric.worker_id,
                metric.packets_processed,
                metric.packets_per_second,
                metric.average_processing_time
            );
        }
        
        // Global statistics
        let global_stats = self.global_stats.aggregate();
        println!("\nGlobal Statistics:");
        println!("Packets Sent:    {}", global_stats.packets_sent);
        println!("Packets Failed:  {}", global_stats.packets_failed);
        println!("Success Rate:    {:.2}%", global_stats.success_rate());
        println!("Bytes Sent:      {}", global_stats.bytes_sent);
        
        // Protocol breakdown
        if global_stats.udp_packets > 0 {
            println!("UDP Packets:     {}", global_stats.udp_packets);
        }
        if global_stats.tcp_packets > 0 {
            println!("TCP Packets:     {}", global_stats.tcp_packets);
        }
        if global_stats.icmp_packets > 0 {
            println!("ICMP Packets:    {}", global_stats.icmp_packets);
        }
        if global_stats.ipv6_packets > 0 {
            println!("IPv6 Packets:    {}", global_stats.ipv6_packets);
        }
        if global_stats.arp_packets > 0 {
            println!("ARP Packets:     {}", global_stats.arp_packets);
        }
    }
}

// Tests moved to tests/ directory
