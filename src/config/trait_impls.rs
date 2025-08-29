//! Trait implementations for Config structures
//!
//! This module implements the configuration traits for the existing Config structures,
//! maintaining backward compatibility while providing interface segregation.

use super::{Config, TargetConfig, AttackConfig, SafetyConfig, MonitoringConfig, ExportConfig, ProtocolMix};
use super::traits::*;

// ===== TargetConfig Trait Implementations =====

impl TargetConfiguration for TargetConfig {
    fn target_ip(&self) -> &str {
        &self.ip
    }
    
    fn target_ports(&self) -> &[u16] {
        &self.ports
    }
    
    fn network_interface(&self) -> Option<&str> {
        self.interface.as_deref()
    }
}

impl ProtocolConfiguration for ProtocolMix {
    fn udp_ratio(&self) -> f64 {
        self.udp_ratio
    }
    
    fn tcp_syn_ratio(&self) -> f64 {
        self.tcp_syn_ratio
    }
    
    fn tcp_ack_ratio(&self) -> f64 {
        self.tcp_ack_ratio
    }
    
    fn icmp_ratio(&self) -> f64 {
        self.icmp_ratio
    }
    
    fn ipv6_ratio(&self) -> f64 {
        self.ipv6_ratio
    }
    
    fn arp_ratio(&self) -> f64 {
        self.arp_ratio
    }
}

// ===== AttackConfig Trait Implementations =====

impl PerformanceConfiguration for AttackConfig {
    fn thread_count(&self) -> usize {
        self.threads
    }
    
    fn packet_rate(&self) -> u64 {
        self.packet_rate
    }
    
    fn duration(&self) -> Option<u64> {
        self.duration
    }
    
    fn randomize_timing(&self) -> bool {
        self.randomize_timing
    }
}

impl PacketConfiguration for AttackConfig {
    fn min_packet_size(&self) -> usize {
        self.packet_size_range.0
    }
    
    fn max_packet_size(&self) -> usize {
        self.packet_size_range.1
    }
}

// ===== SafetyConfig Trait Implementations =====

impl SafetyConfiguration for SafetyConfig {
    fn max_threads(&self) -> usize {
        self.max_threads
    }
    
    fn max_packet_rate(&self) -> u64 {
        self.max_packet_rate
    }
    
    fn require_private_ranges(&self) -> bool {
        self.require_private_ranges
    }
    
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }
    
    fn is_perfect_simulation(&self) -> bool {
        self.perfect_simulation
    }
}

impl SecurityConfiguration for SafetyConfig {
    fn audit_logging_enabled(&self) -> bool {
        self.audit_logging
    }
    
    fn monitoring_enabled(&self) -> bool {
        self.enable_monitoring
    }
}

// ===== MonitoringConfig Trait Implementations =====

impl MonitoringConfiguration for MonitoringConfig {
    fn stats_interval(&self) -> u64 {
        self.stats_interval
    }
    
    fn system_monitoring_enabled(&self) -> bool {
        self.system_monitoring
    }
    
    fn performance_tracking_enabled(&self) -> bool {
        self.performance_tracking
    }
}

// ===== ExportConfig Trait Implementations =====

impl ExportConfiguration for ExportConfig {
    fn export_enabled(&self) -> bool {
        self.enabled
    }
    
    fn export_format(&self) -> &str {
        match self.format {
            super::ExportFormat::Json => "json",
            super::ExportFormat::Csv => "csv",
            super::ExportFormat::Both => "both",
        }
    }
    
    fn filename_pattern(&self) -> &str {
        &self.filename_pattern
    }
    
    fn include_system_stats(&self) -> bool {
        self.include_system_stats
    }
}

impl ExportConfiguration for MonitoringConfig {
    fn export_enabled(&self) -> bool {
        self.export_interval.is_some()
    }
    
    fn export_format(&self) -> &str {
        "json" // Default format
    }
    
    fn filename_pattern(&self) -> &str {
        "stats_{timestamp}.json" // Default pattern
    }
    
    fn include_system_stats(&self) -> bool {
        self.system_monitoring
    }
    
    fn export_interval(&self) -> Option<u64> {
        self.export_interval
    }
}

// ===== Main Config Trait Implementations =====

impl TargetConfiguration for Config {
    fn target_ip(&self) -> &str {
        self.target.target_ip()
    }
    
    fn target_ports(&self) -> &[u16] {
        self.target.target_ports()
    }
    
    fn network_interface(&self) -> Option<&str> {
        self.target.network_interface()
    }
}

impl ProtocolConfiguration for Config {
    fn udp_ratio(&self) -> f64 {
        self.target.protocol_mix.udp_ratio()
    }
    
