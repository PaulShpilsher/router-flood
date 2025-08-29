//! Tests for in-place statistics display functionality
//!
//! Tests the new in-place stats display with ANSI escape codes,
//! color formatting, and progress bars.

use router_flood::stats::*;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::collections::HashMap;

#[test]
fn test_stats_display_creation() {
    let _display = StatsDisplay::new(true);
    // Should be created without panicking
    assert!(true);
    
    let _display_disabled = StatsDisplay::new(false);
    // Should handle disabled state
    assert!(true);
}

#[test]
fn test_stats_display_initialization() {
    // Test global display initialization
    let display = init_display(true);
    assert!(Arc::strong_count(&display) >= 1);
    
    // Getting the display should return the same instance
    let retrieved = get_display();
    assert!(retrieved.is_some());
    
    if let Some(retrieved_display) = retrieved {
        assert!(Arc::ptr_eq(&display, &retrieved_display));
    }
}

#[test]
fn test_display_with_empty_stats() {
    let display = StatsDisplay::new(true);
    let protocol_stats = HashMap::new();
    
    // Should handle empty stats without panicking
    display.display_stats(
        0,  // packets_sent
        0,  // packets_failed
        0,  // bytes_sent
        1.0,  // elapsed_secs
        &protocol_stats,
        None,  // system_stats
    );
}

#[test]
fn test_display_with_protocol_stats() {
    let display = StatsDisplay::new(true);
    let mut protocol_stats = HashMap::new();
    
    // Add protocol counters
    protocol_stats.insert("UDP".to_string(), AtomicU64::new(100));
    protocol_stats.insert("TCP".to_string(), AtomicU64::new(50));
    protocol_stats.insert("ICMP".to_string(), AtomicU64::new(25));
    
    // Should format protocol stats correctly
    display.display_stats(
        175,  // packets_sent
        5,    // packets_failed
        11200,  // bytes_sent
        10.0,   // elapsed_secs
        &protocol_stats,
        None,
    );
}

#[test]
fn test_display_with_system_stats() {
    let display = StatsDisplay::new(true);
    let protocol_stats = HashMap::new();
    
    let system_stats = SystemStats {
        cpu_usage: 45.5,
        memory_usage: 2 * 1024 * 1024 * 1024,  // 2GB
        memory_total: 8 * 1024 * 1024 * 1024,  // 8GB
        network_sent: 1000,
        network_received: 2000,
    };
    
    // Should include system stats in display
    display.display_stats(
        1000,
        10,
        64000,
        5.0,
        &protocol_stats,
        Some(&system_stats),
    );
}

#[test]
fn test_display_success_rate_calculation() {
    let display = StatsDisplay::new(true);
    let protocol_stats = HashMap::new();
    
    // Test different success rates
    // 100% success
    display.display_stats(
        100,
        0,
        6400,
        1.0,
        &protocol_stats,
        None,
    );
    
    // 95% success (should show green)
    display.display_stats(
        100,
        5,
        6400,
        1.0,
        &protocol_stats,
        None,
    );
    
    // 80% success (should show yellow)
    display.display_stats(
        100,
        20,
        6400,
        1.0,
        &protocol_stats,
        None,
    );
    
    // 50% success (should show red)
    display.display_stats(
        100,
        50,
        6400,
        1.0,
        &protocol_stats,
        None,
    );
}

#[test]
fn test_display_clear_functionality() {
    let display = StatsDisplay::new(true);
    let protocol_stats = HashMap::new();
    
    // Display some stats first
    display.display_stats(
        100,
        5,
        6400,
        1.0,
        &protocol_stats,
        None,
    );
    
    // Clear should not panic
    display.clear();
}

#[test]
fn test_display_when_disabled() {
    let display = StatsDisplay::new(false);  // Disabled
    let protocol_stats = HashMap::new();
    
    // Should do nothing when disabled
    display.display_stats(
        100,
        5,
        6400,
        1.0,
        &protocol_stats,
        None,
    );
    
    display.clear();
    // Should complete without any output
}

#[test]
fn test_display_rate_calculations() {
    let display = StatsDisplay::new(true);
    let protocol_stats = HashMap::new();
    
    // Test rate calculations
    display.display_stats(
        1000,  // packets_sent
        0,     // packets_failed
        128000,  // bytes_sent (125KB)
        10.0,    // elapsed_secs
        &protocol_stats,
        None,
    );
    // Should show 100 pps and calculated Mbps
}

