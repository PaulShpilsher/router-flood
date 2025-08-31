//! Configuration builder with fluent API and validation

use super::validation::ConfigValidator;
use crate::config::{Config, ProtocolMix, BurstPattern};
use crate::error::{ConfigError, ValidationError, Result, messages};
use crate::security::validation::validate_target_ip;
use std::net::IpAddr;

/// Builder for creating and validating configurations
#[derive(Debug)]
pub struct ConfigBuilder {
    config: Config,
    errors: Vec<ValidationError>,
}

impl ConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new() -> Self {
        Self {
            config: crate::config::default_config(),
            errors: Vec::new(),
        }
    }
    
    /// Create a builder from an existing configuration
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            errors: Vec::new(),
        }
    }
    
    /// Set the target IP address with validation
    pub fn target_ip(mut self, ip: &str) -> Self {
        match ip.parse::<IpAddr>() {
            Ok(addr) => {
                if let Err(e) = validate_target_ip(&addr) {
                    if let crate::error::RouterFloodError::Validation(val_err) = e {
                        self.errors.push(val_err);
                    }
                } else {
                    self.config.target.ip = ip.to_string();
                }
            }
            Err(_) => {
                self.errors.push(ValidationError::InvalidIpRange {
                    ip: ip.to_string(),
                    reason: messages::INVALID_IP_FORMAT,
                });
            }
        }
        self
    }
    
    /// Set target ports with validation
    pub fn target_ports(mut self, ports: Vec<u16>) -> Self {
        if ports.is_empty() {
            self.errors.push(ValidationError::SystemRequirement(
                messages::NO_TARGET_PORTS
            ));
        } else if ports.len() > 100 {
            self.errors.push(ValidationError::ExceedsLimit {
                field: messages::TARGET_PORTS_FIELD,
                value: ports.len() as u64,
                limit: 100,
            });
        } else {
            self.config.target.ports = ports;
        }
        self
    }
    
    /// Set protocol mix with validation
    pub fn protocol_mix(mut self, mix: ProtocolMix) -> Self {
        let total = mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + 
                   mix.icmp_ratio + mix.ipv6_ratio + mix.arp_ratio;
        
        if (total - 1.0).abs() > 0.001 {
            self.errors.push(ValidationError::SystemRequirement(
                messages::PROTOCOL_RATIOS_SUM
            ));
        } else if [mix.udp_ratio, mix.tcp_syn_ratio, mix.tcp_ack_ratio, 
                   mix.icmp_ratio, mix.ipv6_ratio, mix.arp_ratio]
                   .iter().any(|&ratio| !(0.0..=1.0).contains(&ratio)) {
            self.errors.push(ValidationError::SystemRequirement(
                messages::PROTOCOL_RATIOS_RANGE
            ));
        } else {
            self.config.target.protocol_mix = mix;
        }
        self
    }
    
    /// Set thread count with validation
    pub fn threads(mut self, threads: usize) -> Self {
        if threads == 0 {
            self.errors.push(ValidationError::SystemRequirement(
                messages::THREAD_COUNT_ZERO
            ));
        } else if threads > crate::constants::MAX_THREADS {
            self.errors.push(ValidationError::ExceedsLimit {
                field: messages::THREAD_COUNT_EXCEEDS_LIMIT,
                value: threads as u64,
                limit: crate::constants::MAX_THREADS as u64,
            });
        } else {
            self.config.attack.threads = threads;
        }
        self
    }
    
    /// Set packet rate with validation
    pub fn packet_rate(mut self, rate: u64) -> Self {
        if rate == 0 {
            self.errors.push(ValidationError::SystemRequirement(
                messages::PACKET_RATE_ZERO
            ));
        } else if rate > crate::constants::MAX_PACKET_RATE {
            self.errors.push(ValidationError::ExceedsLimit {
                field: messages::PACKET_RATE_EXCEEDS_LIMIT,
                value: rate,
                limit: crate::constants::MAX_PACKET_RATE,
            });
        } else {
            self.config.attack.packet_rate = rate;
        }
        self
    }
    
    /// Set packet size range with validation
    pub fn packet_size_range(mut self, min_size: usize, max_size: usize) -> Self {
        if min_size > max_size {
            self.errors.push(ValidationError::SystemRequirement(
                messages::MIN_PACKET_SIZE_TOO_LARGE
            ));
        } else if min_size < crate::constants::MIN_PAYLOAD_SIZE {
            self.errors.push(ValidationError::ExceedsLimit {
                field: messages::MIN_PACKET_SIZE_FIELD,
                value: min_size as u64,
                limit: crate::constants::MIN_PAYLOAD_SIZE as u64,
            });
        } else if max_size > crate::constants::MAX_PAYLOAD_SIZE {
            self.errors.push(ValidationError::ExceedsLimit {
                field: messages::MAX_PACKET_SIZE_FIELD,
                value: max_size as u64,
                limit: crate::constants::MAX_PAYLOAD_SIZE as u64,
            });
        } else {
            self.config.attack.packet_size_range = (min_size, max_size);
        }
        self
    }
    
    /// Set burst pattern
    pub fn burst_pattern(mut self, pattern: BurstPattern) -> Self {
        // Validate burst pattern parameters
        match &pattern {
            BurstPattern::Sustained { rate } => {
                if *rate == 0 {
                    self.errors.push(ValidationError::SystemRequirement(
                        messages::SUSTAINED_RATE_ZERO
                    ));
                } else if *rate > crate::constants::MAX_PACKET_RATE {
                    self.errors.push(ValidationError::ExceedsLimit {
                        field: messages::SUSTAINED_RATE_FIELD,
                        value: *rate,
                        limit: crate::constants::MAX_PACKET_RATE,
                    });
                }
            }
            BurstPattern::Bursts { burst_size, burst_interval_ms } => {
                if *burst_size == 0 {
                    self.errors.push(ValidationError::SystemRequirement(
                        messages::BURST_SIZE_ZERO
                    ));
                }
                if *burst_interval_ms == 0 {
                    self.errors.push(ValidationError::SystemRequirement(
                        messages::BURST_INTERVAL_ZERO
                    ));
                }
            }
            BurstPattern::Ramp { start_rate, end_rate, ramp_duration } => {
                if *start_rate == 0 || *end_rate == 0 {
                    self.errors.push(ValidationError::SystemRequirement(
                        messages::RAMP_RATES_ZERO
                    ));
                }
                if *ramp_duration == 0 {
                    self.errors.push(ValidationError::SystemRequirement(
                        messages::RAMP_DURATION_ZERO
                    ));
                }
            }
        }
        
        if self.errors.is_empty() {
            self.config.attack.burst_pattern = pattern;
        }
        self
    }
    
    /// Enable or disable dry-run mode
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.config.safety.dry_run = enabled;
        self
    }
    
    /// Set duration in seconds
    pub fn duration(mut self, duration_secs: Option<u64>) -> Self {
        if let Some(duration) = duration_secs {
            if duration == 0 {
                self.errors.push(ValidationError::SystemRequirement(
                    messages::DURATION_ZERO
                ));
            } else {
                self.config.attack.duration = Some(duration);
            }
        } else {
            self.config.attack.duration = None;
        }
        self
    }
    
    /// Build the configuration, returning errors if validation failed
    #[must_use = "Configuration must be validated before use"]
    pub fn build(self) -> Result<Config> {
        if !self.errors.is_empty() {
            // Create a comprehensive error message
            let error_messages: Vec<String> = self.errors
                .iter()
                .map(|e| e.to_string())
                .collect();
            
            return Err(ConfigError::InvalidValue {
                field: "configuration".to_string(),
                value: "multiple_errors".to_string(),
                reason: format!("Validation failed: {}", error_messages.join("; ")),
            }.into());
        }
        
        // Final comprehensive validation
        ConfigValidator::validate(&self.config)?;
        
        Ok(self.config)
    }
    
    /// Get current validation errors without building
    pub fn validation_errors(&self) -> &[ValidationError] {
        &self.errors
    }
    
    /// Check if the current configuration is valid
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