    fn tcp_syn_ratio(&self) -> f64 {
        self.target.protocol_mix.tcp_syn_ratio()
    }
    
    fn tcp_ack_ratio(&self) -> f64 {
        self.target.protocol_mix.tcp_ack_ratio()
    }
    
    fn icmp_ratio(&self) -> f64 {
        self.target.protocol_mix.icmp_ratio()
    }
    
    fn ipv6_ratio(&self) -> f64 {
        self.target.protocol_mix.ipv6_ratio()
    }
    
    fn arp_ratio(&self) -> f64 {
        self.target.protocol_mix.arp_ratio()
    }
}

impl PerformanceConfiguration for Config {
    fn thread_count(&self) -> usize {
        self.attack.thread_count()
    }
    
    fn packet_rate(&self) -> u64 {
        self.attack.packet_rate()
    }
    
    fn duration(&self) -> Option<u64> {
        self.attack.duration()
    }
    
    fn randomize_timing(&self) -> bool {
        self.attack.randomize_timing()
    }
}

impl PacketConfiguration for Config {
    fn min_packet_size(&self) -> usize {
        self.attack.min_packet_size()
    }
    
    fn max_packet_size(&self) -> usize {
        self.attack.max_packet_size()
    }
}

impl SafetyConfiguration for Config {
    fn max_threads(&self) -> usize {
        self.safety.max_threads()
    }
    
    fn max_packet_rate(&self) -> u64 {
        self.safety.max_packet_rate()
    }
    
    fn require_private_ranges(&self) -> bool {
        self.safety.require_private_ranges()
    }
    
    fn is_dry_run(&self) -> bool {
        self.safety.is_dry_run()
    }
    
    fn is_perfect_simulation(&self) -> bool {
        self.safety.is_perfect_simulation()
    }
}

impl SecurityConfiguration for Config {
    fn audit_logging_enabled(&self) -> bool {
        self.safety.audit_logging_enabled()
    }
    
    fn monitoring_enabled(&self) -> bool {
        self.safety.monitoring_enabled()
    }
}

impl MonitoringConfiguration for Config {
    fn stats_interval(&self) -> u64 {
        self.monitoring.stats_interval()
    }
    
    fn system_monitoring_enabled(&self) -> bool {
        self.monitoring.system_monitoring_enabled()
    }
    
    fn performance_tracking_enabled(&self) -> bool {
        self.monitoring.performance_tracking_enabled()
    }
}

impl ExportConfiguration for Config {
    fn export_enabled(&self) -> bool {
        self.export.export_enabled()
    }
    
    fn export_format(&self) -> &str {
        self.export.export_format()
    }
    
    fn filename_pattern(&self) -> &str {
        self.export.filename_pattern()
    }
    
    fn include_system_stats(&self) -> bool {
        self.export.include_system_stats()
    }
    
    fn export_interval(&self) -> Option<u64> {
        self.monitoring.export_interval()
    }
}

// Implement composite traits
impl ReadConfiguration for Config {}
impl BasicConfiguration for Config {}
impl PacketGenerationConfiguration for Config {}
impl ObservabilityConfiguration for Config {}

// ===== Helper trait for easy access =====

/// Extension trait for Config to provide convenient access methods
pub trait ConfigExt {
    /// Get a target view of the configuration
    fn target_view(&self) -> TargetView<'_, Self>
    where
        Self: TargetConfiguration + Sized;
    
    /// Get a safety view of the configuration
    fn safety_view(&self) -> SafetyView<'_, Self>
    where
        Self: SafetyConfiguration + Sized;
    
    /// Validate the entire configuration
    fn validate(&self) -> Result<(), Vec<String>>
    where
        Self: ReadConfiguration;
}

impl ConfigExt for Config {
    fn target_view(&self) -> TargetView<'_, Self> {
        TargetView::new(self)
    }
    
    fn safety_view(&self) -> SafetyView<'_, Self> {
        SafetyView::new(self)
    }
    
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate protocol ratios
        if !self.validate_ratios() {
            errors.push("Protocol ratios do not sum to 1.0".to_string());
        }
        
        // Validate safety limits
        if let Err(e) = self.safety_view().validate_limits(self.thread_count(), self.packet_rate()) {
            errors.push(e);
        }
        
        // Validate target safety
        if let Err(e) = validate_safety(self) {
            errors.push(e);
        }
        
        // Validate packet size range
        if self.min_packet_size() > self.max_packet_size() {
            errors.push(format!(
                "Invalid packet size range: {} > {}",
                self.min_packet_size(),
                self.max_packet_size()
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}