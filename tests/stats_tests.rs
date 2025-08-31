//! Statistics module tests

use router_flood::stats::Stats;
use std::sync::Arc;
use std::thread;

#[test]
fn test_stats_creation() {
    let stats = Stats::new(None);
    assert_eq!(stats.packets_sent(), 0);
    assert_eq!(stats.bytes_sent(), 0);
    assert_eq!(stats.packets_failed(), 0);
}

#[test]
fn test_stats_increment() {
    let stats = Stats::new(None);
    
    stats.increment_sent(100, "UDP");
    assert_eq!(stats.packets_sent(), 1);
    assert_eq!(stats.bytes_sent(), 100);
    
    stats.increment_sent(200, "TCP");
    assert_eq!(stats.packets_sent(), 2);
    assert_eq!(stats.bytes_sent(), 300);
    
    stats.increment_failed();
    assert_eq!(stats.packets_failed(), 1);
}

#[test]
fn test_stats_reset() {
    let stats = Stats::new(None);
    
    stats.increment_sent(100, "UDP");
    stats.increment_sent(200, "TCP");
    stats.increment_failed();
    
    assert_eq!(stats.packets_sent(), 2);
    assert_eq!(stats.bytes_sent(), 300);
    assert_eq!(stats.packets_failed(), 1);
    
    stats.reset();
    
    assert_eq!(stats.packets_sent(), 0);
    assert_eq!(stats.bytes_sent(), 0);
    assert_eq!(stats.packets_failed(), 0);
}

#[test]
fn test_stats_thread_safety() {
    let stats = Arc::new(Stats::new(None));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let stats_clone = Arc::clone(&stats);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                stats_clone.increment_sent(64, "UDP");
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(stats.packets_sent(), 10000);
    assert_eq!(stats.bytes_sent(), 640000);
}