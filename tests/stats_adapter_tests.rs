//! Tests for stats adapter module

use router_flood::stats::adapter::{LockFreeStatsAdapter, LocalStatsExt};
use router_flood::stats::LocalStats;
use router_flood::stats::lockfree::{LockFreeStats, ProtocolId};
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[test]
fn test_lock_free_adapter_creation() {
    let adapter = LockFreeStatsAdapter::new(None);
    
    assert!(!adapter.session_id.is_empty());
    assert!(adapter.export_config.is_none());
}

#[test]
fn test_lock_free_adapter_with_export_config() {
    use router_flood::config::{ExportConfig, ExportFormat};
    
    let export_config = ExportConfig {
        enabled: true,
        format: ExportFormat::Json,
        filename_pattern: "test_stats".to_string(),
        include_system_stats: false,
    };
    
    let adapter = LockFreeStatsAdapter::new(Some(export_config.clone()));
    
    assert!(adapter.export_config.is_some());
    assert_eq!(adapter.export_config.as_ref().unwrap().filename_pattern, "test_stats");
}

#[test]
fn test_adapter_increment_operations() {
    let adapter = LockFreeStatsAdapter::new(None);
    
    // Test increment_sent
    adapter.increment_sent(100, "UDP");
    adapter.increment_sent(200, "TCP");
    adapter.increment_sent(300, "INVALID"); // Should be ignored
    
    // Test increment_failed
    adapter.increment_failed();
    
    let stats = adapter.inner();
    let snapshot = stats.snapshot();
    
    assert_eq!(snapshot.packets_sent, 2); // Only valid protocols counted
    assert_eq!(snapshot.packets_failed, 1);
    assert_eq!(snapshot.bytes_sent, 300); // 100 + 200
}

#[test]
fn test_adapter_to_flood_stats_conversion() {
    let adapter = LockFreeStatsAdapter::new(None);
    
    // Add some data
    adapter.increment_sent(500, "UDP");
    adapter.increment_sent(600, "TCP");
    adapter.increment_failed();
    
    // Convert to FloodStats
    let flood_stats = adapter.to_flood_stats();
    
    assert_eq!(flood_stats.packets_sent.load(Ordering::Relaxed), 2);
    assert_eq!(flood_stats.packets_failed.load(Ordering::Relaxed), 1);
    assert_eq!(flood_stats.bytes_sent.load(Ordering::Relaxed), 1100);
    assert_eq!(flood_stats.session_id, adapter.session_id);
    
    // Check protocol stats
    let udp_count = flood_stats.protocol_stats
        .get("UDP")
        .unwrap()
        .load(Ordering::Relaxed);
    assert_eq!(udp_count, 1);
    
    let tcp_count = flood_stats.protocol_stats
        .get("TCP")
        .unwrap()
        .load(Ordering::Relaxed);
    assert_eq!(tcp_count, 1);
}

#[test]
fn test_local_stats_with_lock_free() {
    let lock_free = Arc::new(LockFreeStats::new());
    let local_stats = LocalStats::with_lock_free(lock_free.clone(), 10);
    
    // LocalStats should work with lock-free backend
    // Note: We can't directly test much here without exposing internals
    // but we verify it creates successfully
    drop(local_stats);
    
    // After drop, stats should be flushed
    let snapshot = lock_free.snapshot();
    assert_eq!(snapshot.packets_sent, 0); // No data was added
}

#[test]
fn test_adapter_protocol_mapping() {
    let adapter = LockFreeStatsAdapter::new(None);
    
    // Test all valid protocols
    adapter.increment_sent(100, "UDP");
    adapter.increment_sent(100, "TCP");
    adapter.increment_sent(100, "ICMP");
    adapter.increment_sent(100, "IPv6");
    adapter.increment_sent(100, "ARP");
    
    let snapshot = adapter.inner().snapshot();
    
    assert_eq!(snapshot.protocol_counts[ProtocolId::Udp as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Tcp as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Icmp as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Ipv6 as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Arp as usize], 1);
}

#[test]
fn test_adapter_concurrent_access() {
    let adapter = Arc::new(LockFreeStatsAdapter::new(None));
    let num_threads = 10;
    let updates_per_thread = 100;
    
    let mut handles = vec![];
    
    for i in 0..num_threads {
        let adapter_clone = adapter.clone();
        let handle = std::thread::spawn(move || {
            for _ in 0..updates_per_thread {
                if i % 2 == 0 {
                    adapter_clone.increment_sent(50, "UDP");
                } else {
                    adapter_clone.increment_sent(50, "TCP");
                }
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let snapshot = adapter.inner().snapshot();
    assert_eq!(snapshot.packets_sent, (num_threads * updates_per_thread) as u64);
    assert_eq!(snapshot.bytes_sent, (num_threads * updates_per_thread * 50) as u64);
}

#[test]
fn test_adapter_session_id_uniqueness() {
    let adapter1 = LockFreeStatsAdapter::new(None);
    let adapter2 = LockFreeStatsAdapter::new(None);
    
    // Session IDs should be unique
    assert_ne!(adapter1.session_id, adapter2.session_id);
}

#[test]
fn test_flood_stats_conversion_preserves_data() {
    let adapter = LockFreeStatsAdapter::new(None);
    
    // Add various data
    for _ in 0..10 {
        adapter.increment_sent(100, "UDP");
    }
    for _ in 0..5 {
        adapter.increment_sent(200, "TCP");
    }
    for _ in 0..3 {
        adapter.increment_failed();
    }
    
    let flood_stats = adapter.to_flood_stats();
    
    // Verify all data is preserved
    assert_eq!(flood_stats.packets_sent.load(Ordering::Relaxed), 15);
    assert_eq!(flood_stats.packets_failed.load(Ordering::Relaxed), 3);
    assert_eq!(flood_stats.bytes_sent.load(Ordering::Relaxed), 2000); // (10*100) + (5*200)
    
    // Verify protocol counts
    assert_eq!(
        flood_stats.protocol_stats.get("UDP").unwrap().load(Ordering::Relaxed),
        10
    );
    assert_eq!(
        flood_stats.protocol_stats.get("TCP").unwrap().load(Ordering::Relaxed),
        5
    );
}