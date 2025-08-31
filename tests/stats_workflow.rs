//! Statistics collection workflow integration tests

use router_flood::stats::Stats;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_stats_lifecycle() {
    let stats = Stats::new(None);
    
    // Initial state
    assert_eq!(stats.packets_sent(), 0);
    assert_eq!(stats.bytes_sent(), 0);
    assert_eq!(stats.packets_failed(), 0);
    
    // Increment phase
    for i in 1..=100 {
        stats.increment_sent(64, "UDP");
        assert_eq!(stats.packets_sent(), i);
        assert_eq!(stats.bytes_sent(), 64 * i);
    }
    
    // Add failures
    for i in 1..=10 {
        stats.increment_failed();
        assert_eq!(stats.packets_failed(), i);
    }
    
    // Reset phase
    stats.reset();
    assert_eq!(stats.packets_sent(), 0);
    assert_eq!(stats.bytes_sent(), 0);
    assert_eq!(stats.packets_failed(), 0);
}

#[test]
fn test_concurrent_stats_collection() {
    let stats = Arc::new(Stats::new(None));
    let mut handles = vec![];
    
    // Spawn multiple threads that update stats
    for thread_id in 0..10 {
        let stats_clone = Arc::clone(&stats);
        let handle = thread::spawn(move || {
            for packet_num in 0..100 {
                let protocol = if thread_id % 2 == 0 { "UDP" } else { "TCP" };
                let bytes = 64 + (packet_num % 100) as u64;
                stats_clone.increment_sent(bytes, protocol);
                
                // Occasionally fail
                if packet_num % 10 == 0 {
                    stats_clone.increment_failed();
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify totals
    assert_eq!(stats.packets_sent(), 1000); // 10 threads * 100 packets
    assert_eq!(stats.packets_failed(), 100); // 10 threads * 10 failures each
    
    // Bytes should be > 64000 (minimum) and < 164000 (maximum)
    let bytes = stats.bytes_sent();
    assert!(bytes >= 64000);
    assert!(bytes <= 164000);
}

#[test]
fn test_stats_with_different_protocols() {
    let stats = Stats::new(None);
    
    // Send packets with different protocols
    stats.increment_sent(100, "UDP");
    stats.increment_sent(200, "TCP");
    stats.increment_sent(150, "ICMP");
    stats.increment_sent(300, "IPv6");
    
    // Verify totals
    assert_eq!(stats.packets_sent(), 4);
    assert_eq!(stats.bytes_sent(), 750);
}

#[test]
fn test_stats_rate_calculation() {
    let stats = Stats::new(None);
    
    // Add packets over time
    let start = std::time::Instant::now();
    for _ in 0..100 {
        stats.increment_sent(1000, "UDP");
        thread::sleep(Duration::from_millis(1));
    }
    let elapsed = start.elapsed();
    
    // Calculate rate
    let packets = stats.packets_sent() as f64;
    let seconds = elapsed.as_secs_f64();
    let rate = packets / seconds;
    
    // Rate should be reasonable (not infinite or zero)
    assert!(rate > 0.0);
    assert!(rate < 100000.0); // Less than 100k pps
}

#[test]
fn test_stats_overflow_handling() {
    let stats = Stats::new(None);
    
    // Try to cause overflow with large values
    for _ in 0..1000 {
        stats.increment_sent(u64::MAX / 1000, "UDP");
    }
    
    // Should not panic, values should be reasonable
    let packets = stats.packets_sent();
    let bytes = stats.bytes_sent();
    
    assert_eq!(packets, 1000);
    // Bytes will overflow but should handle gracefully
    assert!(bytes > 0);
}

#[test]
fn test_stats_export_configuration() {
    use router_flood::config::Export;
    
    // Create stats with export config
    let export_config = Export {
        enabled: true,
        format: router_flood::config::ExportFormat::Json,
        path: "/tmp/stats".to_string(),
        interval_seconds: 60,
        include_system_stats: true,
    };
    
    let stats = Stats::new(Some(export_config.clone()));
    
    // Verify export config is stored
    assert!(stats.export_config.is_some());
    let stored_config = stats.export_config.as_ref().unwrap();
    assert!(stored_config.enabled);
    assert_eq!(stored_config.path, "/tmp/stats");
}

#[test]
fn test_stats_session_tracking() {
    let stats = Stats::new(None);
    
    // Session ID should be unique
    let session_id = &stats.session_id;
    assert!(session_id.starts_with("session_"));
    
    // Start time should be set
    let elapsed = stats.start_time.elapsed();
    assert!(elapsed.as_secs() < 10); // Should be recent
}