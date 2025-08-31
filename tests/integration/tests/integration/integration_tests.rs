//! Integration tests for router-flood

#![allow(clippy::uninlined_format_args)]
//!
//! These tests verify the integration and interaction between different modules
//! of the router-flood application, focusing on end-to-end functionality.

use router_flood::config::{default_config, load_config};
use router_flood::security::validation::{validate_system_requirements, validate_target_ip, validate_comprehensive_security};
use router_flood::error::RouterFloodError;
use router_flood::stats::Stats;
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_config_and_validation_integration() {
    let config = default_config();
    let target_ip = config.target.ip.parse::<IpAddr>().unwrap();
    
    // Test that default config passes all validations
    let validation_result = validate_comprehensive_security(
        &target_ip,
        &config.target.ports,
        config.attack.threads,
        config.attack.packet_rate,
    );
    
    assert!(validation_result.is_ok(), "Default config should pass validation");
}

#[test]
fn test_nonexistent_config_file_fallback() {
    let result = load_config(Some("definitely_nonexistent_file.yaml"));
    
    // Should return default config when file doesn't exist
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.attack.threads, 4);
    assert_eq!(config.attack.packet_rate, 100);
}

#[test]
fn test_comprehensive_ip_validation() {
    let test_cases = vec![
        // Valid private IPs
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), true),
        (IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), true),
        (IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)), true),
        
        // Invalid public IPs
        (IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), false),        // Google DNS
        (IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), false),        // Cloudflare DNS
        (IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)), false),    // TEST-NET-3
        
        // Loopback (should be invalid)
        (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), false),
        (IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), false),
    ];
    
    for (ip, should_be_valid) in test_cases {
        let result = validate_target_ip(&ip);
        
        if should_be_valid {
            assert!(result.is_ok(), "Expected {} to be valid private IP", ip);
        } else {
            assert!(result.is_err(), "Expected {} to be invalid IP", ip);
            
            // Verify we get the expected error types
            match result.unwrap_err() {
                RouterFloodError::Validation(_) => {
                    // Expected validation error
                }
                other => panic!("Expected ValidationError for IP {}, got {:?}", ip, other),
            }
        }
    }
}

#[test]
fn test_system_requirements_dry_run_integration() {
    // Dry run should always pass validation regardless of privileges
    let result = validate_system_requirements(true);
    assert!(result.is_ok());
}

#[test]
fn test_stats_and_protocols_integration() {
    let stats = Stats::default();
    
    // Test that all protocol constants work with stats
    let test_protocols = vec!["UDP", "TCP", "ICMP", "IPv6", "ARP"];
    
    for protocol in &test_protocols {
        stats.increment_sent(100, protocol);
    }
    
    // Verify all increments worked
    assert_eq!(stats.packets_sent(), test_protocols.len() as u64);
    assert_eq!(stats.bytes_sent(), 100 * test_protocols.len() as u64);
    
    // Test failed increments
    stats.increment_failed();
    assert_eq!(stats.packets_failed(), 1);
}

#[test]
fn test_configuration_security_limits() {
    let mut config = default_config();
    
    // Test that safety limits are enforced
    config.attack.threads = 1000; // Exceeds MAX_THREADS
    config.attack.packet_rate = 50000; // Exceeds MAX_PACKET_RATE
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    let result = validate_comprehensive_security(
        &target_ip,
        &config.target.ports,
        config.attack.threads,
        config.attack.packet_rate,
    );
    
    assert!(result.is_err(), "Should reject configuration exceeding safety limits");
}

#[test]
fn test_error_type_conversions() {
    // Test that various error types can be converted properly
    use router_flood::error::{ConfigError, NetworkError, ValidationError};
    
    let config_error = ConfigError::FileNotFound("test.yaml".to_string());
    let router_error: RouterFloodError = config_error.into();
    
    match router_error {
        RouterFloodError::Config(_) => {} // Expected
        _ => panic!("Expected Config error variant"),
    }
    
    let network_error = NetworkError::InterfaceNotFound("eth0".to_string());
    let router_error: RouterFloodError = network_error.into();
    
    match router_error {
        RouterFloodError::Network(_) => {} // Expected
        _ => panic!("Expected Network error variant"),
    }
    
    let validation_error = ValidationError::InvalidIpRange {
        ip: "8.8.8.8".to_string(),
        reason: "Not private",
    };
    let router_error: RouterFloodError = validation_error.into();
    
    match router_error {
        RouterFloodError::Validation(_) => {} // Expected
        _ => panic!("Expected Validation error variant"),
    }
}

#[test]
fn test_session_id_uniqueness() {
    // Test that multiple FloodStats instances have unique session IDs
    let stats1 = Stats::default();
    let stats2 = Stats::default();
    let stats3 = Stats::default();
    
    assert_ne!(stats1.session_id, stats2.session_id);
    assert_ne!(stats2.session_id, stats3.session_id);
    assert_ne!(stats1.session_id, stats3.session_id);
    
    // Session IDs should not be empty
    assert!(!stats1.session_id.is_empty());
    assert!(!stats2.session_id.is_empty());
    assert!(!stats3.session_id.is_empty());
}

#[test]
fn test_protocol_constants_consistency() {
    use router_flood::constants::protocols;
    
    // Verify all protocol constants are defined and non-empty
    assert!(!protocols::UDP.is_empty());
    assert!(!protocols::TCP.is_empty());
    assert!(!protocols::ICMP.is_empty());
    assert!(!protocols::IPV6.is_empty());
    assert!(!protocols::ARP.is_empty());
    
    // Verify ALL_PROTOCOLS contains all individual protocols
    assert!(protocols::ALL_PROTOCOLS.contains(&protocols::UDP));
    assert!(protocols::ALL_PROTOCOLS.contains(&protocols::TCP));
    assert!(protocols::ALL_PROTOCOLS.contains(&protocols::ICMP));
    assert!(protocols::ALL_PROTOCOLS.contains(&protocols::IPV6));
    assert!(protocols::ALL_PROTOCOLS.contains(&protocols::ARP));
    
    // Verify expected count
    assert_eq!(protocols::ALL_PROTOCOLS.len(), 5);
}

#[tokio::test]
async fn test_stats_export_dry_run() {
    use router_flood::config::{Export, ExportFormat};
    
    let export_config = Export {
        enabled: true,
        format: ExportFormat::Json,
        filename_pattern: "test".to_string(),
        include_system_stats: false,
    };
    
    let stats = Stats::new(Some(export_config));
    
    // Add some test data
    stats.increment_sent(100, "UDP");
    stats.increment_sent(200, "TCP");
    stats.increment_failed();
    
    // This test doesn't actually export (to avoid file system side effects in tests)
    // but ensures the stats are properly structured for export
    assert!(stats.export_config.is_some());
    assert_eq!(stats.packets_sent(), 2);
    assert_eq!(stats.bytes_sent(), 300);
    assert_eq!(stats.packets_failed(), 1);
}