//! Core interfaces for dependency injection and module decoupling
//!
//! This module defines the core traits and interfaces that enable
//! dependency injection and reduce coupling between modules.

use std::net::IpAddr;
use std::sync::Arc;
use async_trait::async_trait;

use crate::error::Result;
use crate::packet::PacketType;

/// Trait for statistics collection with minimal interface
pub trait StatsCollector: Send + Sync {
    /// Record a successfully sent packet
    fn record_packet_sent(&self, protocol: &str, size: usize);
    
    /// Record a failed packet attempt
    fn record_packet_failed(&self);
    
    /// Get current packet count for monitoring
    fn get_packet_count(&self) -> u64;
    
    /// Get current failure count for monitoring
    fn get_failure_count(&self) -> u64;
}

/// Trait for packet building with minimal dependencies
pub trait PacketBuilder: Send + Sync {
    /// Build a packet for the given parameters
    fn build_packet(
        &mut self,
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
    ) -> Result<(Vec<u8>, &'static str)>;
    
    /// Get the next packet type based on protocol mix
    fn next_packet_type(&mut self) -> PacketType;
    
    /// Get packet type appropriate for the target IP
    fn next_packet_type_for_ip(&mut self, target_ip: IpAddr) -> PacketType;
}

/// Trait for packet transmission
#[async_trait]
pub trait PacketSender: Send + Sync {
    /// Send a packet to the target
    async fn send_packet(
        &self,
        packet_data: &[u8],
        target_ip: IpAddr,
        packet_type: PacketType,
    ) -> Result<()>;
    
    /// Check if sender is in dry-run mode
    fn is_dry_run(&self) -> bool;
}

/// Trait for target port management
pub trait TargetProvider: Send + Sync {
    /// Get the next target port in rotation
    fn next_port(&self) -> u16;
    
    /// Get all configured ports
    fn get_ports(&self) -> &[u16];
}

/// Trait for rate limiting
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Apply rate limiting delay
    async fn apply_delay(&self);
    
    /// Update the target rate
    fn set_rate(&mut self, packets_per_second: u64);
}

/// Configuration provider trait for worker settings
pub trait WorkerConfig: Send + Sync {
    /// Get the number of worker threads
    fn thread_count(&self) -> usize;
    
    /// Get the target packet rate
    fn packet_rate(&self) -> u64;
    
    /// Get packet size range
    fn packet_size_range(&self) -> (usize, usize);
    
    /// Check if timing should be randomized
    fn randomize_timing(&self) -> bool;
    
    /// Check if perfect simulation is enabled
    fn perfect_simulation(&self) -> bool;
    
    /// Check if dry run mode is enabled
    fn dry_run(&self) -> bool;
}

/// Factory trait for creating workers with dependencies
pub trait WorkerFactory: Send + Sync {
    /// Create a new worker with injected dependencies
    fn create_worker(
        &self,
        worker_id: usize,
        stats_collector: Arc<dyn StatsCollector>,
        packet_builder: Box<dyn PacketBuilder>,
        packet_sender: Arc<dyn PacketSender>,
        target_provider: Arc<dyn TargetProvider>,
        rate_limiter: Box<dyn RateLimiter>,
    ) -> Box<dyn Worker>;
}

/// Worker trait for packet processing
#[async_trait]
pub trait Worker: Send + Sync {
    /// Run the worker until stopped
    async fn run(&mut self, running: Arc<std::sync::atomic::AtomicBool>);
    
    /// Get worker ID
    fn id(&self) -> usize;
}

/// Simplified worker implementation using dependency injection
pub struct InjectedWorker {
    id: usize,
    stats_collector: Arc<dyn StatsCollector>,
    packet_builder: Box<dyn PacketBuilder>,
    packet_sender: Arc<dyn PacketSender>,
    target_provider: Arc<dyn TargetProvider>,
    rate_limiter: Box<dyn RateLimiter>,
}

impl InjectedWorker {
    pub fn new(
        id: usize,
        stats_collector: Arc<dyn StatsCollector>,
        packet_builder: Box<dyn PacketBuilder>,
        packet_sender: Arc<dyn PacketSender>,
        target_provider: Arc<dyn TargetProvider>,
        rate_limiter: Box<dyn RateLimiter>,
    ) -> Self {
        Self {
            id,
            stats_collector,
            packet_builder,
            packet_sender,
            target_provider,
            rate_limiter,
        }
    }
}

#[async_trait]
impl Worker for InjectedWorker {
    async fn run(&mut self, running: Arc<std::sync::atomic::AtomicBool>) {
        use std::sync::atomic::Ordering;
        
        while running.load(Ordering::Relaxed) {
            // Get next target and packet type
            let target_port = self.target_provider.next_port();
            let packet_type = self.packet_builder.next_packet_type();
            
            // For simplicity, use a default target IP (this would be injected in real implementation)
            let target_ip = "192.168.1.1".parse().unwrap();
            
            // Build packet
            match self.packet_builder.build_packet(packet_type, target_ip, target_port) {
                Ok((packet_data, protocol)) => {
                    // Send packet
                    match self.packet_sender.send_packet(&packet_data, target_ip, packet_type).await {
                        Ok(()) => {
                            self.stats_collector.record_packet_sent(protocol, packet_data.len());
                        }
                        Err(_) => {
                            self.stats_collector.record_packet_failed();
                        }
                    }
                }
                Err(_) => {
                    self.stats_collector.record_packet_failed();
                }
            }
            
            // Apply rate limiting
            self.rate_limiter.apply_delay().await;
        }
    }
    
