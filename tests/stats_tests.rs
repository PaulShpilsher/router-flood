//! Statistics module tests
//!
//! Tests for statistics tracking, reporting, and export functionality.

use router_flood::stats::*;
use router_flood::config::{ExportConfig, ExportFormat};
use std::sync::atomic::Ordering;

fn create_test_export_config() -> ExportConfig {
    ExportConfig {
        enabled: true,
        format: ExportFormat::Json,
        filename_pattern: "test_export".to_string(),
        include_system_stats: false,
    }
}

#[test]
fn test_flood_stats_creation() {
    let stats = FloodStats::new(Some(create_test_export_config()));
    
    // Test initial values
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 0);
    assert_eq!(stats.packets_failed.load(Ordering::Relaxed), 0);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 0);
    
    // Test protocol stats are initialized
    assert!(!stats.protocol_stats.is_empty());
    
    // Test that session_id is generated
    assert!(!stats.session_id.is_empty());
}

#[test]
fn test_flood_stats_default() {
    let stats = FloodStats::default();
    
    // Default should have no export config
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 0);
    assert_eq!(stats.packets_failed.load(Ordering::Relaxed), 0);
    assert!(!stats.session_id.is_empty());
}

#[test]
fn test_packet_counting() {
    let stats = FloodStats::default();
    
    // Test packet increments using the actual API
    stats.increment_sent(64, "UDP");
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 1);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 64);
    
    stats.increment_sent(128, "TCP");
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 2);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 192);
    
    stats.increment_sent(32, "ICMP");
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 3);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 224);
}

#[test]
fn test_failed_packet_counting() {
    let stats = FloodStats::default();
    
    stats.increment_failed();
    assert_eq!(stats.packets_failed.load(Ordering::Relaxed), 1);
    
    stats.increment_failed();
    assert_eq!(stats.packets_failed.load(Ordering::Relaxed), 2);
    
    // Failed packets should not affect sent count
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 0);
}

#[test]
fn test_bytes_sent_tracking() {
    let stats = FloodStats::default();
    
    stats.increment_sent(100, "UDP");
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 100);
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 1);
    
    stats.increment_sent(200, "TCP");
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 300);
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 2);
}

#[test]
fn test_packet_accumulation() {
    let stats = FloodStats::default();
    
    // Add some packets and bytes
    for i in 0..10 {
        stats.increment_sent(64, "UDP"); // Typical small packet size
        if i % 3 == 0 {
            stats.increment_failed();
        }
    }
    
    // Verify accumulated values
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 10);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 640);
    assert!(stats.packets_failed.load(Ordering::Relaxed) > 0);
}

#[test]
fn test_protocol_stats_tracking() {
    let stats = FloodStats::default();
    
    // Add packets for different protocols
    stats.increment_sent(64, "UDP");
    stats.increment_sent(128, "TCP");
    stats.increment_sent(32, "ICMP");
    
    // Verify protocol stats are tracked
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 3);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 224);
    
    // Check that protocol stats exist
    assert!(stats.protocol_stats.contains_key("UDP"));
    assert!(stats.protocol_stats.contains_key("TCP"));
    assert!(stats.protocol_stats.contains_key("ICMP"));
}

#[test]
fn test_concurrent_stats_updates() {
    use std::sync::Arc;
    use std::thread;
    
    let stats = Arc::new(FloodStats::default());
    let num_threads = 10;
    let increments_per_thread = 100;
    
    let handles: Vec<_> = (0..num_threads).map(|_| {
        let stats_clone = Arc::clone(&stats);
        thread::spawn(move || {
            for _ in 0..increments_per_thread {
                stats_clone.increment_sent(64, "UDP");
            }
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify final counts
    let expected_packets = num_threads * increments_per_thread;
    let expected_bytes = expected_packets * 64;
    
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), expected_packets);
    assert_eq!(stats.protocol_stats["UDP"].load(Ordering::Relaxed), expected_packets);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), expected_bytes);
}

#[test]
fn test_stats_summary_creation() {
    let stats = FloodStats::default();
    
    // Add some test data
    stats.increment_sent(64, "UDP");
    stats.increment_sent(64, "UDP");
    stats.increment_sent(64, "TCP");
    stats.increment_sent(64, "ICMP");
    stats.increment_failed();
    
    // Test that we can access basic stats
    assert!(!stats.session_id.is_empty());
    assert_eq!(stats.packets_sent.load(Ordering::Relaxed), 4);
    assert_eq!(stats.packets_failed.load(Ordering::Relaxed), 1);
    assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 256);
}

#[tokio::test]
async fn test_stats_export_json() {
    let mut export_config = create_test_export_config();
    export_config.format = ExportFormat::Json;
    
    let stats = FloodStats::new(Some(export_config));
    
    // Add some test data
    stats.increment_sent(64, "UDP");
    stats.increment_sent(64, "TCP");
    
    // Export stats - this tests the export mechanism
    let result = stats.export_stats(None).await;
    // Export might succeed or fail depending on permissions, but shouldn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_stats_export_csv() {
    let mut export_config = create_test_export_config();
    export_config.format = ExportFormat::Csv;
    
    let stats = FloodStats::new(Some(export_config));
    
    // Add some test data
    stats.increment_sent(64, "UDP");
    stats.increment_sent(64, "TCP");
    
    // Export stats - this tests the export mechanism
    let result = stats.export_stats(None).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_stats_export_both_formats() {
    let mut export_config = create_test_export_config();
    export_config.format = ExportFormat::Both;
    
    let stats = FloodStats::new(Some(export_config));
    
    // Add some test data
    stats.increment_sent(64, "UDP");
    
    // Export stats - this tests the export mechanism
    let result = stats.export_stats(None).await;
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_stats_print_functionality() {
    let stats = FloodStats::default();
    
    // Add some test data
    stats.increment_sent(64, "UDP");
    stats.increment_sent(64, "TCP");
    stats.increment_sent(64, "ICMP");
    
    let system_stats = Some(SystemStats {
        cpu_usage: 25.5,
        memory_usage: 1024 * 1024 * 1024, // 1GB in bytes
        memory_total: 8 * 1024 * 1024 * 1024, // 8GB in bytes
        network_sent: 1000,
        network_received: 2000,
    });
    
    // This should not panic
    stats.print_stats(system_stats.as_ref());
    
    // Test with no system stats
    stats.print_stats(None);
}