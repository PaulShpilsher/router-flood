//! Configuration traits for interface segregation
//!
//! This module provides focused traits that represent different aspects of configuration,
//! following the Interface Segregation Principle (ISP) from SOLID.

use std::net::IpAddr;

// ===== Core Target Configuration =====

/// Configuration for network targeting
pub trait TargetConfiguration {
    /// Get the target IP address
    fn target_ip(&self) -> &str;
    
    /// Get the target ports
    fn target_ports(&self) -> &[u16];
    
    /// Get the network interface to use (if specified)
    fn network_interface(&self) -> Option<&str>;
}

/// Configuration for protocol selection
pub trait ProtocolConfiguration {
    /// Get UDP packet ratio (0.0 - 1.0)
    fn udp_ratio(&self) -> f64;
    
    /// Get TCP SYN packet ratio (0.0 - 1.0)
    fn tcp_syn_ratio(&self) -> f64;
    
    /// Get TCP ACK packet ratio (0.0 - 1.0)
    fn tcp_ack_ratio(&self) -> f64;
    
    /// Get ICMP packet ratio (0.0 - 1.0)
    fn icmp_ratio(&self) -> f64;
    
    /// Get IPv6 packet ratio (0.0 - 1.0)
    fn ipv6_ratio(&self) -> f64;
    
    /// Get ARP packet ratio (0.0 - 1.0)
    fn arp_ratio(&self) -> f64;
    
    /// Validate that all ratios sum to 1.0 (within tolerance)
    fn validate_ratios(&self) -> bool {
        let total = self.udp_ratio() + self.tcp_syn_ratio() + self.tcp_ack_ratio() 
                  + self.icmp_ratio() + self.ipv6_ratio() + self.arp_ratio();
        (total - 1.0).abs() < 0.001
    }
}

// ===== Performance Configuration =====

/// Configuration for performance and threading
pub trait PerformanceConfiguration {
    /// Get the number of worker threads
    fn thread_count(&self) -> usize;
    
    /// Get the packet rate per thread (packets per second)
    fn packet_rate(&self) -> u64;
    
    /// Get the test duration in seconds (if limited)
    fn duration(&self) -> Option<u64>;
    
    /// Check if timing should be randomized
    fn randomize_timing(&self) -> bool;
}

/// Configuration for packet characteristics
pub trait PacketConfiguration {
    /// Get the minimum packet size in bytes
    fn min_packet_size(&self) -> usize;
    
    /// Get the maximum packet size in bytes
    fn max_packet_size(&self) -> usize;
    
    /// Get the packet size range as a tuple
    fn packet_size_range(&self) -> (usize, usize) {
        (self.min_packet_size(), self.max_packet_size())
    }
}

// ===== Safety Configuration =====

/// Configuration for safety limits and constraints
pub trait SafetyConfiguration {
    /// Get the maximum allowed threads
    fn max_threads(&self) -> usize;
    
    /// Get the maximum allowed packet rate
    fn max_packet_rate(&self) -> u64;
    
    /// Check if only private IP ranges are allowed
    fn require_private_ranges(&self) -> bool;
    
    /// Check if this is a dry-run (no packets sent)
    fn is_dry_run(&self) -> bool;
    
    /// Check if perfect simulation is enabled (100% success rate)
    fn is_perfect_simulation(&self) -> bool;
}

/// Configuration for security and auditing
pub trait SecurityConfiguration {
    /// Check if audit logging is enabled
    fn audit_logging_enabled(&self) -> bool;
    
    /// Check if monitoring is enabled
    fn monitoring_enabled(&self) -> bool;
    
    /// Get the audit log path (if specified)
    fn audit_log_path(&self) -> Option<&str> {
        None // Default implementation
    }
}

// ===== Monitoring Configuration =====

/// Configuration for statistics and monitoring
pub trait MonitoringConfiguration {
    /// Get the statistics update interval in seconds
    fn stats_interval(&self) -> u64;
    
    /// Check if system monitoring is enabled
    fn system_monitoring_enabled(&self) -> bool;
    
    /// Check if performance tracking is enabled
    fn performance_tracking_enabled(&self) -> bool;
}

/// Configuration for data export
pub trait ExportConfiguration {
    /// Check if export is enabled
    fn export_enabled(&self) -> bool;
    