    fn id(&self) -> usize {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
    use std::time::Duration;
    use tokio::time;
    
    // Mock implementations for testing
    struct MockStatsCollector {
        sent_count: AtomicU64,
        failed_count: AtomicU64,
    }
    
    impl MockStatsCollector {
        fn new() -> Self {
            Self {
                sent_count: AtomicU64::new(0),
                failed_count: AtomicU64::new(0),
            }
        }
    }
    
    impl StatsCollector for MockStatsCollector {
        fn record_packet_sent(&self, _protocol: &str, _size: usize) {
            self.sent_count.fetch_add(1, Ordering::Relaxed);
        }
        
        fn record_packet_failed(&self) {
            self.failed_count.fetch_add(1, Ordering::Relaxed);
        }
        
        fn get_packet_count(&self) -> u64 {
            self.sent_count.load(Ordering::Relaxed)
        }
        
        fn get_failure_count(&self) -> u64 {
            self.failed_count.load(Ordering::Relaxed)
        }
    }
    
    struct MockPacketBuilder;
    
    impl PacketBuilder for MockPacketBuilder {
        fn build_packet(
            &mut self,
            _packet_type: PacketType,
            _target_ip: IpAddr,
            _target_port: u16,
        ) -> Result<(Vec<u8>, &'static str)> {
            Ok((vec![0u8; 64], "UDP"))
        }
        
        fn next_packet_type(&mut self) -> PacketType {
            PacketType::Udp
        }
        
        fn next_packet_type_for_ip(&mut self, _target_ip: IpAddr) -> PacketType {
            PacketType::Udp
        }
    }
    
    struct MockPacketSender {
        dry_run: bool,
    }
    
    impl MockPacketSender {
        fn new(dry_run: bool) -> Self {
            Self { dry_run }
        }
    }
    
    #[async_trait]
    impl PacketSender for MockPacketSender {
        async fn send_packet(
            &self,
            _packet_data: &[u8],
            _target_ip: IpAddr,
            _packet_type: PacketType,
        ) -> Result<()> {
            if self.dry_run {
                // Simulate sending
                Ok(())
            } else {
                // Would actually send packet
                Ok(())
            }
        }
        
        fn is_dry_run(&self) -> bool {
            self.dry_run
        }
    }
    
    struct MockTargetProvider {
        ports: Vec<u16>,
        current: AtomicU64,
    }
    
    impl MockTargetProvider {
        fn new(ports: Vec<u16>) -> Self {
            Self {
                ports,
                current: AtomicU64::new(0),
            }
        }
    }
    
    impl TargetProvider for MockTargetProvider {
        fn next_port(&self) -> u16 {
            let index = self.current.fetch_add(1, Ordering::Relaxed) as usize % self.ports.len();
            self.ports[index]
        }
        
        fn get_ports(&self) -> &[u16] {
            &self.ports
        }
    }
    
    struct MockRateLimiter {
        delay: Duration,
    }
    
    impl MockRateLimiter {
        fn new(packets_per_second: u64) -> Self {
            let delay = Duration::from_nanos(1_000_000_000 / packets_per_second);
            Self { delay }
        }
    }
    
    #[async_trait]
    impl RateLimiter for MockRateLimiter {
        async fn apply_delay(&self) {
            time::sleep(self.delay).await;
        }
        
        fn set_rate(&mut self, packets_per_second: u64) {
            self.delay = Duration::from_nanos(1_000_000_000 / packets_per_second);
        }
    }
    
    #[tokio::test]
    async fn test_injected_worker() {
        let stats = Arc::new(MockStatsCollector::new());
        let packet_builder = Box::new(MockPacketBuilder);
        let packet_sender = Arc::new(MockPacketSender::new(true));
        let target_provider = Arc::new(MockTargetProvider::new(vec![80, 443]));
        let rate_limiter = Box::new(MockRateLimiter::new(1000));
        
        let mut worker = InjectedWorker::new(
            0,
            stats.clone(),
            packet_builder,
            packet_sender,
            target_provider,
            rate_limiter,
        );
        
        let running = Arc::new(AtomicBool::new(true));
        
        // Run worker for a short time
        let worker_running = running.clone();
        let worker_handle = tokio::spawn(async move {
            worker.run(worker_running).await;
        });
        
        // Let it run briefly
        time::sleep(Duration::from_millis(10)).await;
        
        // Stop the worker
        running.store(false, Ordering::Relaxed);
        
        // Wait for completion
        let _ = worker_handle.await;
        
        // Verify some packets were processed
        assert!(stats.get_packet_count() > 0);
    }
}