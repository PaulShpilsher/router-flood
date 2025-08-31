//! Configuration management for router-flood
//!
//! This module provides configuration structures and validation.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::constants::{
    defaults, MAX_THREADS, MAX_PACKET_RATE, MIN_PAYLOAD_SIZE, MAX_PAYLOAD_SIZE,
    DEFAULT_CONFIG_FILE, DEFAULT_EXPORT_INTERVAL,
};
use crate::error::{ConfigError, Result};

/// Main configuration structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub target: Target,
    pub attack: LoadConfig,
    pub safety: Safety,
    pub monitoring: Monitoring,
    pub export: Export,
}

/// Target configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Target {
    pub ip: String,
    pub ports: Vec<u16>,
    pub protocol_mix: ProtocolMix,
    pub interface: Option<String>,
}

/// Protocol distribution configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtocolMix {
    pub udp_ratio: f64,
    pub tcp_syn_ratio: f64,
    pub tcp_ack_ratio: f64,
    pub icmp_ratio: f64,
    pub custom_ratio: f64,
}

impl Default for ProtocolMix {
    fn default() -> Self {
        Self {
            udp_ratio: 0.25,
            tcp_syn_ratio: 0.25,
            tcp_ack_ratio: 0.25,
            icmp_ratio: 0.25,
            custom_ratio: 0.0,
        }
    }
}

/// Load/attack configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadConfig {
    pub threads: usize,
    pub packet_rate: f64,
    pub payload_size: usize,
    pub duration: Option<u64>,
    pub burst_mode: bool,
    pub burst_pattern: Option<BurstPattern>,
}

/// Burst pattern configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BurstPattern {
    pub burst_duration_ms: u64,
    pub idle_duration_ms: u64,
    pub burst_multiplier: f64,
}

/// Safety configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Safety {
    pub dry_run: bool,
    pub rate_limit: bool,
    pub max_bandwidth_mbps: Option<f64>,
    pub allow_localhost: bool,
    pub require_confirmation: bool,
}

/// Monitoring configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Monitoring {
    pub enabled: bool,
    pub interval_ms: u64,
    pub verbose: bool,
    pub show_stats: bool,
}

/// Export configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Export {
    pub enabled: bool,
    pub format: ExportFormat,
    pub path: String,
    pub interval_seconds: u64,
    pub include_system_stats: bool,
}

/// Export format enumeration
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Yaml,
    Text,
}

impl Default for Config {
    fn default() -> Self {
        default_config()
    }
}

/// Load configuration from file
pub fn load_config(path: Option<&str>) -> Result<Config> {
    let config_path = path.unwrap_or(DEFAULT_CONFIG_FILE);
    
    if !Path::new(config_path).exists() {
        info!("Config file not found at {}, using defaults", config_path);
        return Ok(default_config());
    }
    
    let contents = std::fs::read_to_string(config_path)
        .map_err(|e| ConfigError::new(format!("Failed to read config file: {}", e)))?;
    
    let config: Config = serde_yaml::from_str(&contents)
        .map_err(|e| ConfigError::new(format!("Failed to parse config: {}", e)))?;
    
    validate_config(&config)?;
    
    Ok(config)
}

/// Create default configuration
pub fn default_config() -> Config {
    Config {
        target: Target {
            ip: "192.168.1.1".to_string(),
            ports: vec![80, 443],
            protocol_mix: ProtocolMix::default(),
            interface: None,
        },
        attack: LoadConfig {
            threads: defaults::DEFAULT_THREADS,
            packet_rate: defaults::DEFAULT_PACKET_RATE,
            payload_size: defaults::DEFAULT_PAYLOAD_SIZE,
            duration: Some(defaults::DEFAULT_DURATION_SECONDS),
            burst_mode: false,
            burst_pattern: None,
        },
        safety: Safety {
            dry_run: false,
            rate_limit: true,
            max_bandwidth_mbps: Some(defaults::DEFAULT_MAX_BANDWIDTH_MBPS),
            allow_localhost: false,
            require_confirmation: true,
        },
        monitoring: Monitoring {
            enabled: true,
            interval_ms: defaults::DEFAULT_STATS_INTERVAL_MS,
            verbose: false,
            show_stats: true,
        },
        export: Export {
            enabled: false,
            format: ExportFormat::Json,
            path: "./stats".to_string(),
            interval_seconds: DEFAULT_EXPORT_INTERVAL,
            include_system_stats: false,
        },
    }
}

/// Validate configuration
pub fn validate_config(config: &Config) -> Result<()> {
    // Validate threads
    if config.attack.threads == 0 || config.attack.threads > MAX_THREADS {
        return Err(ConfigError::new(
            format!("Thread count must be between 1 and {}", MAX_THREADS)
        ).into());
    }
    
    // Validate packet rate
    if config.attack.packet_rate <= 0.0 || config.attack.packet_rate > MAX_PACKET_RATE as f64 {
        return Err(ConfigError::new(
            format!("Packet rate must be between 0 and {}", MAX_PACKET_RATE)
        ).into());
    }
    
    // Validate payload size
    if config.attack.payload_size < MIN_PAYLOAD_SIZE || config.attack.payload_size > MAX_PAYLOAD_SIZE {
        return Err(ConfigError::new(
            format!("Payload size must be between {} and {}", MIN_PAYLOAD_SIZE, MAX_PAYLOAD_SIZE)
        ).into());
    }
    
    // Validate protocol mix
    let total_ratio = config.target.protocol_mix.udp_ratio
        + config.target.protocol_mix.tcp_syn_ratio
        + config.target.protocol_mix.tcp_ack_ratio
        + config.target.protocol_mix.icmp_ratio
        + config.target.protocol_mix.custom_ratio;
    
    if (total_ratio - 1.0).abs() > 0.01 {
        return Err(ConfigError::new(
            format!("Protocol mix ratios must sum to 1.0, got {}", total_ratio)
        ).into());
    }
    
    // Validate ports
    if config.target.ports.is_empty() {
        return Err(ConfigError::new("At least one target port must be specified").into());
    }
    
    Ok(())
}

/// Configuration builder for fluent API
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: default_config(),
        }
    }
    
    pub fn target_ip(mut self, ip: impl Into<String>) -> Self {
        self.config.target.ip = ip.into();
        self
    }
    
    pub fn target_ports(mut self, ports: Vec<u16>) -> Self {
        self.config.target.ports = ports;
        self
    }
    
    pub fn threads(mut self, threads: usize) -> Self {
        self.config.attack.threads = threads;
        self
    }
    
    pub fn packet_rate(mut self, rate: f64) -> Self {
        self.config.attack.packet_rate = rate;
        self
    }
    
    pub fn payload_size(mut self, size: usize) -> Self {
        self.config.attack.payload_size = size;
        self
    }
    
    pub fn duration(mut self, seconds: u64) -> Self {
        self.config.attack.duration = Some(seconds);
        self
    }
    
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.config.safety.dry_run = enabled;
        self
    }
    
    pub fn build(self) -> Result<Config> {
        validate_config(&self.config)?;
        Ok(self.config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}