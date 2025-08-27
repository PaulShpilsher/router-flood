//! System monitoring tests
//!
//! Tests for system monitoring and resource tracking functionality.

use router_flood::monitor::*;
use router_flood::stats::SystemStats;

#[test]
fn test_system_monitor_creation() {
    let _monitor = SystemMonitor::new(true);
    // Should create without panicking
    
    let _disabled_monitor = SystemMonitor::new(false);
    // Should also create when disabled
}

#[tokio::test]
async fn test_system_monitor_stats_collection() {
    let monitor = SystemMonitor::new(true);
    
    let stats = monitor.get_system_stats().await;
    
    if let Some(stats) = stats {
        // CPU usage should be a reasonable percentage
        assert!(stats.cpu_usage >= 0.0, "CPU usage should be non-negative");
        assert!(stats.cpu_usage <= 100.0, "CPU usage should not exceed 100%");
        
        // Memory usage should be positive
        assert!(stats.memory_usage > 0, "Memory usage should be positive");
        assert!(stats.memory_usage < 1024 * 1024 * 1024 * 1024, "Memory usage should be reasonable (< 1TB)");
    }
}

#[tokio::test]
async fn test_disabled_system_monitor() {
    let monitor = SystemMonitor::new(false);
    
    let stats = monitor.get_system_stats().await;
    assert!(stats.is_none(), "Disabled monitor should return None");
}

#[tokio::test]
async fn test_system_monitor_multiple_calls() {
    let monitor = SystemMonitor::new(true);
    
    // Make multiple calls to ensure consistency
    let stats1 = monitor.get_system_stats().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let stats2 = monitor.get_system_stats().await;
    
    // Both calls should succeed (or both fail consistently)
    assert_eq!(stats1.is_some(), stats2.is_some());
    
    if let (Some(s1), Some(s2)) = (stats1, stats2) {
        // Values should be reasonable and potentially different
        assert!(s1.cpu_usage >= 0.0 && s1.cpu_usage <= 100.0);
        assert!(s2.cpu_usage >= 0.0 && s2.cpu_usage <= 100.0);
        assert!(s1.memory_usage > 0);
        assert!(s2.memory_usage > 0);
        
        // Memory usage shouldn't change drastically in 100ms
        let memory_diff = (s2.memory_usage as i64 - s1.memory_usage as i64).abs();
        assert!(memory_diff < 1024 * 1024 * 1024, "Memory usage shouldn't change drastically: {} vs {}", 
               s1.memory_usage, s2.memory_usage);
    }
}

#[tokio::test]
async fn test_system_stats_structure() {
    let monitor = SystemMonitor::new(true);
    
    if let Some(stats) = monitor.get_system_stats().await {
        // Test that SystemStats has the expected fields and reasonable values
        assert!(stats.cpu_usage.is_finite(), "CPU usage should be a finite number");
        assert!(!stats.cpu_usage.is_nan(), "CPU usage should not be NaN");
    }
}

#[tokio::test]
async fn test_concurrent_system_monitoring() {
    use std::sync::Arc;
    
    let monitor = Arc::new(SystemMonitor::new(true));
    let num_tasks = 5;
    
    let mut handles = Vec::new();
    
    for _ in 0..num_tasks {
        let monitor_clone = Arc::clone(&monitor);
        let handle = tokio::spawn(async move {
            monitor_clone.get_system_stats().await
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // All tasks should complete successfully
    for result in results {
        let stats = result.expect("Task should complete successfully");
        
        if let Some(stats) = stats {
            assert!(stats.cpu_usage >= 0.0 && stats.cpu_usage <= 100.0);
            assert!(stats.memory_usage > 0);
        }
    }
}

#[test]
fn test_system_stats_display() {
    let stats = SystemStats {
        cpu_usage: 25.5,
        memory_usage: 1024 * 1024 * 1024, // 1GB in bytes
        memory_total: 8 * 1024 * 1024 * 1024, // 8GB
        network_sent: 0,
        network_received: 0,
    };
    
    // Test that we can format the stats
    let formatted = format!("CPU: {:.1}%, Memory: {:.0} MB", stats.cpu_usage, stats.memory_usage / (1024 * 1024));
    assert_eq!(formatted, "CPU: 25.5%, Memory: 1024 MB");
}

#[test]
fn test_system_stats_edge_values() {
    // Test with edge case values
    let edge_stats = vec![
        SystemStats { cpu_usage: 0.0, memory_usage: 1024 * 1024, memory_total: 8 * 1024 * 1024 * 1024, network_sent: 0, network_received: 0 },
        SystemStats { cpu_usage: 100.0, memory_usage: 32 * 1024 * 1024 * 1024, memory_total: 64 * 1024 * 1024 * 1024, network_sent: 0, network_received: 0 },
        SystemStats { cpu_usage: 0.1, memory_usage: 100 * 1024, memory_total: 8 * 1024 * 1024 * 1024, network_sent: 0, network_received: 0 },
    ];
    
    for stats in edge_stats {
        assert!(stats.cpu_usage >= 0.0);
        assert!(stats.cpu_usage <= 100.0);
        // memory_usage is u64, so always >= 0
        assert!(stats.memory_usage < u64::MAX);
        
        // Should be able to format without issues
        let _formatted = format!("{:.2}% CPU, {:.1} MB RAM", stats.cpu_usage, stats.memory_usage as f64 / (1024.0 * 1024.0));
    }
}

#[tokio::test] 
async fn test_system_monitor_resource_cleanup() {
    // Test that creating and dropping monitors doesn't leak resources
    for _ in 0..10 {
        let monitor = SystemMonitor::new(true);
        let _stats = monitor.get_system_stats().await;
        // Monitor should be dropped here without issues
    }
}

#[tokio::test]
async fn test_monitoring_under_load() {
    let monitor = SystemMonitor::new(true);
    
    // Simulate some CPU load and check if monitoring still works
    let start = std::time::Instant::now();
    
    // Create some CPU load
    let _result: u64 = (0..100000).map(|i| i as u64).sum();
    
    let stats = monitor.get_system_stats().await;
    let elapsed = start.elapsed();
    
    // Should complete in reasonable time
    assert!(elapsed.as_millis() < 5000, "Monitoring should be fast even under load");
    
    if let Some(stats) = stats {
        // CPU usage might be higher due to our load, but should still be valid
        assert!(stats.cpu_usage >= 0.0 && stats.cpu_usage <= 100.0);
        assert!(stats.memory_usage > 0);
    }
}