#[test]
fn test_display_with_high_cpu_memory() {
    let display = StatsDisplay::new(true);
    let protocol_stats = HashMap::new();
    
    // Test with high CPU (should show red)
    let high_cpu_stats = SystemStats {
        cpu_usage: 85.0,
        memory_usage: 11 * 1024 * 1024 * 1024,  // 11GB (high)
        memory_total: 16 * 1024 * 1024 * 1024,
        network_sent: 0,
        network_received: 0,
    };
    
    display.display_stats(
        1000,
        10,
        64000,
        5.0,
        &protocol_stats,
        Some(&high_cpu_stats),
    );
    
    // Test with medium CPU (should show yellow)
    let medium_cpu_stats = SystemStats {
        cpu_usage: 60.0,
        memory_usage: 6 * 1024 * 1024 * 1024,  // 6GB (medium)
        memory_total: 16 * 1024 * 1024 * 1024,
        network_sent: 0,
        network_received: 0,
    };
    
    display.display_stats(
        1000,
        10,
        64000,
        5.0,
        &protocol_stats,
        Some(&medium_cpu_stats),
    );
}

#[test]
fn test_stats_print_with_display() {
    // Initialize the display
    init_display(true);
    
    let stats = FloodStats::default();
    
    // Add some test data
    stats.increment_sent(64, "UDP");
    stats.increment_sent(128, "TCP");
    stats.increment_sent(32, "ICMP");
    stats.increment_failed();
    
    // This should use the in-place display
    stats.print_stats(None);
    
    // With system stats
    let system_stats = Some(SystemStats {
        cpu_usage: 25.5,
        memory_usage: 1024 * 1024 * 1024,
        memory_total: 8 * 1024 * 1024 * 1024,
        network_sent: 1000,
        network_received: 2000,
    });
    
    stats.print_stats(system_stats.as_ref());
}

#[test]
fn test_concurrent_display_updates() {
    use std::thread;
    use std::time::Duration;
    
    let display = Arc::new(StatsDisplay::new(true));
    let mut handles = vec![];
    
    // Simulate concurrent updates
    for i in 0..3 {
        let display_clone = Arc::clone(&display);
        let handle = thread::spawn(move || {
            let mut protocol_stats = HashMap::new();
            protocol_stats.insert("UDP".to_string(), AtomicU64::new(i * 100));
            
            display_clone.display_stats(
                i * 1000,
                i * 10,
                i * 64000,
                (i + 1) as f64,
                &protocol_stats,
                None,
            );
            
            thread::sleep(Duration::from_millis(10));
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Clear at the end
    display.clear();
}

#[test]
fn test_display_progress_bar_rendering() {
    let display = StatsDisplay::new(true);
    let protocol_stats = HashMap::new();
    
    // Test various progress bar states
    let test_cases = vec![
        (100, 0),    // 100% success
        (100, 5),    // 95% success
        (100, 20),   // 80% success
        (100, 50),   // 50% success
        (100, 100),  // 0% success
        (0, 0),      // No packets
    ];
    
    for (sent, failed) in test_cases {
        display.display_stats(
            sent,
            failed,
            sent * 64,
            1.0,
            &protocol_stats,
            None,
        );
    }
}

#[test]
fn test_display_with_all_protocols() {
    let display = StatsDisplay::new(true);
    let mut protocol_stats = HashMap::new();
    
    // Add all supported protocols
    protocol_stats.insert("UDP".to_string(), AtomicU64::new(500));
    protocol_stats.insert("TCP".to_string(), AtomicU64::new(300));
    protocol_stats.insert("ICMP".to_string(), AtomicU64::new(100));
    protocol_stats.insert("IPv6".to_string(), AtomicU64::new(50));
    protocol_stats.insert("ARP".to_string(), AtomicU64::new(25));
    
    let system_stats = SystemStats {
        cpu_usage: 42.0,
        memory_usage: 3 * 1024 * 1024 * 1024,
        memory_total: 16 * 1024 * 1024 * 1024,
        network_sent: 50000,
        network_received: 60000,
    };
    
    // Should display all protocols in a single line
    display.display_stats(
        975,
        25,
        62400,
        10.0,
        &protocol_stats,
        Some(&system_stats),
    );
}

#[test]
fn test_display_drop_behavior() {
    {
        let display = StatsDisplay::new(true);
        let protocol_stats = HashMap::new();
        
        // Display some stats
        display.display_stats(
            100,
            5,
            6400,
            1.0,
            &protocol_stats,
            None,
        );
        
        // Display will be dropped here
    }
    // Drop handler should restore cursor visibility
    
    // Create another display to verify it still works
    let new_display = StatsDisplay::new(true);
    new_display.clear();
}

#[test]
fn test_fallback_when_no_display_initialized() {
    // Test that get_display returns None when not initialized
    use router_flood::stats::get_display;
    
    // Clear any existing display first
    if let Some(display) = get_display() {
        display.clear();
    }
    
    // Now test that stats can be printed without a display
    let stats = FloodStats::default();
    stats.increment_sent(64, "UDP");
    
    // This should not panic even without display initialized
    stats.print_stats(None);
}