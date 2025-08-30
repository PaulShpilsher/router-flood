//! Adapter implementations for bridging existing code with new interfaces
//!
//! This module provides adapter implementations that allow existing
//! components to work with the new dependency injection interfaces.

use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use async_trait::async_trait;
use tokio::time;

use crate::core::interfaces::{
    StatsCollector, PacketBuilder as PacketBuilderTrait, PacketSender, 
    TargetProvider, RateLimiter, WorkerConfig
};
use crate::core::target::MultiPortTarget;
use crate::config::{Config, ProtocolMix};
use crate::constants::{NANOSECONDS_PER_SECOND, timing};
use crate::error::Result;
use crate::packet::{PacketBuilder, PacketType};
use crate::stats::FloodStats;
use crate::transport::WorkerChannels;

/// Adapter for FloodStats to implement StatsCollector trait
pub struct FloodStatsAdapter {
    inner: Arc<FloodStats>,
}

impl FloodStatsAdapter {
    pub fn new(stats: Arc<FloodStats>) -> Self {
        Self { inner: stats }
    }
}

impl StatsCollector for FloodStatsAdapter {
    fn record_packet_sent(&self, protocol: &str, size: usize) {
        self.inner.increment_sent(size as u64, protocol);
    }
    
    fn record_packet_failed(&self) {
        self.inner.increment_failed();
    }
    
    fn get_packet_count(&self) -> u64 {
        self.inner.packets_sent.load(Ordering::Relaxed)
    }
    
    fn get_failure_count(&self) -> u64 {
        self.inner.packets_failed.load(Ordering::Relaxed)
    }
}

/// Adapter for PacketBuilder to implement PacketBuilderTrait
pub struct PacketBuilderAdapter {
    inner: PacketBuilder,
}

impl PacketBuilderAdapter {
    pub fn new(packet_size_range: (usize, usize), protocol_mix: ProtocolMix) -> Self {
        Self {
            inner: PacketBuilder::new(packet_size_range, protocol_mix),
        }
    }
}

impl PacketBuilderTrait for PacketBuilderAdapter {
    fn build_packet(
        &mut self,
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
    ) -> Result<(Vec<u8>, &'static str)> {
        self.inner.build_packet(packet_type, target_ip, target_port)
    }
    
    fn next_packet_type(&mut self) -> PacketType {
        self.inner.next_packet_type()
    }
    
    fn next_packet_type_for_ip(&mut self, target_ip: IpAddr) -> PacketType {
        self.inner.next_packet_type_for_ip(target_ip)
    }
}

/// Adapter for WorkerChannels to implement PacketSender trait
pub struct WorkerChannelsAdapter {
    channels: WorkerChannels,
    dry_run: bool,
    perfect_simulation: bool,
    success_rate: f64,
}

impl WorkerChannelsAdapter {
    pub fn new(channels: WorkerChannels, dry_run: bool, perfect_simulation: bool) -> Self {
        Self {
            channels,
            dry_run,
            perfect_simulation,
            success_rate: 0.98, // 98% success rate for realistic simulation
        }
    }
}

#[async_trait]
impl PacketSender for WorkerChannelsAdapter {
    async fn send_packet(
        &self,
        packet_data: &[u8],
        target_ip: IpAddr,
        packet_type: PacketType,
    ) -> Result<()> {
        if self.dry_run {
            // Simulate packet sending
            if self.perfect_simulation {
                Ok(())
            } else {
                // Simulate realistic failure rate
                use rand::Rng;
                let mut rng = rand::thread_rng();
                if rng.gen::<f64>() < self.success_rate {
                    Ok(())
                } else {
                    Err(crate::error::NetworkError::PacketSend("Simulated failure".to_string()).into())
                }
            }
        } else {
            // Determine channel type based on packet type
            let channel_type = match packet_type {
                PacketType::Udp | PacketType::TcpSyn | PacketType::TcpAck | PacketType::Icmp => {
                    crate::transport::ChannelType::IPv4
                }
                PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp => {
                    crate::transport::ChannelType::IPv6
                }
                PacketType::Arp => {
                    crate::transport::ChannelType::Layer2
                }
            };
            
            self.channels.send_packet(packet_data, target_ip, channel_type)
        }
    }
    
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

/// Adapter for MultiPortTarget to implement TargetProvider trait
pub struct MultiPortTargetAdapter {
    inner: Arc<MultiPortTarget>,
}

impl MultiPortTargetAdapter {
    pub fn new(target: Arc<MultiPortTarget>) -> Self {
        Self { inner: target }
    }
}

impl TargetProvider for MultiPortTargetAdapter {
    fn next_port(&self) -> u16 {
        self.inner.next_port()
    }
    
    fn get_ports(&self) -> &[u16] {
        // Note: MultiPortTarget doesn't expose ports directly,
        // so we'd need to modify it or store ports separately
        // For now, return empty slice as placeholder
        &[]
    }
}

/// Simple rate limiter implementation
pub struct SimpleRateLimiter {
    delay: Duration,
    randomize: bool,
}

impl SimpleRateLimiter {
    pub fn new(packets_per_second: u64, randomize: bool) -> Self {
        let delay = Duration::from_nanos(NANOSECONDS_PER_SECOND / packets_per_second);
        Self { delay, randomize }
    }
}

#[async_trait]
impl RateLimiter for SimpleRateLimiter {
    async fn apply_delay(&self) {
        let actual_delay = if self.randomize {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(timing::JITTER_MIN..timing::JITTER_MAX);
            Duration::from_nanos((self.delay.as_nanos() as f64 * jitter) as u64)
        } else {
            self.delay
        };
        
        time::sleep(actual_delay).await;
    }
    
