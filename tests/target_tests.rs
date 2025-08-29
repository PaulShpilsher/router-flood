//! Target module tests

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unnecessary_cast)]
//!
//! Tests for multi-port target management and port selection logic.

use router_flood::core::target::*;
use std::collections::HashSet;

#[test]
fn test_multi_port_target_creation() {
    let ports = vec![80, 443, 22];
    let target = MultiPortTarget::new(ports.clone());
    
    // Should store ports correctly
    assert_eq!(target.get_ports(), ports);
}

#[test]
fn test_multi_port_target_empty_ports() {
    let target = MultiPortTarget::new(vec![]);
    assert!(target.get_ports().is_empty());
}

#[test]
fn test_multi_port_target_single_port() {
    let target = MultiPortTarget::new(vec![80]);
    assert_eq!(target.get_ports(), vec![80]);
    
    // Should always return the same port
    for _ in 0..10 {
        assert_eq!(target.next_port(), 80);
    }
}

#[test]
fn test_multi_port_target_port_rotation() {
    let ports = vec![80, 443, 22, 8080];
    let target = MultiPortTarget::new(ports.clone());
    
    let mut seen_ports = HashSet::new();
    
    // Get more ports than we have to ensure rotation
    for _ in 0..20 {
        let port = target.next_port();
        assert!(ports.contains(&port), "Returned port should be in original list");
        seen_ports.insert(port);
    }
    
    // Should have seen all ports due to rotation
    assert_eq!(seen_ports.len(), ports.len(), "Should rotate through all ports");
    for expected_port in ports {
        assert!(seen_ports.contains(&expected_port), "Should have seen port {}", expected_port);
    }
}

#[test]
fn test_multi_port_target_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    let ports = vec![80, 443, 22, 8080, 9000];
    let target = Arc::new(MultiPortTarget::new(ports.clone()));
    let num_threads = 10;
    let requests_per_thread = 100;
    
    let handles: Vec<_> = (0..num_threads).map(|_| {
        let target_clone = Arc::clone(&target);
        thread::spawn(move || {
            let mut thread_ports = Vec::new();
            for _ in 0..requests_per_thread {
                thread_ports.push(target_clone.next_port());
            }
            thread_ports
        })
    }).collect();
    
    let mut all_returned_ports = Vec::new();
    for handle in handles {
        let thread_ports = handle.join().unwrap();
        all_returned_ports.extend(thread_ports);
    }
    
    // Verify all returned ports are valid
    for port in &all_returned_ports {
        assert!(ports.contains(port), "All returned ports should be valid");
    }
    
    // Verify we got the expected number of ports
    assert_eq!(all_returned_ports.len(), num_threads * requests_per_thread);
    
    // Verify we saw all different ports (due to rotation)
    let unique_ports: HashSet<_> = all_returned_ports.into_iter().collect();
    assert_eq!(unique_ports.len(), ports.len(), "Should see all unique ports");
}

#[test]
fn test_multi_port_target_distribution() {
    let ports = vec![80, 443];
    let target = MultiPortTarget::new(ports);
    let num_requests = 1000;
    
    let mut port_counts = std::collections::HashMap::new();
    
    for _ in 0..num_requests {
        let port = target.next_port();
        *port_counts.entry(port).or_insert(0) += 1;
    }
    
    // Should have seen both ports
    assert_eq!(port_counts.len(), 2);
    assert!(port_counts.contains_key(&80));
    assert!(port_counts.contains_key(&443));
    
    // Distribution should be roughly even (allow for some variance)
    let count_80 = port_counts[&80];
    let count_443 = port_counts[&443];
    let expected = num_requests / 2;
    let tolerance = num_requests / 10; // 10% tolerance
    
    assert!((count_80 as i32 - expected as i32).abs() < tolerance as i32, 
           "Port 80 distribution should be roughly even: {} vs expected ~{}", count_80, expected);
    assert!((count_443 as i32 - expected as i32).abs() < tolerance as i32,
           "Port 443 distribution should be roughly even: {} vs expected ~{}", count_443, expected);
}

#[test]
fn test_multi_port_target_with_duplicate_ports() {
    let ports_with_duplicates = vec![80, 443, 80, 22, 443];
    let target = MultiPortTarget::new(ports_with_duplicates);
    
    // Should handle duplicates gracefully
    let returned_ports = target.get_ports();
    
    // Verify all returned ports are valid
    for _ in 0..20 {
        let port = target.next_port();
        assert!(returned_ports.contains(&port));
    }
}

#[test]
fn test_multi_port_target_edge_case_ports() {
    // Test with edge case port numbers
    let edge_ports = vec![1, 65535, 1024, 49152];
    let target = MultiPortTarget::new(edge_ports.clone());
    
    assert_eq!(target.get_ports(), edge_ports);
    
    // Should handle all edge case ports
    for _ in 0..20 {
        let port = target.next_port();
        assert!(edge_ports.contains(&port));
        assert!(port >= 1); // port is u16, so <= 65535 is always true
    }
}

#[test]
fn test_multi_port_target_large_port_list() {
    // Test with a large number of ports
    let large_port_list: Vec<u16> = (8000..9000).collect(); // 1000 ports
    let target = MultiPortTarget::new(large_port_list.clone());
    
    assert_eq!(target.get_ports().len(), 1000);
    
    let mut seen_ports = HashSet::new();
    
    // Get enough samples to likely see most ports
    for _ in 0..5000 {
        let port = target.next_port();
        assert!(large_port_list.contains(&port));
        seen_ports.insert(port);
    }
    
    // Should have seen a good portion of the ports
    assert!(seen_ports.len() > 900, "Should see most ports in large list");
}

#[test]
fn test_multi_port_target_consistency() {
    let ports = vec![80, 443, 22];
    let target = MultiPortTarget::new(ports.clone());
    
    // Test that get_ports() returns consistent results
    assert_eq!(target.get_ports(), ports);
    assert_eq!(target.get_ports(), ports);
    assert_eq!(target.get_ports(), target.get_ports());
    
    // The ports list should not change after creation
    for _ in 0..100 {
        target.next_port(); // This might change internal state
        assert_eq!(target.get_ports(), ports); // But this should stay the same
    }
}

#[test]
fn test_multi_port_target_concurrent_port_access() {
    use std::sync::Arc;
    use std::thread;
    use std::sync::Barrier;
    
    let ports = vec![80, 443, 22, 8080];
    let target = Arc::new(MultiPortTarget::new(ports.clone()));
    let num_threads = 50;
    let barrier = Arc::new(Barrier::new(num_threads));
    
    let handles: Vec<_> = (0..num_threads).map(|_| {
        let target_clone = Arc::clone(&target);
        let barrier_clone = Arc::clone(&barrier);
        thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            // All threads request a port at the same time
            target_clone.next_port()
        })
    }).collect();
    
    let mut results = Vec::new();
    for handle in handles {
        let port = handle.join().unwrap();
        results.push(port);
    }
    
    // All results should be valid ports
    for port in results {
        assert!(ports.contains(&port), "Concurrent access should return valid ports");
    }
}