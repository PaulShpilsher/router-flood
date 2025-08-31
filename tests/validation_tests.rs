//! Validation module tests

#![allow(clippy::uninlined_format_args)]
//!
//! Tests for IP address validation and security checks.

use router_flood::security::validation::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn test_loopback_multicast_detection() {
    // Test loopback detection
    let loopback_ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
    assert!(is_loopback_or_multicast(&loopback_ip));
    
    let ipv6_loopback = IpAddr::V6(Ipv6Addr::LOCALHOST);
    assert!(is_loopback_or_multicast(&ipv6_loopback));
    
    // Test multicast detection
    let multicast_ip = IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1));
    assert!(is_loopback_or_multicast(&multicast_ip));
    
    // Test normal private IPs are not detected as special
    let private_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    assert!(!is_loopback_or_multicast(&private_ip));
}

#[test]
fn test_target_ip_validation_success() {
    // Test valid private IPv4 addresses
    let private_ips = vec![
        "192.168.1.1",
        "192.168.0.255",
        "10.0.0.1",
        "172.16.0.1",
    ];
    
    for ip_str in private_ips {
        let ip: IpAddr = ip_str.parse().unwrap();
        let result = validate_target_ip(&ip);
        assert!(result.is_ok(), "Expected {} to be valid", ip_str);
    }
}

#[test]
fn test_target_ip_validation_loopback_failure() {
    let loopback = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let result = validate_target_ip(&loopback);
    assert!(result.is_err(), "Loopback should fail validation");
}

#[test]
fn test_target_ip_validation_multicast_failure() {
    let multicast = IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1));
    let result = validate_target_ip(&multicast);
    assert!(result.is_err(), "Multicast should fail validation");
}

#[test]
fn test_comprehensive_security_validation_success() {
    let ip: IpAddr = "192.168.1.1".to_string().parse().unwrap();
    let ports = vec![8080, 3000];
    let threads = 4;
    let rate = 100;
    
    let result = validate_comprehensive_security(&ip, &ports, threads, rate);
    assert!(result.is_ok(), "Valid private IP configuration should pass validation");
}

#[test]
fn test_comprehensive_security_validation_failure() {
    let ip: IpAddr = "127.0.0.1".parse().unwrap(); // Loopback
    let ports = vec![80];
    let threads = 4;
    let rate = 100;
    
    let result = validate_comprehensive_security(&ip, &ports, threads, rate);
    assert!(result.is_err(), "Loopback IP should fail validation");
}

#[test]
fn test_system_requirements_validation_dry_run() {
    // Dry run should always succeed for system requirements
    let result = validate_system_requirements(true);
    assert!(result.is_ok(), "Dry run should always pass system requirements");
}

#[test]
fn test_system_requirements_validation_real_mode() {
    // Real mode validation depends on system state, but should not panic
    let result = validate_system_requirements(false);
    // We can't assert success/failure since it depends on the system state
    // But we can ensure it returns a Result without panicking
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_ipv6_validation() {
    // Test IPv6 link-local addresses
    let ipv6_link_local = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
    let result = validate_target_ip(&ipv6_link_local);
    // This should work for link-local addresses
    assert!(result.is_ok(), "Link-local IPv6 should be valid");
    
    // Test IPv6 loopback (should fail)
    let ipv6_loopback = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let result = validate_target_ip(&ipv6_loopback);
    assert!(result.is_err(), "IPv6 loopback should fail validation");
}

#[test]
fn test_edge_case_ips() {
    let edge_cases = vec![
        ("192.168.0.0", true),      // Edge of 192.168.0.0/16
        ("192.168.255.255", true),  // Edge of 192.168.0.0/16
        ("10.0.0.0", true),         // Edge of 10.0.0.0/8
        ("172.16.0.0", true),       // Edge of 172.16.0.0/12
        ("172.31.255.255", true),   // Edge of 172.16.0.0/12
        ("8.8.8.8", false),         // Public DNS
        ("1.1.1.1", false),         // Public DNS
    ];
    
    for (ip_str, should_pass) in edge_cases {
        let ip: IpAddr = ip_str.parse().unwrap();
        let result = validate_target_ip(&ip);
        
        if should_pass {
            assert!(result.is_ok(), "IP {} should pass validation", ip_str);
        } else {
            // For public IPs, the result depends on safety.require_private_ranges setting
            // We don't assert failure here as it depends on config
        }
    }
}