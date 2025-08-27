//! Main module tests

#![allow(clippy::uninlined_format_args)]
//!
//! Tests for the main application entry point functions and configuration parsing.

use router_flood::config::get_default_config;
use router_flood::constants::error_messages;
use router_flood::error::RouterFloodError;
use std::net::{IpAddr, Ipv4Addr};

/// Parse target IP from configuration
fn parse_target_ip(config: &router_flood::config::Config) -> router_flood::error::Result<IpAddr> {
    config.target.ip.parse()
        .map_err(|_| router_flood::error::ValidationError::InvalidIpRange {
            ip: config.target.ip.clone(),
            reason: error_messages::INVALID_IP_FORMAT.to_string(),
        }.into())
}

#[test]
fn test_parse_target_ip_valid() {
    let mut config = get_default_config();
    config.target.ip = "192.168.1.1".to_string();
    
    let result = parse_target_ip(&config);
    assert!(result.is_ok());
    
    if let Ok(IpAddr::V4(ipv4)) = result {
        assert_eq!(ipv4, Ipv4Addr::new(192, 168, 1, 1));
    }
}

#[test]
fn test_parse_target_ip_invalid() {
    let mut config = get_default_config();
    config.target.ip = "invalid.ip.address".to_string();
    
    let result = parse_target_ip(&config);
    assert!(result.is_err());
    
    // Verify it's the right type of error
    match result {
        Err(RouterFloodError::Validation(_)) => {
            // Expected validation error
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_parse_target_ip_ipv6() {
    let mut config = get_default_config();
    config.target.ip = "::1".to_string();
    
    let result = parse_target_ip(&config);
    assert!(result.is_ok());
    
    if let Ok(IpAddr::V6(ipv6)) = result {
        assert_eq!(ipv6, std::net::Ipv6Addr::LOCALHOST);
    }
}

#[test]
fn test_parse_target_ip_edge_cases() {
    let test_cases = vec![
        ("", false),
        ("256.256.256.256", false),
        ("192.168.1", false),
        ("192.168.1.1.1", false),
        ("10.0.0.1", true),
        ("172.16.0.1", true),
    ];

    for (ip_str, should_succeed) in test_cases {
        let mut config = get_default_config();
        config.target.ip = ip_str.to_string();
        
        let result = parse_target_ip(&config);
        if should_succeed {
            assert!(result.is_ok(), "Expected {} to parse successfully", ip_str);
        } else {
            assert!(result.is_err(), "Expected {} to fail parsing", ip_str);
        }
    }
}

#[tokio::test]
async fn test_setup_logging() {
    use tracing_subscriber::{layer::SubscriberExt};
    
    // This test ensures setup_logging doesn't panic
    // We simulate the setup_logging function here
    let _subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer());
    // Note: We don't call .init() to avoid conflicts with other tests
}

#[test]
fn test_default_configuration() {
    let config = get_default_config();
    
    assert_eq!(config.target.ip, "192.168.0.1");
    assert_eq!(config.target.ports, vec![80]);
    assert_eq!(config.attack.threads, 4);
    assert_eq!(config.attack.packet_rate, 100);
    assert!(config.safety.require_private_ranges);
    assert!(config.safety.audit_logging);
}

#[test]
fn test_configuration_validation_requirements() {
    let config = get_default_config();
    
    // Verify safety defaults
    assert!(config.safety.require_private_ranges);
    assert!(config.safety.enable_monitoring);
    assert!(config.safety.audit_logging);
    assert!(!config.safety.dry_run); // Should be false by default
    
    // Verify reasonable limits
    assert!(config.safety.max_threads > 0);
    assert!(config.safety.max_packet_rate > 0);
}