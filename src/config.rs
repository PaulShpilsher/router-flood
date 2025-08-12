use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

pub const CONFIG_FILE: &str = "router_flood_config.yaml";
pub const MAX_THREADS: usize = 100;
pub const MAX_PACKET_RATE: u64 = 10000;
pub const MIN_PAYLOAD_SIZE: usize = 20;
pub const MAX_PAYLOAD_SIZE: usize = 1400;

/// Configuration structures for YAML config file support
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Both,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BurstPattern {
    Sustained { rate: u64 },
    Bursts { burst_size: usize, burst_interval_ms: u64 },
    Ramp { start_rate: u64, end_rate: u64, ramp_duration: u64 },
}

/// Load configuration from YAML file
pub fn load_config(config_path: Option<&str>) -> Result<Config, String> {
    let config_file = config_path.unwrap_or(CONFIG_FILE);

    if !Path::new(config_file).exists() {
        info!("Config file {} not found, using defaults", config_file);
        return Ok(get_default_config());
    }

    let config_str = std::fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    serde_yaml::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config file: {}", e))
}

pub fn get_default_config() -> Config {
    Config {
        target: TargetConfig {
            ip: "192.168.1.1".to_string(),
            ports: vec![80],
            protocol_mix: ProtocolMix {
                udp_ratio: 0.6,
                tcp_syn_ratio: 0.25,
                tcp_ack_ratio: 0.05,
                icmp_ratio: 0.05,
                ipv6_ratio: 0.03,
                arp_ratio: 0.02,
            },
            interface: None,
        },
        attack: AttackConfig {
            threads: 4,
            packet_rate: 100,
            duration: None,
            packet_size_range: (MIN_PAYLOAD_SIZE, MAX_PAYLOAD_SIZE),
            burst_pattern: BurstPattern::Sustained { rate: 100 },
            randomize_timing: true,
        },
        safety: SafetyConfig {
            max_threads: MAX_THREADS,
            max_packet_rate: MAX_PACKET_RATE,
            require_private_ranges: true,
            enable_monitoring: true,
            audit_logging: true,
            dry_run: false,
        },
        monitoring: MonitoringConfig {
            stats_interval: 5,
            system_monitoring: true,
            export_interval: Some(60),
            performance_tracking: true,
        },
        export: ExportConfig {
            enabled: false,
            format: ExportFormat::Json,
            filename_pattern: "router_flood".to_string(),
            include_system_stats: true,
        },
    }
}
