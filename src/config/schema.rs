//! Configuration schema validation and templates
//!
//! This module provides JSON schema validation for YAML configurations
//! and pre-built configuration templates for common scenarios.

use crate::error::{ConfigError, Result};
use super::{Config, TargetConfig, AttackConfig, SafetyConfig, MonitoringConfig, ExportConfig, ProtocolMix, ExportFormat, get_default_config};

/// Configuration schema validator
pub struct ConfigSchema;

impl ConfigSchema {
    /// Validate configuration against schema
    pub fn validate(config: &Config) -> Result<()> {
        Self::validate_target(&config.target)?;
        Self::validate_attack(&config.attack)?;
        Self::validate_protocol_mix(&config.target.protocol_mix)?;
        Self::validate_safety(&config.safety)?;
        Self::validate_export(&config.export)?;
        Self::validate_monitoring(&config.monitoring)?;
        Ok(())
    }

    /// Validate target configuration
    fn validate_target(target: &TargetConfig) -> Result<()> {
        // Validate IP format
        if target.ip.parse::<std::net::IpAddr>().is_err() {
            return Err(ConfigError::InvalidValue {
                field: "target.ip".to_string(),
                value: target.ip.clone(),
                reason: "Invalid IP address format".to_string(),
            }.into());
        }

        // Validate ports
        if target.ports.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "target.ports".to_string(),
                value: "empty".to_string(),
                reason: "At least one port must be specified".to_string(),
            }.into());
        }

        for &port in &target.ports {
            if port == 0 {
                return Err(ConfigError::InvalidValue {
                    field: "target.ports".to_string(),
                    value: port.to_string(),
                    reason: "Port 0 is not valid".to_string(),
                }.into());
            }
        }

        Ok(())
    }

    /// Validate attack configuration
    fn validate_attack(attack: &AttackConfig) -> Result<()> {
        if attack.threads == 0 {
            return Err(ConfigError::InvalidValue {
                field: "attack.threads".to_string(),
                value: attack.threads.to_string(),
                reason: "Thread count must be greater than 0".to_string(),
            }.into());
        }

        if attack.packet_rate == 0 {
            return Err(ConfigError::InvalidValue {
                field: "attack.packet_rate".to_string(),
                value: attack.packet_rate.to_string(),
                reason: "Packet rate must be greater than 0".to_string(),
            }.into());
        }

        // Validate packet size range
        let (min_size, max_size) = attack.packet_size_range;
        if min_size >= max_size {
            return Err(ConfigError::InvalidValue {
                field: "attack.packet_size_range".to_string(),
                value: format!("({}, {})", min_size, max_size),
                reason: "Minimum size must be less than maximum size".to_string(),
            }.into());
        }

        if min_size < 20 {
            return Err(ConfigError::InvalidValue {
                field: "attack.packet_size_range".to_string(),
                value: min_size.to_string(),
                reason: "Minimum packet size must be at least 20 bytes".to_string(),
            }.into());
        }

        if max_size > 1500 {
            return Err(ConfigError::InvalidValue {
                field: "attack.packet_size_range".to_string(),
                value: max_size.to_string(),
                reason: "Maximum packet size should not exceed 1500 bytes (standard MTU)".to_string(),
            }.into());
        }

        Ok(())
    }

    /// Validate protocol mix ratios
    fn validate_protocol_mix(mix: &ProtocolMix) -> Result<()> {
        let total = mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + 
                   mix.icmp_ratio + mix.ipv6_ratio + mix.arp_ratio;
        
        if (total - 1.0).abs() > 0.001 {
            return Err(ConfigError::InvalidValue {
                field: "target.protocol_mix".to_string(),
                value: format!("{:.3}", total),
                reason: "Protocol ratios must sum to 1.0".to_string(),
            }.into());
        }

        // Check individual ratios are non-negative
        let ratios = [
            ("udp_ratio", mix.udp_ratio),
            ("tcp_syn_ratio", mix.tcp_syn_ratio),
            ("tcp_ack_ratio", mix.tcp_ack_ratio),
            ("icmp_ratio", mix.icmp_ratio),
            ("ipv6_ratio", mix.ipv6_ratio),
            ("arp_ratio", mix.arp_ratio),
        ];

        for (name, ratio) in ratios {
            if ratio < 0.0 {
                return Err(ConfigError::InvalidValue {
                    field: format!("target.protocol_mix.{}", name),
                    value: ratio.to_string(),
                    reason: "Protocol ratios must be non-negative".to_string(),
                }.into());
            }
        }

        Ok(())
    }

    /// Validate safety configuration
    fn validate_safety(safety: &SafetyConfig) -> Result<()> {
        if safety.max_threads == 0 {
            return Err(ConfigError::InvalidValue {
                field: "safety.max_threads".to_string(),
                value: safety.max_threads.to_string(),
                reason: "Maximum threads must be greater than 0".to_string(),
            }.into());
        }

        if safety.max_packet_rate == 0 {
            return Err(ConfigError::InvalidValue {
                field: "safety.max_packet_rate".to_string(),
                value: safety.max_packet_rate.to_string(),
                reason: "Maximum packet rate must be greater than 0".to_string(),
            }.into());
        }

        Ok(())
    }

    /// Validate export configuration
    fn validate_export(export: &ExportConfig) -> Result<()> {
        if export.filename_pattern.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "export.filename_pattern".to_string(),
                value: "empty".to_string(),
                reason: "Filename pattern cannot be empty".to_string(),
            }.into());
        }

        Ok(())
    }

    /// Validate monitoring configuration
    fn validate_monitoring(monitoring: &MonitoringConfig) -> Result<()> {
        if monitoring.stats_interval == 0 {
            return Err(ConfigError::InvalidValue {
                field: "monitoring.stats_interval".to_string(),
                value: monitoring.stats_interval.to_string(),
                reason: "Stats interval must be greater than 0".to_string(),
            }.into());
        }

        if let Some(export_interval) = monitoring.export_interval {
            if export_interval == 0 {
                return Err(ConfigError::InvalidValue {
                    field: "monitoring.export_interval".to_string(),
                    value: export_interval.to_string(),
                    reason: "Export interval must be greater than 0".to_string(),
                }.into());
            }
        }

        Ok(())
    }
}