    fn set_rate(&mut self, packets_per_second: u64) {
        self.delay = Duration::from_nanos(NANOSECONDS_PER_SECOND / packets_per_second);
    }
}

/// Adapter for Config to implement WorkerConfig trait
pub struct ConfigAdapter {
    config: Config,
}

impl ConfigAdapter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl WorkerConfig for ConfigAdapter {
    fn thread_count(&self) -> usize {
        self.config.attack.threads
    }
    
    fn packet_rate(&self) -> u64 {
        self.config.attack.packet_rate
    }
    
    fn packet_size_range(&self) -> (usize, usize) {
        self.config.attack.packet_size_range
    }
    
    fn randomize_timing(&self) -> bool {
        self.config.attack.randomize_timing
    }
    
    fn perfect_simulation(&self) -> bool {
        self.config.safety.perfect_simulation
    }
    
    fn dry_run(&self) -> bool {
        self.config.safety.dry_run
    }
}

/// Factory for creating workers with dependency injection
pub struct DefaultWorkerFactory {
    config: Arc<dyn WorkerConfig>,
}

impl DefaultWorkerFactory {
    pub fn new(config: Arc<dyn WorkerConfig>) -> Self {
        Self { config }
    }
}

impl crate::core::interfaces::WorkerFactory for DefaultWorkerFactory {
    fn create_worker(
        &self,
        worker_id: usize,
        stats_collector: Arc<dyn StatsCollector>,
        packet_builder: Box<dyn PacketBuilderTrait>,
        packet_sender: Arc<dyn PacketSender>,
        target_provider: Arc<dyn TargetProvider>,
        rate_limiter: Box<dyn RateLimiter>,
    ) -> Box<dyn crate::core::interfaces::Worker> {
        Box::new(crate::core::interfaces::InjectedWorker::new(
            worker_id,
            stats_collector,
            packet_builder,
            packet_sender,
            target_provider,
            rate_limiter,
        ))
    }
}

/// Simplified worker manager using dependency injection
pub struct InjectedWorkerManager {
    workers: Vec<Box<dyn crate::core::interfaces::Worker>>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl InjectedWorkerManager {
    pub fn new(
        factory: &dyn crate::core::interfaces::WorkerFactory,
        config: &dyn WorkerConfig,
        stats: Arc<FloodStats>,
        target: Arc<MultiPortTarget>,
        channels: Vec<WorkerChannels>,
    ) -> Self {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let mut workers = Vec::new();
        
        for (worker_id, worker_channels) in channels.into_iter().enumerate() {
            let stats_collector = Arc::new(FloodStatsAdapter::new(stats.clone()));
            let packet_builder = Box::new(PacketBuilderAdapter::new(
                config.packet_size_range(),
                // Note: We'd need to get protocol_mix from config
                // For now, use default values
                crate::config::ProtocolMix {
                    udp_ratio: 0.6,
                    tcp_syn_ratio: 0.25,
                    tcp_ack_ratio: 0.05,
                    icmp_ratio: 0.05,
                    ipv6_ratio: 0.03,
                    arp_ratio: 0.02,
                },
            ));
            let packet_sender = Arc::new(WorkerChannelsAdapter::new(
                worker_channels,
                config.dry_run(),
                config.perfect_simulation(),
            ));
            let target_provider = Arc::new(MultiPortTargetAdapter::new(target.clone()));
            let rate_limiter = Box::new(SimpleRateLimiter::new(
                config.packet_rate() / config.thread_count() as u64,
                config.randomize_timing(),
            ));
            
            let worker = factory.create_worker(
                worker_id,
                stats_collector,
                packet_builder,
                packet_sender,
                target_provider,
                rate_limiter,
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
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_default_config;
    
    #[test]
    fn test_flood_stats_adapter() {
        let stats = Arc::new(FloodStats::default());
        let adapter = FloodStatsAdapter::new(stats.clone());
        
        adapter.record_packet_sent("UDP", 64);
        adapter.record_packet_failed();
        
        assert_eq!(adapter.get_packet_count(), 1);
        assert_eq!(adapter.get_failure_count(), 1);
    }
    
    #[test]
    fn test_config_adapter() {
        let config = get_default_config();
        let adapter = ConfigAdapter::new(config.clone());
        
        assert_eq!(adapter.thread_count(), config.attack.threads);
        assert_eq!(adapter.packet_rate(), config.attack.packet_rate);
        assert_eq!(adapter.dry_run(), config.safety.dry_run);
    }
    
    #[tokio::test]
    async fn test_simple_rate_limiter() {
        let mut limiter = SimpleRateLimiter::new(1000, false);
        
        let start = std::time::Instant::now();
        limiter.apply_delay().await;
        let elapsed = start.elapsed();
        
        // Should be approximately 1ms for 1000 pps
        assert!(elapsed >= Duration::from_micros(900));
        assert!(elapsed <= Duration::from_micros(1100));
    }
}