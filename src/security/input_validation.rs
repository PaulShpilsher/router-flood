//! Input validation for security hardening
//!
//! This module provides input validation with security-focused checks.

use std::net::IpAddr;
use crate::error::{Result, ValidationError};

/// Input validator with security focus
pub struct InputValidation {
    config: ValidationConfig,
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_string_length: usize,
    pub max_array_size: usize,
    pub allow_special_chars: bool,
    pub strict_ip_validation: bool,
    pub enable_pattern_detection: bool,
}

/// Validation result with security context
#[derive(Debug)]
pub struct ValidationResult<T> {
    pub value: T,
    pub warnings: Vec<String>,
}

/// Sanitized string wrapper
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedString {
    value: String,
}

/// Validated IP address with security metadata
#[derive(Debug, Clone)]
pub struct ValidatedIpAddr {
    pub addr: IpAddr,
    pub is_private: bool,
    pub security_level: IpSecurityLevel,
}

/// IP address security classification
#[derive(Debug, Clone, PartialEq)]
pub enum IpSecurityLevel {
    Safe,
    Restricted,
    Forbidden,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_string_length: 1024,
            max_array_size: 1000,
            allow_special_chars: false,
            strict_ip_validation: true,
            enable_pattern_detection: true,
        }
    }
}

impl InputValidation {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn validate_ip_address(&self, ip_str: &str) -> Result<ValidationResult<ValidatedIpAddr>> {
        let addr: IpAddr = ip_str.parse()
            .map_err(|_| ValidationError::InvalidIpRange {
                ip: ip_str.to_string(),
                reason: "Invalid IP address format",
            })?;
        
        let is_private = match addr {
            IpAddr::V4(ipv4) => ipv4.is_private(),
            IpAddr::V6(ipv6) => ipv6.is_unique_local(),
        };
        
        let security_level = if is_private {
            IpSecurityLevel::Safe
        } else {
            IpSecurityLevel::Forbidden
        };
        
        Ok(ValidationResult {
            value: ValidatedIpAddr {
                addr,
                is_private,
                security_level,
            },
            warnings: Vec::new(),
        })
    }

    pub fn validate_port_list(&self, ports: &[u16]) -> Result<ValidationResult<Vec<u16>>> {
        if ports.len() > self.config.max_array_size {
            return Err(ValidationError::ExceedsLimit {
                field: "port_list",
                value: ports.len() as u64,
                limit: self.config.max_array_size as u64,
            }.into());
        }
        Ok(ValidationResult {
            value: ports.to_vec(),
            warnings: Vec::new(),
        })
    }
}

impl SanitizedString {
    pub fn new(value: String) -> Self {
        Self { value }
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Convenience function to create an input validator
pub fn create_input_validator() -> InputValidation {
    InputValidation::new(ValidationConfig::default())
}

/// Convenience function to validate an IP address
pub fn validate_ip_address(ip_str: &str) -> Result<ValidatedIpAddr> {
    let addr: IpAddr = ip_str.parse()
        .map_err(|_| ValidationError::InvalidIpRange {
            ip: ip_str.to_string(),
            reason: "Invalid IP address format",
        })?;
    
    let is_private = match addr {
        IpAddr::V4(ipv4) => ipv4.is_private(),
        IpAddr::V6(ipv6) => ipv6.is_unique_local(),
    };
    
    let security_level = if is_private {
        IpSecurityLevel::Safe
    } else {
        IpSecurityLevel::Forbidden
    };
    
    Ok(ValidatedIpAddr {
        addr,
        is_private,
        security_level,
    })
}

/// Convenience function to validate a port list
pub fn validate_port_list(ports: &[u16]) -> Result<Vec<u16>> {
    if ports.len() > 1000 {
        return Err(ValidationError::ExceedsLimit {
            field: "port_list",
            value: ports.len() as u64,
            limit: 1000,
        }.into());
    }
    Ok(ports.to_vec())
}