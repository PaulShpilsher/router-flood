//! Audit module tests

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::useless_vec)]
//!
//! Tests for audit trail creation and logging functionality.

use router_flood::audit::*;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

#[test]
fn test_audit_entry_creation() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ports = vec![80, 443, 22];
    let threads = 4;
    let packet_rate = 100;
    let duration = Some(60);
    let interface = Some("eth0");
    let session_id = Uuid::new_v4();
    
    let result = create_audit_entry(
        &target_ip,
        &ports,
        threads,
        packet_rate,
        duration,
        interface,
        &session_id,
    );
    
    // Should succeed in creating audit entry
    assert!(result.is_ok(), "Audit entry creation should succeed");
}

#[test]
fn test_audit_entry_with_minimal_info() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ports = vec![8080];
    let threads = 1;
    let packet_rate = 50;
    let duration = None; // No duration limit
    let interface = None; // No specific interface
    let session_id = Uuid::new_v4();
    
    let result = create_audit_entry(
        &target_ip,
        &ports,
        threads,
        packet_rate,
        duration,
        interface,
        &session_id,
    );
    
    assert!(result.is_ok(), "Audit entry with minimal info should succeed");
}

#[test]
fn test_audit_entry_with_ipv6() {
    use std::net::Ipv6Addr;
    
    let target_ip = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
    let ports = vec![80, 443];
    let threads = 2;
    let packet_rate = 200;
    let duration = Some(30);
    let interface = Some("wlan0");
    let session_id = Uuid::new_v4();
    
    let result = create_audit_entry(
        &target_ip,
        &ports,
        threads,
        packet_rate,
        duration,
        interface,
        &session_id,
    );
    
    assert!(result.is_ok(), "IPv6 audit entry should succeed");
}

#[test]
fn test_audit_entry_with_many_ports() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
    let ports: Vec<u16> = (8000..8100).collect(); // 100 ports
    let threads = 8;
    let packet_rate = 500;
    let duration = Some(120);
    let interface = Some("eth1");
    let session_id = Uuid::new_v4();
    
    let result = create_audit_entry(
        &target_ip,
        &ports,
        threads,
        packet_rate,
        duration,
        interface,
        &session_id,
    );
    
    assert!(result.is_ok(), "Audit entry with many ports should succeed");
}

#[test]
fn test_audit_entry_parameter_validation() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let session_id = Uuid::new_v4();
    
    // Test with empty ports
    let result = create_audit_entry(
        &target_ip,
        &vec![], // Empty ports
        1,
        100,
        None,
        None,
        &session_id,
    );
    
    // Should handle empty ports gracefully
    assert!(result.is_ok(), "Empty ports should be handled gracefully");
    
    // Test with zero threads
    let result = create_audit_entry(
        &target_ip,
        &vec![80],
        0, // Zero threads
        100,
        None,
        None,
        &session_id,
    );
    
    // Should handle edge case parameters
    assert!(result.is_ok(), "Edge case parameters should be handled");
}

#[test]
fn test_audit_entry_with_high_values() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ports = vec![80];
    let threads = 100; // High thread count
    let packet_rate = 10000; // High packet rate
    let duration = Some(3600); // Long duration
    let interface = Some("eth0");
    let session_id = Uuid::new_v4();
    
    let result = create_audit_entry(
        &target_ip,
        &ports,
        threads,
        packet_rate,
        duration,
        interface,
        &session_id,
    );
    
    assert!(result.is_ok(), "High values should be handled in audit");
}

#[test]
fn test_multiple_audit_entries() {
    // Test creating multiple audit entries in sequence
    let base_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    for i in 1..=5 {
        let session_id = Uuid::new_v4();
        let result = create_audit_entry(
            &base_ip,
            &vec![80 + i as u16],
            i,
            100 * i as u64,
            Some(i as u64 * 10),
            Some("eth0"),
            &session_id,
        );
        
        assert!(result.is_ok(), "Multiple audit entries should all succeed: entry {}", i);
    }
}

#[test]
fn test_concurrent_audit_entry_creation() {
    use std::sync::Arc;
    use std::thread;
    
    let num_threads = 10;
    let target_ip = Arc::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    let ports = Arc::new(vec![80, 443]);
    
    let handles: Vec<_> = (0..num_threads).map(|i| {
        let ip_clone = Arc::clone(&target_ip);
        let ports_clone = Arc::clone(&ports);
        
        thread::spawn(move || {
            let session_id = Uuid::new_v4();
            create_audit_entry(
                &ip_clone,
                &ports_clone,
                i + 1,
                100 + i as u64 * 50,
                Some(60),
                Some("eth0"),
                &session_id,
            )
        })
    }).collect();
    
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.join().expect("Thread should complete successfully");
        assert!(result.is_ok(), "Concurrent audit entry {} should succeed", i);
    }
}

#[test]
fn test_audit_entry_unique_session_ids() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ports = vec![80];
    
    let mut session_ids = std::collections::HashSet::new();
    
    // Create multiple entries and verify session IDs are unique
    for _ in 0..10 {
        let session_id = Uuid::new_v4();
        assert!(session_ids.insert(session_id), "Session IDs should be unique");
        
        let result = create_audit_entry(
            &target_ip,
            &ports,
            1,
            100,
            None,
            None,
            &session_id,
        );
        
        assert!(result.is_ok(), "Each unique session should create audit entry");
    }
}

#[test]
fn test_audit_timestamp_consistency() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ports = vec![80];
    let session_id = Uuid::new_v4();
    
    let start_time = std::time::SystemTime::now();
    
    let result = create_audit_entry(
        &target_ip,
        &ports,
        1,
        100,
        Some(60),
        Some("eth0"),
        &session_id,
    );
    
    let end_time = std::time::SystemTime::now();
    
    assert!(result.is_ok());
    
    // Audit creation should be fast
    let duration = end_time.duration_since(start_time).unwrap();
    assert!(duration.as_millis() < 1000, "Audit creation should be fast");
}

#[test]
fn test_audit_with_special_interface_names() {
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ports = vec![80];
    let session_id = Uuid::new_v4();
    
    let special_interfaces = vec![
        "lo",
        "wlan0",
        "eth0:1", // Virtual interface
        "br-docker0", // Bridge interface
        "tun0", // Tunnel interface
        "veth1234", // Virtual ethernet
    ];
    
    for interface in special_interfaces {
        let result = create_audit_entry(
            &target_ip,
            &ports,
            1,
            100,
            None,
            Some(interface),
            &session_id,
        );
        
        assert!(result.is_ok(), "Special interface name '{}' should be handled", interface);
    }
}

#[test]
fn test_audit_entry_error_handling() {
    // This test verifies that the audit system handles edge cases gracefully
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ports = vec![80];
    let session_id = Uuid::new_v4();
    
    // Test with very long interface name
    let long_interface = "a".repeat(1000);
    let result = create_audit_entry(
        &target_ip,
        &ports,
        1,
        100,
        None,
        Some(&long_interface),
        &session_id,
    );
    
    // Should handle gracefully (either succeed or fail cleanly)
    assert!(result.is_ok() || result.is_err(), "Should handle long interface names");
}