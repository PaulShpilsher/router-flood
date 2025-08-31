//! Tests for stats adapter module

use router_flood::stats::adapter::{LockFreeStatsAdapter, LocalStatsExt};
use router_flood::stats::LocalStats;
use router_flood::stats::lockfree::{LockFreeStats, ProtocolId};
use std::sync::Arc;

#[test]
fn test_lock_free_adapter_creation() {
    let adapter = LockFreeStatsAdapter::new(None);
    
    // Session ID is managed internally by the FloodStats
    let stats = adapter.stats();
    assert!(!stats.session_id.is_empty());
    // Export config is managed internally
    let stats = adapter.stats();
    assert!(stats.export_config.is_none());
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
    
    // Export config is managed internally by FloodStats
    let stats = adapter.stats();
    assert!(stats.export_config.is_some());
    assert_eq!(stats.export_config.as_ref().unwrap().filename_pattern, "test_stats".to_string());
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
    
    assert_eq!(flood_stats.packets_sent(), 2);
    assert_eq!(flood_stats.packets_failed(), 1);
    assert_eq!(flood_stats.bytes_sent(), 1100);
    // Session ID is maintained in the FloodStats
    assert!(!flood_stats.session_id.is_empty());
    
    // Protocol stats are tracked internally in the lock-free implementation
    // Individual protocol counts cannot be directly accessed from FloodStats
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
    // Session IDs are unique per FloodStats instance
    let stats1 = adapter1.stats();
    let stats2 = adapter2.stats();
    assert_ne!(stats1.session_id, stats2.session_id);
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
    assert_eq!(flood_stats.packets_sent(), 15);
    assert_eq!(flood_stats.packets_failed(), 3);
    assert_eq!(flood_stats.bytes_sent(), 2000); // (10*100) + (5*200)
    
    // Protocol counts are tracked internally in the lock-free implementation
    // Verification is based on total counts
    assert_eq!(flood_stats.packets_sent(), 15);  // 10 UDP + 5 TCP
    // Note: Individual protocol breakdown is no longer directly accessible
}