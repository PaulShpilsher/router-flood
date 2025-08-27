//! Centralized configuration validation

use crate::config::Config;
use crate::error::{ValidationError, Result};
use crate::validation::validate_comprehensive_security;
use std::net::IpAddr;

/// Centralized configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Perform comprehensive validation of a configuration
    pub fn validate(config: &Config) -> Result<()> {
        // Parse and validate target IP
        let target_ip: IpAddr = config.target.ip.parse()
            .map_err(|_| ValidationError::InvalidIpRange {
                ip: config.target.ip.clone(),
                reason: "Invalid IP address format".to_string(),
            })?;
        
        // Validate comprehensive security
        validate_comprehensive_security(
            &target_ip,
            &config.target.ports,
            config.attack.threads,
            config.attack.packet_rate,
        )?;
        
        // Validate protocol mix ratios
        Self::validate_protocol_mix(config)?;
        
        // Validate packet size range
        Self::validate_packet_size_range(config)?;
        
        // Validate burst pattern
        Self::validate_burst_pattern(config)?;
        
        // Validate monitoring configuration
        Self::validate_monitoring_config(config)?;
        
        Ok(())
    }
    
    fn validate_protocol_mix(config: &Config) -> Result<()> {
        let mix = &config.target.protocol_mix;
        let total = mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + 
                   mix.icmp_ratio + mix.ipv6_ratio + mix.arp_ratio;
        
        if (total - 1.0).abs() > 0.001 {
            return Err(ValidationError::SystemRequirement(
                format!("Protocol ratios must sum to 1.0, got {:.3}", total)
            ).into());
        }
        
        // Check individual ratios are valid
        let ratios = [
            ("udp_ratio", mix.udp_ratio),
            ("tcp_syn_ratio", mix.tcp_syn_ratio),
            ("tcp_ack_ratio", mix.tcp_ack_ratio),
            ("icmp_ratio", mix.icmp_ratio),
            ("ipv6_ratio", mix.ipv6_ratio),
            ("arp_ratio", mix.arp_ratio),
        ];
        
        for (name, ratio) in ratios {
            if ratio < 0.0 || ratio > 1.0 {
                return Err(ValidationError::SystemRequirement(
                    format!("{} must be between 0.0 and 1.0, got {}", name, ratio)
                ).into());
            }
        }
        
        Ok(())
    }
    
    fn validate_packet_size_range(config: &Config) -> Result<()> {
        let (min_size, max_size) = config.attack.packet_size_range;
        
        if min_size > max_size {
            return Err(ValidationError::SystemRequirement(
                "Minimum packet size cannot be greater than maximum".to_string()
            ).into());
        }
        
        if min_size < crate::constants::MIN_PAYLOAD_SIZE {
            return Err(ValidationError::ExceedsLimit {
                field: "min_packet_size".to_string(),
                value: min_size as u64,
                limit: crate::constants::MIN_PAYLOAD_SIZE as u64,
            }.into());
        }
        
        if max_size > crate::constants::MAX_PAYLOAD_SIZE {
            return Err(ValidationError::ExceedsLimit {
                field: "max_packet_size".to_string(),
                value: max_size as u64,
                limit: crate::constants::MAX_PAYLOAD_SIZE as u64,
            }.into());
        }
        
        Ok(())
    }
    
    fn validate_burst_pattern(config: &Config) -> Result<()> {
        use crate::config::BurstPattern;
        
        match &config.attack.burst_pattern {
            BurstPattern::Sustained { rate } => {
                if *rate == 0 {
                    return Err(ValidationError::SystemRequirement(
                        "Sustained rate must be greater than 0".to_string()
                    ).into());
                }
                if *rate > crate::constants::MAX_PACKET_RATE {
                    return Err(ValidationError::ExceedsLimit {
                        field: "sustained_rate".to_string(),
                        value: *rate,
                        limit: crate::constants::MAX_PACKET_RATE,
                    }.into());
                }
            }
            BurstPattern::Bursts { burst_size, burst_interval_ms } => {
                if *burst_size == 0 {
                    return Err(ValidationError::SystemRequirement(
                        "Burst size must be greater than 0".to_string()
                    ).into());
                }
                if *burst_interval_ms == 0 {
                    return Err(ValidationError::SystemRequirement(
                        "Burst interval must be greater than 0".to_string()
                    ).into());
                }
            }
            BurstPattern::Ramp { start_rate, end_rate, ramp_duration } => {
                if *start_rate == 0 || *end_rate == 0 {
                    return Err(ValidationError::SystemRequirement(
                        "Ramp start and end rates must be greater than 0".to_string()
                    ).into());
                }
                if *ramp_duration == 0 {
                    return Err(ValidationError::SystemRequirement(
                        "Ramp duration must be greater than 0".to_string()
                    ).into());
                }
                if *start_rate > crate::constants::MAX_PACKET_RATE || 
                   *end_rate > crate::constants::MAX_PACKET_RATE {
                    return Err(ValidationError::ExceedsLimit {
                        field: "ramp_rate".to_string(),
                        value: (*start_rate).max(*end_rate),
                        limit: crate::constants::MAX_PACKET_RATE,
                    }.into());
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_monitoring_config(config: &Config) -> Result<()> {
        if config.monitoring.stats_interval == 0 {
            return Err(ValidationError::SystemRequirement(
                "Stats interval must be greater than 0".to_string()
            ).into());
        }
        
        if let Some(export_interval) = config.monitoring.export_interval {
            if export_interval == 0 {
                return Err(ValidationError::SystemRequirement(
                    "Export interval must be greater than 0".to_string()
                ).into());
            }
        }
        
        Ok(())
    }
}