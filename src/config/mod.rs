//! Configuration management with builder pattern and validation
//!
//! This module provides enhanced configuration management with centralized
//! validation and a fluent builder API.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::constants::{
    defaults, MAX_THREADS, MAX_PACKET_RATE, MIN_PAYLOAD_SIZE, MAX_PAYLOAD_SIZE,
    DEFAULT_CONFIG_FILE, DEFAULT_EXPORT_INTERVAL,
};
use crate::error::{ConfigError, Result};

// Re-export submodules that exist
pub mod builder;
pub mod simplified;
pub mod traits;
pub mod validation;

// Re-export from submodules
pub use builder::ConfigBuilder;
// Re-export from submodules (commented out until modules are properly implemented)
// pub use simplified::{SimpleConfig, IntensityLevel};
// pub use traits::{TargetView, SafetyView, is_private_ip, validate_safety};

/// Main configuration structure for YAML config file support
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub target: TargetConfig,
    pub attack: AttackConfig,
    pub safety: SafetyConfig,
    pub monitoring: MonitoringConfig,
    pub export: ExportConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetConfig {
    pub ip: String,
    pub ports: Vec<u16>,
    pub protocol_mix: ProtocolMix,
    pub interface: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtocolMix {
    pub udp_ratio: f64,
    pub tcp_syn_ratio: f64,
    pub tcp_ack_ratio: f64,
    pub icmp_ratio: f64,
    pub ipv6_ratio: f64,
    pub arp_ratio: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttackConfig {
    pub threads: usize,
    pub packet_rate: u64,
    pub duration: Option<u64>,
    pub packet_size_range: (usize, usize),
    pub burst_pattern: BurstPattern,
    pub randomize_timing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetyConfig {
    pub max_threads: usize,
    pub max_packet_rate: u64,
    pub require_private_ranges: bool,
    pub enable_monitoring: bool,
    pub audit_logging: bool,
    pub dry_run: bool,
    pub perfect_simulation: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub stats_interval: u64,
    pub system_monitoring: bool,
    pub export_interval: Option<u64>,
    pub performance_tracking: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportConfig {
    pub enabled: bool,
    pub format: ExportFormat,
    pub filename_pattern: String,
    pub include_system_stats: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Both,
}

impl std::str::FromStr for ExportFormat {
    type Err = ConfigError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "both" => Ok(ExportFormat::Both),
            _ => Err(ConfigError::InvalidValue {
                field: "format".to_string(),
                value: s.to_string(),
                reason: "Must be 'json', 'csv', or 'both'".to_string(),
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BurstPattern {
    Sustained { rate: u64 },
    Bursts { burst_size: usize, burst_interval_ms: u64 },
    Ramp { start_rate: u64, end_rate: u64, ramp_duration: u64 },
}

/// Load configuration from YAML file
pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    let config_file = config_path.unwrap_or(DEFAULT_CONFIG_FILE);

    if !Path::new(config_file).exists() {
        info!("Config file {} not found, using defaults", config_file);
        return Ok(get_default_config());
    }

    let config_str = std::fs::read_to_string(config_file)
        .map_err(|e| ConfigError::FileNotFound(format!("Failed to read config file: {}", e)))?;

    serde_yaml::from_str(&config_str)
        .map_err(|e| ConfigError::ParseError(format!("Failed to parse config file: {}", e)))
        .map_err(Into::into)
}

pub fn get_default_config() -> Config {
    Config {
        target: TargetConfig {
            ip: defaults::TARGET_IP.to_string(),
            ports: vec![defaults::TARGET_PORT],
            protocol_mix: ProtocolMix {
                udp_ratio: defaults::UDP_RATIO,
                tcp_syn_ratio: defaults::TCP_SYN_RATIO,
                tcp_ack_ratio: defaults::TCP_ACK_RATIO,
                icmp_ratio: defaults::ICMP_RATIO,
                ipv6_ratio: defaults::IPV6_RATIO,
                arp_ratio: defaults::ARP_RATIO,
            },
            interface: None,
        },
        attack: AttackConfig {
            threads: defaults::THREAD_COUNT,
            packet_rate: defaults::PACKET_RATE,
            duration: None,
            packet_size_range: (MIN_PAYLOAD_SIZE, MAX_PAYLOAD_SIZE),
            burst_pattern: BurstPattern::Sustained { rate: defaults::PACKET_RATE },
            randomize_timing: true,
        },
        safety: SafetyConfig {
            max_threads: MAX_THREADS,
            max_packet_rate: MAX_PACKET_RATE,
            require_private_ranges: true,
            enable_monitoring: true,
            audit_logging: true,
            dry_run: false,
            perfect_simulation: false,
        },
        monitoring: MonitoringConfig {
            stats_interval: defaults::STATS_INTERVAL,
            system_monitoring: true,
            export_interval: Some(DEFAULT_EXPORT_INTERVAL),
            performance_tracking: true,
        },
        export: ExportConfig {
            enabled: false,
            format: ExportFormat::Json,
            filename_pattern: defaults::FILENAME_PATTERN.to_string(),
            include_system_stats: true,
        },
    }
}

/// Placeholder for ConfigTemplates (referenced by CLI)
pub struct ConfigTemplates;

impl ConfigTemplates {
    pub fn list_templates() -> Vec<&'static str> {
        vec!["basic", "web_server", "dns_server", "high_performance"]
    }

    pub fn get_template(name: &str) -> Option<Config> {
        match name {
            "basic" => Some(Self::basic_template()),
            "web_server" => Some(Self::web_server_template()),
            "dns_server" => Some(Self::dns_server_template()),
            "high_performance" => Some(Self::high_performance_template()),
            _ => None,
        }
    }

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

    fn web_server_template() -> Config {
        let mut config = get_default_config();
        config.target.ip = "192.168.1.100".to_string();
        config.target.ports = vec![80, 443, 8080, 8443];
        config.attack.threads = 4;
        config.attack.packet_rate = 200;
        config.attack.duration = Some(60);
        config.export.enabled = true;
        config.export.format = ExportFormat::Both;
        config
    }

    fn dns_server_template() -> Config {
        let mut config = get_default_config();
        config.target.ip = "192.168.1.53".to_string();
        config.target.ports = vec![53];
        config.attack.threads = 6;
        config.attack.packet_rate = 500;
        config.attack.duration = Some(120);
        config.attack.packet_size_range = (64, 512);
        config
    }

    fn high_performance_template() -> Config {
        let mut config = get_default_config();
        config.target.ip = "10.0.0.1".to_string();
        config.target.ports = vec![80, 443, 22, 53, 21, 25];
        config.attack.threads = 8;
        config.attack.packet_rate = 1000;
        config.attack.duration = Some(300);
        config.attack.packet_size_range = (64, 1400);
        config.attack.randomize_timing = false;
        config.export.enabled = true;
        config.export.format = ExportFormat::Json;
        config.monitoring.stats_interval = 1;
        config
    }

    /// Generate template as YAML string
    pub fn template_to_yaml(template: &Config) -> Result<String> {
        serde_yaml::to_string(template)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize template: {}", e)).into())
    }
}

/// Placeholder for ConfigSchema (referenced by CLI)
pub struct ConfigSchema;

impl ConfigSchema {
    pub fn validate(config: &Config) -> Result<()> {
        // Basic validation
        if config.target.ip.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "target.ip".to_string(),
                value: "empty".to_string(),
                reason: "IP address cannot be empty".to_string(),
            }.into());
        }

        if config.target.ports.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "target.ports".to_string(),
                value: "empty".to_string(),
                reason: "At least one port must be specified".to_string(),
            }.into());
        }

        if config.attack.threads == 0 {
            return Err(ConfigError::InvalidValue {
                field: "attack.threads".to_string(),
                value: config.attack.threads.to_string(),
                reason: "Thread count must be greater than 0".to_string(),
            }.into());
        }

        if config.attack.packet_rate == 0 {
            return Err(ConfigError::InvalidValue {
                field: "attack.packet_rate".to_string(),
                value: config.attack.packet_rate.to_string(),
                reason: "Packet rate must be greater than 0".to_string(),
            }.into());
        }

        Ok(())
    }
}