/// Configuration templates for common scenarios
pub struct ConfigTemplates;

impl ConfigTemplates {
    /// Get all available template names
    pub fn list_templates() -> Vec<&'static str> {
        vec![
            "basic",
            "web_server", 
            "dns_server",
            "high_performance",
        ]
    }

    /// Get a configuration template by name
    pub fn get_template(name: &str) -> Option<Config> {
        match name {
            "basic" => Some(Self::basic_template()),
            "web_server" => Some(Self::web_server_template()),
            "dns_server" => Some(Self::dns_server_template()),
            "high_performance" => Some(Self::high_performance_template()),
            _ => None,
        }
    }

    /// Basic testing template
    fn basic_template() -> Config {
        let mut config = get_default_config();
        config.target.ip = "192.168.1.1".to_string();
        config.target.ports = vec![80];
        config.attack.threads = 2;
        config.attack.packet_rate = 50;
        config.attack.duration = Some(30);
        config.safety.dry_run = true;
        config
    }

    /// Web server stress test template
    fn web_server_template() -> Config {
        let mut config = get_default_config();
        config.target.ip = "192.168.1.100".to_string();
        config.target.ports = vec![80, 443, 8080, 8443];
        config.attack.threads = 4;
        config.attack.packet_rate = 200;
        config.attack.duration = Some(60);
        
        // Focus on TCP traffic for web servers
        config.target.protocol_mix = ProtocolMix {
            udp_ratio: 0.1,
            tcp_syn_ratio: 0.7,
            tcp_ack_ratio: 0.15,
            icmp_ratio: 0.03,
            ipv6_ratio: 0.02,
            arp_ratio: 0.0,
        };
        
        config.export.enabled = true;
        config.export.format = ExportFormat::Both;
        config
    }

    /// DNS server stress test template
    fn dns_server_template() -> Config {
        let mut config = get_default_config();
        config.target.ip = "192.168.1.53".to_string();
        config.target.ports = vec![53];
        config.attack.threads = 6;
        config.attack.packet_rate = 500;
        config.attack.duration = Some(120);
        
        // Focus on UDP traffic for DNS
        config.target.protocol_mix = ProtocolMix {
            udp_ratio: 0.9,
            tcp_syn_ratio: 0.05,
            tcp_ack_ratio: 0.02,
            icmp_ratio: 0.02,
            ipv6_ratio: 0.01,
            arp_ratio: 0.0,
        };
        
        config.attack.packet_size_range = (64, 512); // Typical DNS packet sizes
        config
    }

    /// High performance testing template
    fn high_performance_template() -> Config {
        let mut config = get_default_config();
        config.target.ip = "10.0.0.1".to_string();
        config.target.ports = vec![80, 443, 22, 53, 21, 25];
        config.attack.threads = 8;
        config.attack.packet_rate = 1000;
        config.attack.duration = Some(300); // 5 minutes
        
        config.attack.packet_size_range = (64, 1400);
        config.attack.randomize_timing = false; // Consistent timing for max performance
        
        config.export.enabled = true;
        config.export.format = ExportFormat::Json;
        config.monitoring.stats_interval = 1; // More frequent updates
        config
    }

    /// Generate template as YAML string
    pub fn template_to_yaml(template: &Config) -> Result<String> {
        serde_yaml::to_string(template)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize template: {}", e)).into())
    }
}

