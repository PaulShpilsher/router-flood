//! Application configuration with focused, composable structs
//!
//! This module implements the refactored configuration system that addresses
//! the Single Responsibility Principle violation in the original Config struct.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::constants::{
    defaults, MAX_THREADS, MAX_PACKET_RATE, MIN_PAYLOAD_SIZE, MAX_PAYLOAD_SIZE,
    DEFAULT_CONFIG_FILE, DEFAULT_EXPORT_INTERVAL,
};
use crate::error::{ConfigError, Result};

/// Main application configuration composed of focused settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub target: TargetSettings,
    pub execution: ExecutionSettings,
    pub safety: SafetySettings,
    pub observability: ObservabilitySettings,
}

/// Target-specific configuration settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetSettings {
    pub ip: String,
    pub ports: Vec<u16>,
    pub protocol_mix: ProtocolMix,
    pub interface: Option<String>,
}

/// Execution and performance settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionSettings {
    pub threads: usize,
    pub packet_rate: u64,
    pub duration: Option<u64>,
    pub packet_size_range: (usize, usize),
    pub burst_pattern: BurstPattern,
    pub randomize_timing: bool,
}

/// Safety and security settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetySettings {
    pub max_threads: usize,
    pub max_packet_rate: u64,
    pub require_private_ranges: bool,
    pub audit_logging: bool,
    pub dry_run: bool,
    pub perfect_simulation: bool,
}

/// Observability settings (monitoring and export)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObservabilitySettings {
    pub monitoring: MonitoringSettings,
    pub export: ExportSettings,
}

/// Monitoring configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringSettings {
    pub stats_interval: u64,
    pub system_monitoring: bool,
    pub performance_tracking: bool,
}

/// Export configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportSettings {
    pub enabled: bool,
    pub format: ExportFormat,
    pub filename_pattern: String,
    pub include_system_stats: bool,
    pub export_interval: Option<u64>,
}

/// Protocol mix configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtocolMix {
    pub udp_ratio: f64,
    pub tcp_syn_ratio: f64,
    pub tcp_ack_ratio: f64,
    pub icmp_ratio: f64,
    pub ipv6_ratio: f64,
    pub arp_ratio: f64,
}

/// Export format options
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Both,
}

/// Burst pattern configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BurstPattern {
    Sustained { rate: u64 },
    Bursts { burst_size: usize, burst_interval_ms: u64 },
    Ramp { start_rate: u64, end_rate: u64, ramp_duration: u64 },
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

impl ApplicationConfig {
    /// Create a new application configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from YAML file
    pub fn load_from_file(config_path: Option<&str>) -> Result<Self> {
        let config_file = config_path.unwrap_or(DEFAULT_CONFIG_FILE);

        if !Path::new(config_file).exists() {
            info!("Config file {} not found, using defaults", config_file);
            return Ok(Self::default());
        }

        let config_str = std::fs::read_to_string(config_file)
            .map_err(|e| ConfigError::FileNotFound(format!("Failed to read config file: {}", e)))?;

        serde_yaml::from_str(&config_str)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse config file: {}", e)))
            .map_err(Into::into)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        self.target.validate()?;
        self.execution.validate()?;
        self.safety.validate()?;
        self.observability.validate()?;
        Ok(())
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            target: TargetSettings::default(),
            execution: ExecutionSettings::default(),
            safety: SafetySettings::default(),
            observability: ObservabilitySettings::default(),
        }
    }
}

impl Default for TargetSettings {
    fn default() -> Self {
        Self {
            ip: defaults::TARGET_IP.to_string(),
            ports: vec![defaults::TARGET_PORT],
            protocol_mix: ProtocolMix::default(),
            interface: None,
        }
    }
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            threads: defaults::THREAD_COUNT,
            packet_rate: defaults::PACKET_RATE,
            duration: None,
            packet_size_range: (MIN_PAYLOAD_SIZE, MAX_PAYLOAD_SIZE),
            burst_pattern: BurstPattern::Sustained { rate: defaults::PACKET_RATE },
            randomize_timing: true,
        }
    }
}

impl Default for SafetySettings {
    fn default() -> Self {
        Self {
            max_threads: MAX_THREADS,
            max_packet_rate: MAX_PACKET_RATE,
            require_private_ranges: true,
            audit_logging: true,
            dry_run: false,
            perfect_simulation: false,
        }
    }
}

impl Default for ObservabilitySettings {
    fn default() -> Self {
        Self {
            monitoring: MonitoringSettings::default(),
            export: ExportSettings::default(),
        }
    }
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        Self {
            stats_interval: defaults::STATS_INTERVAL,
            system_monitoring: true,
            performance_tracking: true,
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            format: ExportFormat::Json,
            filename_pattern: defaults::FILENAME_PATTERN.to_string(),
            include_system_stats: true,
            export_interval: Some(DEFAULT_EXPORT_INTERVAL),
        }
    }
}

impl Default for ProtocolMix {
    fn default() -> Self {
        Self {
            udp_ratio: defaults::UDP_RATIO,
            tcp_syn_ratio: defaults::TCP_SYN_RATIO,
            tcp_ack_ratio: defaults::TCP_ACK_RATIO,
            icmp_ratio: defaults::ICMP_RATIO,
            ipv6_ratio: defaults::IPV6_RATIO,
            arp_ratio: defaults::ARP_RATIO,
        }
    }
}

