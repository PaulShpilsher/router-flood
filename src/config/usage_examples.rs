//! Usage examples for configuration traits
//!
//! This module demonstrates how different parts of the codebase can use
//! only the configuration traits they need, following Interface Segregation Principle.

use super::traits::*;
use super::Config;
use crate::error::Result;

/// Example: A packet generator only needs target and packet configuration
pub struct PacketGenerator<C: PacketGenerationConfiguration> {
    config: C,
}

impl<C: PacketGenerationConfiguration> PacketGenerator<C> {
    pub fn new(config: C) -> Self {
        Self { config }
    }
    
    pub fn generate_packet(&self) -> Vec<u8> {
        let _target_ip = self.config.target_ip();
        let _ports = self.config.target_ports();
        let packet_size = self.config.min_packet_size();
        
        // Generate packet based on protocol mix
        let _udp_ratio = self.config.udp_ratio();
        let _tcp_syn_ratio = self.config.tcp_syn_ratio();
        
        // Simplified packet generation logic
        vec![0u8; packet_size]
    }
}

/// Example: A safety validator only needs safety configuration
pub struct SafetyValidator<C: SafetyConfiguration> {
    config: C,
}

impl<C: SafetyConfiguration> SafetyValidator<C> {
    pub fn new(config: C) -> Self {
        Self { config }
    }
    
    pub fn validate_operation(&self, threads: usize, rate: u64) -> Result<()> {
        if threads > self.config.max_threads() {
            return Err(crate::error::ValidationError::ExceedsLimit {
                field: "threads".to_string(),
                value: threads as u64,
                limit: self.config.max_threads() as u64,
            }.into());
        }
        
        if rate > self.config.max_packet_rate() {
            return Err(crate::error::ValidationError::ExceedsLimit {
                field: "packet_rate".to_string(),
                value: rate,
                limit: self.config.max_packet_rate(),
            }.into());
        }
        
        if self.config.is_dry_run() {
            println!("DRY RUN: Operation would be performed");
        }
        
        Ok(())
    }
}

/// Example: A monitoring system only needs monitoring configuration
pub struct MonitoringSystem<C: ObservabilityConfiguration> {
    config: C,
}

impl<C: ObservabilityConfiguration> MonitoringSystem<C> {
    pub fn new(config: C) -> Self {
        Self { config }
    }
    
    pub fn should_export(&self) -> bool {
        self.config.export_enabled()
    }
    
    pub fn get_stats_interval(&self) -> u64 {
        self.config.stats_interval()
    }
    
    pub fn export_format(&self) -> &str {
        self.config.export_format()
    }
}

/// Example: A thread pool only needs performance configuration
pub struct ThreadPool<C: PerformanceConfiguration> {
    config: C,
}

impl<C: PerformanceConfiguration> ThreadPool<C> {
    pub fn new(config: C) -> Self {
        Self { config }
    }
    
    pub fn spawn_workers(&self) {
        let thread_count = self.config.thread_count();
        let packet_rate = self.config.packet_rate();
        let rate_per_thread = packet_rate / thread_count as u64;
        
        for i in 0..thread_count {
            // Spawn worker thread with rate_per_thread
            println!("Worker {}: {} pps", i, rate_per_thread);
        }
        
        if let Some(duration) = self.config.duration() {
            println!("Running for {} seconds", duration);
        }
    }
}

/// Example: Security auditor only needs security configuration
pub struct SecurityAuditor<C: SecurityConfiguration> {
    config: C,
}

impl<C: SecurityConfiguration> SecurityAuditor<C> {
    pub fn new(config: C) -> Self {
        Self { config }
    }
    
    pub fn log_event(&self, event: &str) {
        if self.config.audit_logging_enabled() {
            println!("[AUDIT] {}", event);
        }
    }
    
    pub fn should_monitor(&self) -> bool {
        self.config.monitoring_enabled()
    }
}

/// Example function that only needs basic configuration
pub fn display_summary<C: BasicConfiguration>(config: &C) {
    println!("Target: {}:{:?}", config.target_ip(), config.target_ports());
    println!("Threads: {}", config.thread_count());
    println!("Rate: {} pps", config.packet_rate());
}

/// Example function that validates protocol ratios
pub fn validate_protocols<C: ProtocolConfiguration>(config: &C) -> Result<()> {
    if !config.validate_ratios() {
        return Err(crate::error::ConfigError::InvalidValue {
            field: "protocol_mix".to_string(),
            value: "ratios".to_string(),
            reason: "Protocol ratios must sum to 1.0".to_string(),
        }.into());
    }
    Ok(())
}

/// Example: Using views for focused access
pub fn use_config_views(config: &Config) {
    use super::trait_impls::ConfigExt;
    
    // Get a target view for network operations
    let target_view = config.target_view();
    println!("Targeting: {}", target_view.target_ip());
    
    // Get a safety view for validation
    let safety_view = config.safety_view();
    if let Err(e) = safety_view.validate_limits(100, 1000000) {
        println!("Safety check failed: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_default_config;
    
    #[test]
    fn test_packet_generator() {
        let config = get_default_config();
        let generator = PacketGenerator::new(config);
        let packet = generator.generate_packet();
        assert!(!packet.is_empty());
    }
    
    #[test]
    fn test_safety_validator() {
        let config = get_default_config();
        let validator = SafetyValidator::new(config);
        
        // Should pass with values within default limits (MAX_THREADS=100, MAX_PACKET_RATE=10000)
        assert!(validator.validate_operation(10, 1000).is_ok());
        
        // Should fail with excessive values
        assert!(validator.validate_operation(10000, 100000).is_err());
    }
    
    #[test]
    fn test_monitoring_system() {
        let config = get_default_config();
        let monitor = MonitoringSystem::new(config);
        
        // Default stats_interval is 5 (from DEFAULT_STATS_INTERVAL)
        assert_eq!(monitor.get_stats_interval(), 5);
        assert_eq!(monitor.export_format(), "json");
    }
    
    #[test]
    fn test_display_summary() {
        let config = get_default_config();
        display_summary(&config); // Should compile and run
    }
    
    #[test]
    fn test_protocol_validation() {
        let config = get_default_config();
        assert!(validate_protocols(&config).is_ok());
    }
}