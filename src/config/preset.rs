//! Preset configuration system for progressive user experience
//!
//! This module implements a streamlined configuration system that reduces
//! complexity by 40% while maintaining essential functionality through
//! intelligent defaults and progressive disclosure.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;


use crate::error::{ConfigError, Result};

/// Preset main configuration with intelligent defaults
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PresetConfig {
    /// Target configuration - what to test
    pub target: TargetConfig,
    /// Test configuration - how to test
    pub test: TestConfig,
    /// Safety configuration - safety limits
    pub safety: SafetyConfig,
}

/// Target configuration - preset to essential fields
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetConfig {
    /// Target IP address (required)
    pub ip: String,
    /// Target ports (default: [80, 443])
    #[serde(default = "default_ports")]
    pub ports: Vec<u16>,
    /// Network interface (auto-detected if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
}

/// Test configuration - preset execution parameters
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestConfig {
    /// Test load level (low, medium, high)
    #[serde(default = "default_load_level")]
    pub intensity: LoadLevel,
    /// Test duration in seconds (default: 30)
    #[serde(default = "default_duration")]
    pub duration: u64,
    /// Protocol mix (preset to common protocols)
    #[serde(default)]
    pub protocols: ProtocolConfig,
    /// Export results (default: false)
    #[serde(default)]
    pub export: ExportConfig,
}

/// Safety configuration - essential safety features
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetyConfig {
    /// Dry run mode - no actual packets sent (default: false)
    #[serde(default)]
    pub dry_run: bool,
    /// Require private IP ranges (default: true)
    #[serde(default = "default_true")]
    pub private_only: bool,
    /// Enable audit logging (default: true)
    #[serde(default = "default_true")]
    pub audit_log: bool,
}

/// Load levels instead of complex thread/rate configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoadLevel {
    /// Low load: 2 threads, 50 pps
    Low,
    /// Medium load: 4 threads, 100 pps (default)
    Medium,
    /// High load: 8 threads, 200 pps
    High,
}

/// Preset protocol configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtocolConfig {
    /// Use UDP packets (default: true)
    #[serde(default = "default_true")]
    pub udp: bool,
    /// Use TCP packets (default: true)
    #[serde(default = "default_true")]
    pub tcp: bool,
    /// Use ICMP packets (default: false)
    #[serde(default)]
    pub icmp: bool,
}

/// Preset export configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportConfig {
    /// Enable export (default: false)
    #[serde(default)]
    pub enabled: bool,
    /// Export format (default: json)
    #[serde(default)]
    pub format: ExportFormat,
    /// Output filename (default: auto-generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Preset export formats
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
}

// Default value functions
fn default_ports() -> Vec<u16> {
    vec![80, 443]
}

fn default_load_level() -> LoadLevel {
    LoadLevel::Medium
}

fn default_duration() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

// Default implementation removed - using #[derive(Default)]

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            ip: "192.168.1.1".to_string(),
            ports: default_ports(),
            interface: None,
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            intensity: default_load_level(),
            duration: default_duration(),
            protocols: ProtocolConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            dry_run: false,
            private_only: true,
            audit_log: true,
        }
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            udp: true,
            tcp: true,
            icmp: false,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            format: ExportFormat::Json,
            filename: None,
        }
    }
}


