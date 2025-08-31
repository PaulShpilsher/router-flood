//! Configuration validation utilities
//!
//! This module provides validation helpers for configuration values.

use crate::error::{Result, ValidationError};
use std::net::IpAddr;
use std::str::FromStr;

/// Validate an IP address string
pub fn validate_ip(ip_str: &str) -> Result<IpAddr> {
    IpAddr::from_str(ip_str)
        .map_err(|_| ValidationError::new("ip", "Invalid IP address format").into())
}

/// Validate a port number
pub fn validate_port(port: u16) -> Result<u16> {
    if port == 0 {
        Err(ValidationError::new("port", "Port number cannot be 0").into())
    } else {
        Ok(port)
    }
}

/// Validate a list of ports
pub fn validate_ports(ports: &[u16]) -> Result<()> {
    if ports.is_empty() {
        return Err(ValidationError::new("ports", "At least one port must be specified").into());
    }
    
    for &port in ports {
        validate_port(port)?;
    }
    
    Ok(())
}

/// Validate thread count
pub fn validate_threads(threads: usize) -> Result<usize> {
    if threads == 0 {
        Err(ValidationError::new("threads", "Thread count must be at least 1").into())
    } else if threads > 256 {
        Err(ValidationError::new("threads", "Thread count cannot exceed 256").into())
    } else {
        Ok(threads)
    }
}

/// Validate packet rate
pub fn validate_packet_rate(rate: f64) -> Result<f64> {
    if rate <= 0.0 {
        Err(ValidationError::new("packet_rate", "Packet rate must be positive").into())
    } else if rate > 1_000_000_000.0 {
        Err(ValidationError::new("packet_rate", "Packet rate is unrealistically high").into())
    } else {
        Ok(rate)
    }
}

/// Validate payload size
pub fn validate_payload_size(size: usize) -> Result<usize> {
    if size < 1 {
        Err(ValidationError::new("payload_size", "Payload size must be at least 1 byte").into())
    } else if size > 65507 {
        Err(ValidationError::new("payload_size", "Payload size cannot exceed 65507 bytes (UDP limit)").into())
    } else {
        Ok(size)
    }
}

/// Validate duration
pub fn validate_duration(duration: Option<u64>) -> Result<Option<u64>> {
    match duration {
        Some(0) => Err(ValidationError::new("duration", "Duration must be at least 1 second").into()),
        Some(d) if d > 86400 => Err(ValidationError::new("duration", "Duration cannot exceed 24 hours").into()),
        _ => Ok(duration)
    }
}

/// Validate bandwidth limit
pub fn validate_bandwidth(mbps: Option<f64>) -> Result<Option<f64>> {
    match mbps {
        Some(bw) if bw <= 0.0 => Err(ValidationError::new("bandwidth", "Bandwidth limit must be positive").into()),
        Some(bw) if bw > 100_000.0 => Err(ValidationError::new("bandwidth", "Bandwidth limit exceeds 100 Gbps").into()),
        _ => Ok(mbps)
    }
}

/// Validate protocol mix ratios
pub fn validate_protocol_mix(
    udp: f64,
    tcp_syn: f64,
    tcp_ack: f64,
    icmp: f64,
    custom: f64
) -> Result<()> {
    let total = udp + tcp_syn + tcp_ack + icmp + custom;
    
    if (total - 1.0).abs() > 0.01 {
        Err(ValidationError::new(
            "protocol_mix",
            format!("Protocol ratios must sum to 1.0, got {:.2}", total)
        ).into())
    } else {
        Ok(())
    }
}

/// Validate network interface name
pub fn validate_interface(interface: Option<&str>) -> Result<Option<String>> {
    match interface {
        Some(iface) if iface.is_empty() => {
            Err(ValidationError::new("interface", "Interface name cannot be empty").into())
        }
        Some(iface) if iface.len() > 16 => {
            Err(ValidationError::new("interface", "Interface name too long").into())
        }
        Some(iface) => Ok(Some(iface.to_string())),
        None => Ok(None)
    }
}

/// Validate export path
pub fn validate_export_path(path: &str) -> Result<String> {
    if path.is_empty() {
        Err(ValidationError::new("export_path", "Export path cannot be empty").into())
    } else {
        Ok(path.to_string())
    }
}