    /// Get the export format
    fn export_format(&self) -> &str;
    
    /// Get the export filename pattern
    fn filename_pattern(&self) -> &str;
    
    /// Check if system stats should be included
    fn include_system_stats(&self) -> bool;
    
    /// Get the export interval in seconds (if periodic)
    fn export_interval(&self) -> Option<u64> {
        None // Default implementation
    }
}

// ===== Composite Traits =====

/// Full read-only configuration access
pub trait ReadConfiguration: 
    TargetConfiguration + 
    ProtocolConfiguration + 
    PerformanceConfiguration + 
    PacketConfiguration + 
    SafetyConfiguration + 
    SecurityConfiguration + 
    MonitoringConfiguration + 
    ExportConfiguration 
{
    /// Get a description of this configuration
    fn description(&self) -> String {
        format!(
            "Config: {}:{:?} @ {} PPS with {} threads",
            self.target_ip(),
            self.target_ports(),
            self.packet_rate(),
            self.thread_count()
        )
    }
}

/// Minimal configuration for basic operations
pub trait BasicConfiguration: 
    TargetConfiguration + 
    PerformanceConfiguration 
{}

/// Configuration for packet generation
pub trait PacketGenerationConfiguration: 
    TargetConfiguration + 
    ProtocolConfiguration + 
    PacketConfiguration 
{}

/// Configuration for monitoring and statistics
pub trait ObservabilityConfiguration: 
    MonitoringConfiguration + 
    ExportConfiguration 
{}

// ===== Configuration Views =====

/// A view that provides read-only access to target configuration
pub struct TargetView<'a, T: TargetConfiguration> {
    config: &'a T,
}

impl<'a, T: TargetConfiguration> TargetView<'a, T> {
    /// Create a new target view
    pub fn new(config: &'a T) -> Self {
        Self { config }
    }
}

impl<'a, T: TargetConfiguration> TargetConfiguration for TargetView<'a, T> {
    fn target_ip(&self) -> &str {
        self.config.target_ip()
    }
    
    fn target_ports(&self) -> &[u16] {
        self.config.target_ports()
    }
    
    fn network_interface(&self) -> Option<&str> {
        self.config.network_interface()
    }
}

/// A view that provides read-only access to safety configuration
pub struct SafetyView<'a, T: SafetyConfiguration> {
    config: &'a T,
}

impl<'a, T: SafetyConfiguration> SafetyView<'a, T> {
    /// Create a new safety view
    pub fn new(config: &'a T) -> Self {
        Self { config }
    }
    
    /// Validate that current settings are within safety limits
    pub fn validate_limits(&self, threads: usize, rate: u64) -> Result<(), String> {
        if threads > self.config.max_threads() {
            return Err(format!(
                "Thread count {} exceeds maximum {}",
                threads,
                self.config.max_threads()
            ));
        }
        
        if rate > self.config.max_packet_rate() {
            return Err(format!(
                "Packet rate {} exceeds maximum {}",
                rate,
                self.config.max_packet_rate()
            ));
        }
        
        Ok(())
    }
}

impl<'a, T: SafetyConfiguration> SafetyConfiguration for SafetyView<'a, T> {
    fn max_threads(&self) -> usize {
        self.config.max_threads()
    }
    
    fn max_packet_rate(&self) -> u64 {
        self.config.max_packet_rate()
    }
    
    fn require_private_ranges(&self) -> bool {
        self.config.require_private_ranges()
    }
    
    fn is_dry_run(&self) -> bool {
        self.config.is_dry_run()
    }
    
    fn is_perfect_simulation(&self) -> bool {
        self.config.is_perfect_simulation()
    }
}

// ===== Helper Functions =====

/// Check if an IP address is in a private range
pub fn is_private_ip(ip: &str) -> bool {
    if let Ok(addr) = ip.parse::<IpAddr>() {
        match addr {
            IpAddr::V4(ipv4) => ipv4.is_private(),
            IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unique_local(),
        }
    } else {
        false
    }
}

/// Validate configuration against safety requirements
pub fn validate_safety<T: SafetyConfiguration + TargetConfiguration>(config: &T) -> Result<(), String> {
    if config.require_private_ranges() && !is_private_ip(config.target_ip()) {
        return Err(format!(
            "Target IP {} is not in a private range",
            config.target_ip()
        ));
    }
    
    Ok(())
}