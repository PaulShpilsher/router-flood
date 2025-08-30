//! Simple worker implementations using consolidated traits
//!
//! This module provides simple worker implementations that use the
//! consolidated traits from the traits module.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::error::Result;

// Re-export traits from the consolidated traits module
pub use crate::core::traits::{
    StatsCollector, PacketBuilder, TargetProvider, WorkerConfig
};

/// Streamlined worker that doesn't use async trait objects
pub struct SimpleWorker {
    id: usize,
    stats_collector: Arc<dyn StatsCollector>,
    packet_builder: Box<dyn PacketBuilder>,
    target_provider: Arc<dyn TargetProvider>,
    base_delay: Duration,
    randomize_timing: bool,
    dry_run: bool,
    perfect_simulation: bool,
}

impl SimpleWorker {
    pub fn new(
        id: usize,
        stats_collector: Arc<dyn StatsCollector>,
        packet_builder: Box<dyn PacketBuilder>,
        target_provider: Arc<dyn TargetProvider>,
        packet_rate: u64,
        randomize_timing: bool,
        dry_run: bool,
        perfect_simulation: bool,
    ) -> Self {
        let base_delay = Duration::from_nanos(1_000_000_000 / packet_rate);
        
        Self {
            id,
            stats_collector,
            packet_builder,
            target_provider,
            base_delay,
            randomize_timing,
            dry_run,
            perfect_simulation,
        }
    }
    
    /// Run the worker until stopped
    pub async fn run(&mut self, running: Arc<AtomicBool>) {
        // tokio::time not needed in this test
        
        while running.load(Ordering::Relaxed) {
            // Get next target and packet type
            let target_port = self.target_provider.next_port();
            let packet_type = self.packet_builder.next_packet_type();
            
            // For simplicity, use a default target IP (this would be injected in real implementation)
            let target_ip = "192.168.1.1".parse().unwrap();
            
            // Build packet
            match self.packet_builder.build_packet(packet_type, target_ip, target_port) {
                Ok((packet_data, protocol)) => {
                    // Simulate sending (in real implementation, this would use actual transport)
                    if self.dry_run {
                        if self.perfect_simulation || self.simulate_success() {
                            self.stats_collector.record_packet_sent(protocol, packet_data.len());
                        } else {
                            self.stats_collector.record_packet_failed();
                        }
                    } else {
                        // Would actually send packet here
                        self.stats_collector.record_packet_sent(protocol, packet_data.len());
                    }
                }
                Err(_) => {
                    self.stats_collector.record_packet_failed();
                }
            }
            
            // Apply rate limiting
            self.apply_rate_limiting().await;
        }
    }
    
    /// Get worker ID
    pub fn id(&self) -> usize {
        self.id
    }
    
    /// Simulate packet success for dry run mode
    fn simulate_success(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < 0.98 // 98% success rate
    }
    
    /// Apply rate limiting delay
    async fn apply_rate_limiting(&self) {
        use tokio::time;
        
        let delay = if self.randomize_timing {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(0.8..1.2);
            Duration::from_nanos((self.base_delay.as_nanos() as f64 * jitter) as u64)
        } else {
            self.base_delay
        };
        
        time::sleep(delay).await;
    }
}

/// Factory for creating simple workers
pub struct SimpleWorkerFactory {
    config: Arc<dyn WorkerConfig>,
}

impl SimpleWorkerFactory {
    pub fn new(config: Arc<dyn WorkerConfig>) -> Self {
        Self { config }
    }
    
    pub fn create_worker(
        &self,
        worker_id: usize,
        stats_collector: Arc<dyn StatsCollector>,
        packet_builder: Box<dyn PacketBuilder>,
        target_provider: Arc<dyn TargetProvider>,
    ) -> SimpleWorker {
        let per_worker_rate = self.config.packet_rate() / self.config.thread_count() as u64;
        
        SimpleWorker::new(
            worker_id,
            stats_collector,
            packet_builder,
            target_provider,
            per_worker_rate,
            self.config.randomize_timing(),
            self.config.dry_run(),
            self.config.perfect_simulation(),
        )
    }
}

/// Streamlined worker manager
pub struct SimpleWorkerManager {
    workers: Vec<SimpleWorker>,
    running: Arc<AtomicBool>,
}

impl SimpleWorkerManager {
    pub fn new(
        factory: &SimpleWorkerFactory,
        stats_collector: Arc<dyn StatsCollector>,
        target_provider: Arc<dyn TargetProvider>,
        packet_builders: Vec<Box<dyn PacketBuilder>>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let mut workers = Vec::new();
        
        for (worker_id, packet_builder) in packet_builders.into_iter().enumerate() {
            let worker = factory.create_worker(
                worker_id,
                stats_collector.clone(),
                packet_builder,
                target_provider.clone(),
            );
            workers.push(worker);
        }
        
        Self { workers, running }
    }
    
    pub async fn run(&mut self) -> Result<()> {
        let mut handles = Vec::new();
        
        for mut worker in self.workers.drain(..) {
            let running = self.running.clone();
            let handle = tokio::spawn(async move {
                worker.run(running).await;
            });
            handles.push(handle);
        }
        
        // Wait for all workers to complete
        for handle in handles {
            handle.await.map_err(|e| {
                crate::error::NetworkError::PacketSend(format!("Worker join error: {}", e))
            })?;
        }
        
        Ok(())
    }
    
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
    
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

// Tests moved to tests/ directory
