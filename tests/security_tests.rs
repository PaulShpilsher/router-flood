//! Security validation tests

use router_flood::security::validation::validate_target_ip;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn test_private_ipv4_addresses_allowed() {
    // 192.168.0.0/16
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255))).is_ok());
    
    // 10.0.0.0/8
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255))).is_ok());
    
    // 172.16.0.0/12
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255))).is_ok());
}

#[test]
fn test_public_ipv4_addresses_rejected() {
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(172, 32, 0, 1))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(192, 167, 0, 1))).is_err());
}

#[test]
fn test_loopback_addresses_rejected() {
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(127, 255, 255, 255))).is_err());
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))).is_err());
}

#[test]
fn test_multicast_addresses_rejected() {
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(239, 255, 255, 255))).is_err());
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 1))).is_err());
}

#[test]
fn test_broadcast_addresses_rejected() {
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))).is_err());
}

#[test]
fn test_private_ipv6_addresses_allowed() {
    // Link-local fe80::/10
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1))).is_ok());
    
    // Unique local fc00::/7
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1))).is_ok());
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 1))).is_ok());
}

#[test]
fn test_public_ipv6_addresses_rejected() {
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888))).is_err());
}


#[test]
fn test_boundary_addresses() {
    // Edge of private ranges
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0))).is_ok());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255))).is_ok());
    
    // Just outside private ranges
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(9, 255, 255, 255))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(11, 0, 0, 0))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(172, 15, 255, 255))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(172, 32, 0, 0))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(192, 167, 255, 255))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(192, 169, 0, 0))).is_err());
}

#[test]
fn test_special_use_addresses() {
    // 0.0.0.0/8
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(0, 255, 255, 255))).is_err());
    
    // 169.254.0.0/16 (link-local)
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(169, 254, 0, 1))).is_err());
    assert!(validate_target_ip(&IpAddr::V4(Ipv4Addr::new(169, 254, 255, 255))).is_err());
}

#[test]
fn test_error_messages() {
    let result = validate_target_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Invalid IP range"));
    }
    
    let result = validate_target_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Invalid IP range"));
    }
}

#[test]
fn test_ipv6_special_addresses() {
    // Unspecified
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))).is_err());
    
    // Documentation 2001:db8::/32
    assert!(validate_target_ip(&IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1))).is_err());
}

#[test]
fn test_mixed_ip_versions() {
    let v4 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let v6 = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
    
    assert!(validate_target_ip(&v4).is_ok());
    assert!(validate_target_ip(&v6).is_ok());
}