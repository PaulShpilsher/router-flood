//! End-to-end security workflow integration tests

use router_flood::security::validation::{validate_target_ip, validate_comprehensive_security, validate_system_requirements};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn test_complete_security_validation_pipeline() {
    // Test the full security validation workflow
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    let ports = vec![80, 443, 8080];
    let threads = 4;
    let rate = 1000;
    
    // Step 1: Validate IP is private
    let ip_result = validate_target_ip(&ip);
    assert!(ip_result.is_ok());
    
    // Step 2: Comprehensive validation
    let comprehensive_result = validate_comprehensive_security(&ip, &ports, threads, rate);
    assert!(comprehensive_result.is_ok());
    
    // Step 3: System requirements (dry-run mode)
    let system_result = validate_system_requirements(true);
    assert!(system_result.is_ok());
}

#[test]
fn test_security_rejection_pipeline() {
    // Test that public IPs are properly rejected
    let public_ip: IpAddr = "8.8.8.8".parse().unwrap();
    let ports = vec![80];
    let threads = 4;
    let rate = 1000;
    
    // Should fail at IP validation
    let ip_result = validate_target_ip(&public_ip);
    assert!(ip_result.is_err());
    
    // Comprehensive should also fail
    let comprehensive_result = validate_comprehensive_security(&public_ip, &ports, threads, rate);
    assert!(comprehensive_result.is_err());
}

#[test]
fn test_ipv6_security_workflow() {
    // Test IPv6 private address validation
    let ipv6_private = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
    let ipv6_public = IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0, 0, 0, 0, 0, 0x8888));
    
    // Private should pass
    assert!(validate_target_ip(&ipv6_private).is_ok());
    assert!(validate_comprehensive_security(&ipv6_private, &vec![80], 4, 1000).is_ok());
    
    // Public should fail
    assert!(validate_target_ip(&ipv6_public).is_err());
    assert!(validate_comprehensive_security(&ipv6_public, &vec![80], 4, 1000).is_err());
}

#[test]
fn test_multi_layer_validation() {
    // Test that each layer of validation works correctly
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Valid configuration - all should pass
    assert!(validate_comprehensive_security(&ip, &vec![8080], 50, 5000).is_ok());
    
    // Invalid thread count
    assert!(validate_comprehensive_security(&ip, &vec![8080], 101, 5000).is_err());
    
    // Invalid rate
    assert!(validate_comprehensive_security(&ip, &vec![8080], 50, 10001).is_err());
    
    // Empty ports (allowed)
    assert!(validate_comprehensive_security(&ip, &vec![], 50, 5000).is_ok());
}

#[test]
fn test_loopback_and_multicast_rejection() {
    // Loopback should be rejected
    let loopback = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    assert!(validate_comprehensive_security(&loopback, &vec![80], 4, 1000).is_err());
    
    // Multicast should be rejected
    let multicast = IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1));
    assert!(validate_comprehensive_security(&multicast, &vec![80], 4, 1000).is_err());
    
    // Broadcast should be rejected
    let broadcast = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));
    assert!(validate_comprehensive_security(&broadcast, &vec![80], 4, 1000).is_err());
}