// Validation implementations for each settings struct
impl TargetSettings {
    fn validate(&self) -> Result<()> {
        if self.ports.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "ports".to_string(),
                value: "empty".to_string(),
                reason: "At least one port must be specified".to_string(),
            }.into());
        }

        self.protocol_mix.validate()?;
        Ok(())
    }
}

impl ExecutionSettings {
    fn validate(&self) -> Result<()> {
        if self.threads == 0 {
            return Err(ConfigError::InvalidValue {
                field: "threads".to_string(),
                value: self.threads.to_string(),
                reason: "Thread count must be greater than 0".to_string(),
            }.into());
        }

        if self.packet_rate == 0 {
            return Err(ConfigError::InvalidValue {
                field: "packet_rate".to_string(),
                value: self.packet_rate.to_string(),
                reason: "Packet rate must be greater than 0".to_string(),
            }.into());
        }

        self.burst_pattern.validate()?;
        Ok(())
    }
}

impl SafetySettings {
    fn validate(&self) -> Result<()> {
        if self.max_threads == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_threads".to_string(),
                value: self.max_threads.to_string(),
                reason: "Maximum threads must be greater than 0".to_string(),
            }.into());
        }

        if self.max_packet_rate == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_packet_rate".to_string(),
                value: self.max_packet_rate.to_string(),
                reason: "Maximum packet rate must be greater than 0".to_string(),
            }.into());
        }

        Ok(())
    }
}

impl ObservabilitySettings {
    fn validate(&self) -> Result<()> {
        self.monitoring.validate()?;
        self.export.validate()?;
        Ok(())
    }
}

impl MonitoringSettings {
    fn validate(&self) -> Result<()> {
        if self.stats_interval == 0 {
            return Err(ConfigError::InvalidValue {
                field: "stats_interval".to_string(),
                value: self.stats_interval.to_string(),
                reason: "Stats interval must be greater than 0".to_string(),
            }.into());
        }
        Ok(())
    }
}

impl ExportSettings {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            if self.filename_pattern.is_empty() {
                return Err(ConfigError::InvalidValue {
                    field: "filename_pattern".to_string(),
                    value: "empty".to_string(),
                    reason: "Filename pattern cannot be empty when export is enabled".to_string(),
                }.into());
            }

            if let Some(interval) = self.export_interval {
                if interval == 0 {
                    return Err(ConfigError::InvalidValue {
                        field: "export_interval".to_string(),
                        value: interval.to_string(),
                        reason: "Export interval must be greater than 0".to_string(),
                    }.into());
                }
            }
        }
        Ok(())
    }
}

impl ProtocolMix {
    fn validate(&self) -> Result<()> {
        let total = self.udp_ratio + self.tcp_syn_ratio + self.tcp_ack_ratio 
                  + self.icmp_ratio + self.ipv6_ratio + self.arp_ratio;

        if (total - 1.0).abs() > 0.001 {
            return Err(ConfigError::InvalidValue {
                field: "protocol_ratios".to_string(),
                value: format!("{:.3}", total),
                reason: "Protocol ratios must sum to 1.0".to_string(),
            }.into());
        }

        let ratios = [self.udp_ratio, self.tcp_syn_ratio, self.tcp_ack_ratio,
                     self.icmp_ratio, self.ipv6_ratio, self.arp_ratio];
        
        for (i, &ratio) in ratios.iter().enumerate() {
            if !(0.0..=1.0).contains(&ratio) {
                let field_names = ["udp_ratio", "tcp_syn_ratio", "tcp_ack_ratio",
                                 "icmp_ratio", "ipv6_ratio", "arp_ratio"];
                return Err(ConfigError::InvalidValue {
                    field: field_names[i].to_string(),
                    value: ratio.to_string(),
                    reason: "Ratio must be between 0.0 and 1.0".to_string(),
                }.into());
            }
        }

        Ok(())
    }
}

impl BurstPattern {
    fn validate(&self) -> Result<()> {
        match self {
            BurstPattern::Sustained { rate } => {
                if *rate == 0 {
                    return Err(ConfigError::InvalidValue {
                        field: "sustained_rate".to_string(),
                        value: rate.to_string(),
                        reason: "Sustained rate must be greater than 0".to_string(),
                    }.into());
                }
            }
            BurstPattern::Bursts { burst_size, burst_interval_ms } => {
                if *burst_size == 0 {
                    return Err(ConfigError::InvalidValue {
                        field: "burst_size".to_string(),
                        value: burst_size.to_string(),
                        reason: "Burst size must be greater than 0".to_string(),
                    }.into());
                }
                if *burst_interval_ms == 0 {
                    return Err(ConfigError::InvalidValue {
                        field: "burst_interval_ms".to_string(),
                        value: burst_interval_ms.to_string(),
                        reason: "Burst interval must be greater than 0".to_string(),
                    }.into());
                }
            }
            BurstPattern::Ramp { start_rate, end_rate, ramp_duration } => {
                if *start_rate == 0 || *end_rate == 0 {
                    return Err(ConfigError::InvalidValue {
                        field: "ramp_rates".to_string(),
                        value: format!("{}-{}", start_rate, end_rate),
                        reason: "Ramp start and end rates must be greater than 0".to_string(),
                    }.into());
                }
                if *ramp_duration == 0 {
                    return Err(ConfigError::InvalidValue {
                        field: "ramp_duration".to_string(),
                        value: ramp_duration.to_string(),
                        reason: "Ramp duration must be greater than 0".to_string(),
                    }.into());
                }
            }
        }
        Ok(())
    }
}

// Tests moved to tests/ directory
