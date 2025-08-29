//! Comprehensive tests for core modules

use router_flood::core::network::{list_network_interfaces, find_interface_by_name, get_default_interface};
use router_flood::core::target::MultiPortTarget;
use router_flood::core::simulation::{Simulation, setup_network_interface};
mod common;
use common::create_test_config;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

#[test]
fn test_network_interface_discovery() {
    let interfaces = list_network_interfaces();
    
    // Should find at least one interface (loopback)
    assert!(!interfaces.is_empty(), "Should find at least loopback interface");
    
    // Check for loopback
    let loopback = interfaces.iter().find(|i| i.is_loopback());
    assert!(loopback.is_some(), "Should find loopback interface");
    
    // Verify loopback has expected properties
    if let Some(lo) = loopback {
        assert!(lo.is_loopback());
        assert!(lo.ips.iter().any(|ip| match ip {
            pnet::ipnetwork::IpNetwork::V4(net) => net.ip().is_loopback(),
            pnet::ipnetwork::IpNetwork::V6(net) => net.ip().is_loopback(),
        }));
    }
}

#[test]
fn test_find_interface_by_exact_name() {
    let interfaces = list_network_interfaces();
    
    if let Some(first) = interfaces.first() {
        let name = &first.name;
        let found = find_interface_by_name(name);
        
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, *name);
    }
}

#[test]
fn test_find_nonexistent_interface() {
    let found = find_interface_by_name("nonexistent_interface_xyz123");
    assert!(found.is_none());
}

#[test]
fn test_default_interface_selection() {
    let default = get_default_interface();
    
    if let Some(iface) = default {
        // Default should be up and not loopback
        assert!(iface.is_up());
        assert!(!iface.is_loopback());
        assert!(!iface.ips.is_empty());
    }
    // It's OK if no default interface is found (e.g., in containers)
}

#[test]
fn test_multi_port_target_rotation() {
    let ports = vec![80, 443, 8080, 3000];
    let target = MultiPortTarget::new(ports.clone());
    
    let mut seen_ports = std::collections::HashSet::new();
    
    // Get ports multiple times
    for _ in 0..100 {
        let port = target.next_port();
        assert!(ports.contains(&port));
        seen_ports.insert(port);
    }
    
    // Should have seen all ports
    assert_eq!(seen_ports.len(), ports.len());
}

#[test]
fn test_multi_port_target_single_port() {
    let target = MultiPortTarget::new(vec![8080]);
    
    // Should always return the same port
    for _ in 0..10 {
        assert_eq!(target.next_port(), 8080);
    }
}

#[test]
fn test_multi_port_target_concurrent_access() {
    let ports = vec![80, 443];
    let target = Arc::new(MultiPortTarget::new(ports));
    
    let mut handles = vec![];
    let counter_80 = Arc::new(AtomicU64::new(0));
    let counter_443 = Arc::new(AtomicU64::new(0));
    
    for _ in 0..10 {
        let target_clone = target.clone();
        let c80 = counter_80.clone();
        let c443 = counter_443.clone();
        
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let port = target_clone.next_port();
                match port {
                    80 => c80.fetch_add(1, Ordering::Relaxed),
                    443 => c443.fetch_add(1, Ordering::Relaxed),
                    _ => panic!("Unexpected port"),
                };
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let total = counter_80.load(Ordering::Relaxed) + counter_443.load(Ordering::Relaxed);
    assert_eq!(total, 1000);
    
    // Both ports should have been used
    assert!(counter_80.load(Ordering::Relaxed) > 0);
    assert!(counter_443.load(Ordering::Relaxed) > 0);
}

#[test]
fn test_simulation_creation_with_config() {
    let config = create_test_config();
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    let _simulation = Simulation::new(config, target_ip, None);
    
    // Simulation should be created successfully
    // (actual running would require tokio runtime)
}

#[test]
fn test_setup_network_interface_with_specific() {
    let mut config = create_test_config();
    
    // Try to set up with a specific interface that doesn't exist
    config.target.interface = Some("nonexistent_interface".to_string());
    
    let result = setup_network_interface(&config);
    assert!(result.is_err());
}

#[test]
fn test_setup_network_interface_default() {
    let config = create_test_config();
    
    // No specific interface, should try default
    let result = setup_network_interface(&config);
    
    // Result depends on system, but should not panic
    match result {
        Ok(Some(iface)) => {
            assert!(!iface.name.is_empty());
        }
        Ok(None) => {
            // No suitable interface found
        }
        Err(_) => {
            // Error finding interface
        }
    }
}

#[test]
fn test_multi_port_target_distribution() {
    let ports = vec![80, 443, 8080];
    let target = MultiPortTarget::new(ports.clone());
    
    let mut port_counts = std::collections::HashMap::new();
    let iterations = 30000;
    
    for _ in 0..iterations {
        let port = target.next_port();
        *port_counts.entry(port).or_insert(0) += 1;
    }
    
    // Each port should be used roughly equally
    let expected_per_port = iterations / ports.len();
    for port in &ports {
        let count = port_counts.get(port).unwrap_or(&0);
        let deviation = (*count as f64 - expected_per_port as f64).abs();
        let deviation_percent = deviation / expected_per_port as f64 * 100.0;
        
        // Allow 10% deviation
        assert!(
            deviation_percent < 10.0,
            "Port {} used {} times, expected ~{} (deviation: {:.1}%)",
            port, count, expected_per_port, deviation_percent
        );
    }
}

#[test]
fn test_multi_port_target_memory_safety() {
    // Test with extreme cases
    let _target1 = MultiPortTarget::new(vec![]);
    // Should handle empty gracefully (might panic or return default)
    
    let target2 = MultiPortTarget::new(vec![65535]); // Max port
    assert_eq!(target2.next_port(), 65535);
    
    let target3 = MultiPortTarget::new(vec![1]); // Min valid port
    assert_eq!(target3.next_port(), 1);
    
    // Large number of ports
    let large_ports: Vec<u16> = (1000..2000).collect();
    let target4 = MultiPortTarget::new(large_ports.clone());
    let port = target4.next_port();
    assert!(port >= 1000 && port < 2000);
}

