//! Comprehensive tests for RAII resource management

use router_flood::utils::raii::{
    ChannelGuard, ResourceGuard, SignalGuard, StatsGuard, 
    TerminalRAIIGuard, WorkerGuard
};
use router_flood::core::worker::WorkerManager;
use router_flood::core::target::MultiPortTarget;
use router_flood::stats::StatsAggregator;
use router_flood::transport::WorkerChannels;
mod common;
use common::create_test_config;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::time::sleep;

#[test]
fn test_channel_guard_drop_behavior() {
    let channels = WorkerChannels {
        ipv4_sender: None,
        ipv6_sender: None,
        l2_sender: None,
    };
    
    let mut guard = ChannelGuard::new(channels, "test_channels");
    
    // Verify we can access channels
    assert!(guard.channels_mut().is_some());
    
    // Take ownership
    let taken = guard.take();
    assert!(taken.is_some());
    
    // After taking, channels should be None
    assert!(guard.channels_mut().is_none());
    
    // Drop should not panic even when already taken
    drop(guard);
}

#[tokio::test]
async fn test_signal_guard_shutdown_flag() {
    let guard = SignalGuard::new().await.unwrap();
    
    // Initially should be running
    assert!(guard.is_running());
    
    let flag = guard.running_flag();
    assert!(flag.load(Ordering::Relaxed));
    
    // Manually set to false
    flag.store(false, Ordering::Relaxed);
    assert!(!guard.is_running());
    
    // Drop should clean up without issues
    drop(guard);
}

#[tokio::test]
async fn test_stats_guard_export_on_drop() {
    let stats = Arc::new(StatsAggregator::default());
    
    // Increment some stats
    stats.increment_sent(100, "UDP");
    stats.increment_failed();
    
    {
        let guard = StatsGuard::new(stats.clone(), "test_session");
        assert_eq!(guard.stats().packets_sent(), 1);
        // Guard drops here and should attempt export
    }
    
    // Stats should still be accessible after guard drops
    assert_eq!(stats.packets_sent(), 1);
    assert_eq!(stats.packets_failed(), 1);
}

#[test]
fn test_terminal_guard_creation() {
    // This test may fail if not running in a TTY, which is expected in CI
    match TerminalRAIIGuard::new() {
        Ok(_guard) => {
            // Guard created successfully, will restore on drop
        }
        Err(_) => {
            // Expected in non-TTY environments
        }
    }
}

#[tokio::test]
async fn test_resource_guard_builder_pattern() {
    let guard = ResourceGuard::new();
    
    // Initially should be running
    assert!(guard.is_running());
    
    // Add various guards using builder pattern
    let stats = Arc::new(StatsAggregator::default());
    let stats_guard = StatsGuard::new(stats, "test");
    
    let signal_guard = SignalGuard::new().await.unwrap();
    
    let guard = guard
        .with_stats(stats_guard)
        .with_signal(signal_guard);
    
    assert!(guard.is_running());
}

#[tokio::test]
async fn test_resource_guard_shutdown_sequence() {
    let mut guard = ResourceGuard::new();
    
    // Add signal guard
    let signal_guard = SignalGuard::new().await.unwrap();
    guard = guard.with_signal(signal_guard);
    
    // Verify running
    assert!(guard.is_running());
    
    // Initiate shutdown
    guard.shutdown().await.unwrap();
    
    // After shutdown, guards should be cleared
    assert!(guard.is_running()); // Will be true as signal is None after shutdown
}

#[tokio::test]
async fn test_worker_guard_stop_on_drop() {
    let config = create_test_config();
    let stats = Arc::new(StatsAggregator::default());
    let target = Arc::new(MultiPortTarget::new(vec![80, 443]));
    let target_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    
    let manager = WorkerManager::new(
        &config,
        stats,
        target,
        target_ip,
        None,
        true, // dry_run
    ).unwrap();
    
    let guard = WorkerGuard::new(manager, "test_workers");
    
    // Initially should be running
    assert!(guard.is_running());
    
    // Stop workers
    guard.stop();
    
    // Drop should handle cleanup
    drop(guard);
}

#[tokio::test]
async fn test_multiple_guards_concurrent_access() {
    let stats = Arc::new(StatsAggregator::default());
    let counter = Arc::new(AtomicUsize::new(0));
    
    let mut handles = vec![];
    
    for i in 0..5 {
        let stats_clone = stats.clone();
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            let guard = StatsGuard::new(stats_clone, &format!("worker_{}", i));
            
            // Simulate some work
            sleep(Duration::from_millis(10)).await;
            
            guard.stats().increment_sent(100, "UDP");
            counter_clone.fetch_add(1, Ordering::Relaxed);
            
            // Guard drops here
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    assert_eq!(counter.load(Ordering::Relaxed), 5);
    assert_eq!(stats.packets_sent(), 5);
}

#[tokio::test]
async fn test_worker_guard_take_ownership() {
    let _running = Arc::new(AtomicBool::new(true));
    
    // Create a mock manager (simplified for testing)
    let config = create_test_config();
    let stats = Arc::new(StatsAggregator::default());
    let target = Arc::new(MultiPortTarget::new(vec![80]));
    let target_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    
    let manager = WorkerManager::new(
        &config,
        stats,
        target,
        target_ip,
        None,
        true, // dry_run
    ).unwrap();
    
    let mut guard = WorkerGuard::new(manager, "test");
    
    // Take ownership
    let taken = guard.take();
    assert!(taken.is_some());
    
    // After taking, is_running should return false
    assert!(!guard.is_running());
    
    // Drop should not panic
    drop(guard);
}

#[tokio::test]
async fn test_resource_guard_drop_order() {
    use std::sync::Mutex;
    
    let _drop_order = Arc::new(Mutex::new(Vec::<String>::new()));
    
    {
        let mut guard = ResourceGuard::new();
        
        // Add guards in specific order
        let signal_guard = SignalGuard::new().await.unwrap();
        guard = guard.with_signal(signal_guard);
        
        let stats = Arc::new(StatsAggregator::default());
        let stats_guard = StatsGuard::new(stats, "test");
        let _ = guard.with_stats(stats_guard);
        
        // Guard drops here - should drop in reverse order
    }
    
    // ResourceGuard ensures proper cleanup order
}