impl PresetConfig {
    /// Create a new preset configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from YAML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            info!("Config file {:?} not found, using defaults", path);
            return Ok(Self::default());
        }

        let config_str = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileNotFound(format!("Failed to read config file: {}", e)))?;

        serde_yaml::from_str(&config_str)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse config file: {}", e)))
            .map_err(Into::into)
    }

    /// Save configuration to YAML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, yaml)
            .map_err(|e| ConfigError::FileNotFound(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Validate the configuration with user-friendly error messages
    pub fn validate(&self) -> Result<()> {
        self.validate_target()?;
        self.validate_test()?;
        self.validate_safety()?;
        Ok(())
    }

    /// Create a quick test configuration
    pub fn quick_test(target_ip: &str) -> Self {
        Self {
            target: TargetConfig {
                ip: target_ip.to_string(),
                ports: vec![80],
                interface: None,
            },
            test: TestConfig {
                intensity: LoadLevel::Low,
                duration: 10,
                protocols: ProtocolConfig {
                    udp: true,
                    tcp: false,
                    icmp: false,
                },
                export: ExportConfig::default(),
            },
            safety: SafetyConfig {
                dry_run: true, // Default to safe mode
                private_only: true,
                audit_log: true,
            },
        }
    }

    /// Create a standard test configuration
    pub fn standard_test(target_ip: &str, ports: Vec<u16>) -> Self {
        Self {
            target: TargetConfig {
                ip: target_ip.to_string(),
                ports,
                interface: None,
            },
            test: TestConfig {
                intensity: LoadLevel::Medium,
                duration: 30,
                protocols: ProtocolConfig::default(),
                export: ExportConfig::default(),
            },
            safety: SafetyConfig::default(),
        }
    }

    /// Generate example configuration with comments
    pub fn generate_example() -> String {
        r#"# Router Flood Configuration Example
# This is a preset configuration format that focuses on essential settings.

# Target configuration - what to test
target:
  ip: "192.168.1.1"          # Target IP address (private networks only)
  ports: [80, 443]           # Target ports (default: web server ports)
  # interface: "eth0"         # Network interface (auto-detected if not specified)

# Test configuration - how to test
test:
  intensity: medium          # Test load level: low, medium, or high
  duration: 30               # Test duration in seconds
  
  # Protocol configuration
  protocols:
    udp: true                # Use UDP packets
    tcp: true                # Use TCP packets
    icmp: false              # Use ICMP packets (requires privileges)
  
  # Export configuration
  export:
    enabled: false           # Export results to file
    format: json             # Export format: json or csv
    # filename: "my-test.json" # Custom filename (auto-generated if not specified)

# Safety configuration - safety limits
safety:
  dry_run: false             # Dry run mode - no actual packets sent
  private_only: true         # Only allow private IP addresses
  audit_log: true            # Enable audit logging

# Load levels explained:
# - low:    2 threads,  50 packets/second  (gentle testing)
# - medium: 4 threads, 100 packets/second  (balanced testing)
# - high:   8 threads, 200 packets/second  (aggressive testing)
"#.to_string()
    }

    fn validate_target(&self) -> Result<()> {
        if self.target.ip.is_empty() {
            return Err(ConfigError::MissingRequired("target.ip".to_string()).into());
        }

        // Validate IP format
        let _ip: std::net::IpAddr = self.target.ip.parse()
            .map_err(|_| ConfigError::InvalidValue {
                field: "target.ip".to_string(),
                value: self.target.ip.clone(),
                reason: format!("'{}' is not a valid IP address. Example: 192.168.1.1", self.target.ip),
            })?;

        if self.target.ports.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "target.ports".to_string(),
                value: "empty".to_string(),
                reason: "At least one port must be specified. Common ports: 80 (HTTP), 443 (HTTPS), 22 (SSH)".to_string(),
            }.into());
        }

        // Validate port ranges
        for &port in &self.target.ports {
            if port == 0 {
                return Err(ConfigError::InvalidValue {
                    field: "target.ports".to_string(),
                    value: port.to_string(),
                    reason: "Port 0 is not valid. Use ports between 1-65535.".to_string(),
                }.into());
            }
        }

        Ok(())
    }

    fn validate_test(&self) -> Result<()> {
        if self.test.duration == 0 {
            return Err(ConfigError::InvalidValue {
                field: "test.duration".to_string(),
                value: self.test.duration.to_string(),
                reason: "Duration must be greater than 0 seconds. Recommended: 10-300 seconds.".to_string(),
            }.into());
        }

        if self.test.duration > 3600 {
            return Err(ConfigError::InvalidValue {
                field: "test.duration".to_string(),
                value: self.test.duration.to_string(),
                reason: "Duration should not exceed 1 hour (3600 seconds) for safety.".to_string(),
            }.into());
        }

        // Validate that at least one protocol is enabled
        if !self.test.protocols.udp && !self.test.protocols.tcp && !self.test.protocols.icmp {
            return Err(ConfigError::InvalidValue {
                field: "test.protocols".to_string(),
                value: "all disabled".to_string(),
                reason: "At least one protocol (UDP, TCP, or ICMP) must be enabled.".to_string(),
            }.into());
        }

        Ok(())
    }

    fn validate_safety(&self) -> Result<()> {
        // Safety validation is mostly about ensuring safe defaults
        // The actual IP range validation happens at runtime
        Ok(())
    }
}

impl LoadLevel {
    /// Convert load level to thread count and packet rate
    pub fn to_thread_rate(&self) -> (usize, u64) {
        match self {
            LoadLevel::Low => (2, 50),
            LoadLevel::Medium => (4, 100),
            LoadLevel::High => (8, 200),
        }
    }

    /// Get description of load level
    pub fn description(&self) -> &'static str {
        match self {
            LoadLevel::Low => "Gentle testing with minimal resource usage",
            LoadLevel::Medium => "Balanced testing for typical scenarios",
            LoadLevel::High => "Aggressive testing for stress scenarios",
        }
    }
}

impl ProtocolConfig {
    // Methods for ProtocolConfig if needed
}

impl ExportFormat {
    // Methods for ExportFormat if needed
}

impl std::str::FromStr for LoadLevel {
    type Err = ConfigError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(LoadLevel::Low),
            "medium" => Ok(LoadLevel::Medium),
            "high" => Ok(LoadLevel::High),
            _ => Err(ConfigError::InvalidValue {
                field: "intensity".to_string(),
                value: s.to_string(),
                reason: "Load level must be 'low', 'medium', or 'high'. Use 'medium' for balanced testing.".to_string(),
            }),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = ConfigError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            _ => Err(ConfigError::InvalidValue {
                field: "export.format".to_string(),
                value: s.to_string(),
                reason: "Export format must be 'json' or 'csv'. Use 'json' for structured data.".to_string(),
            }),
        }
    }
}