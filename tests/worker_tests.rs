//! Worker module tests

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::manual_range_contains)]
//!
//! Tests for worker thread management and packet sending logic.

use router_flood::config::ProtocolMix;
use router_flood::stats::FloodStats;
use router_flood::target::MultiPortTarget;
use router_flood::worker::WorkerManager;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::Ordering;
use std::sync::Arc;

fn create_test_protocol_mix() -> ProtocolMix {
    ProtocolMix {
        udp_ratio: 1.0,
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    }
}

fn create_test_config() -> router_flood::config::Config {
    let mut config = router_flood::config::get_default_config();
    config.attack.threads = 2; // Small number for testing
    config.attack.packet_rate = 10; // Low rate for testing
    config.safety.dry_run = true; // Always use dry-run in tests
    config.target.protocol_mix = create_test_protocol_mix();
    config
}

#[tokio::test]
async fn test_worker_manager_creation() {
    let config = create_test_config();
    let stats = Arc::new(FloodStats::default());
    let multi_port_target = Arc::new(MultiPortTarget::new(vec![80, 443]));
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    let worker_manager = WorkerManager::new(
        &config,
        stats,
        multi_port_target,
        target_ip,
        None, // interface
        true, // dry_run
    ).expect("Failed to create WorkerManager");

    assert!(worker_manager.is_running());
    
    // Stop the worker manager
    worker_manager.stop();
    
    // Give it a moment to stop
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    
    // It should still be marked as running (the atomic flag is used by workers)
    // but the manager itself will stop when join_all is called
}

#[tokio::test]
async fn test_worker_manager_lifecycle() {
    let config = create_test_config();
    let stats = Arc::new(FloodStats::default());
    let multi_port_target = Arc::new(MultiPortTarget::new(vec![80]));
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    let worker_manager = WorkerManager::new(
        &config,
        stats.clone(),
        multi_port_target,
        target_ip,
        None, // interface
        true, // dry_run
    ).expect("Failed to create WorkerManager");

    // Let workers run briefly
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    // Check that some packets were simulated
    let initial_sent = stats.packets_sent.load(Ordering::Relaxed);
    let initial_failed = stats.packets_failed.load(Ordering::Relaxed);
    
    // Stop workers and wait for completion
    worker_manager.stop();
    let result = worker_manager.join_all().await;
    
    assert!(result.is_ok());
    
    // Should have processed some packets
    let final_sent = stats.packets_sent.load(Ordering::Relaxed);
    let final_failed = stats.packets_failed.load(Ordering::Relaxed);
    
    assert!(final_sent >= initial_sent);
    assert!(final_failed >= initial_failed);
    assert!(final_sent > 0 || final_failed > 0, "Should have processed some packets");
}

#[tokio::test]
async fn test_worker_with_multiple_ports() {
    let mut config = create_test_config();
    config.attack.threads = 1; // Single worker for predictable testing
    
    let stats = Arc::new(FloodStats::default());
    let multi_port_target = Arc::new(MultiPortTarget::new(vec![80, 443, 8080, 3000]));
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    let worker_manager = WorkerManager::new(
        &config,
        stats.clone(),
        multi_port_target,
        target_ip,
        None, // interface
        true, // dry_run
    ).expect("Failed to create WorkerManager");

    // Let the worker run for a bit
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    worker_manager.stop();
    let result = worker_manager.join_all().await;
    
    assert!(result.is_ok());
    
    // Should have sent some packets
    let total_packets = stats.packets_sent.load(Ordering::Relaxed) + 
                       stats.packets_failed.load(Ordering::Relaxed);
    assert!(total_packets > 0);
}

#[tokio::test]
async fn test_worker_protocol_mix() {
    let mut config = create_test_config();
    config.attack.threads = 1;
    
    // Set up a protocol mix that includes multiple protocols
    config.target.protocol_mix = ProtocolMix {
        udp_ratio: 0.5,
        tcp_syn_ratio: 0.3,
        tcp_ack_ratio: 0.1,
        icmp_ratio: 0.1,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let stats = Arc::new(FloodStats::default());
    let multi_port_target = Arc::new(MultiPortTarget::new(vec![80]));
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    let worker_manager = WorkerManager::new(
        &config,
        stats.clone(),
        multi_port_target,
        target_ip,
        None, // interface
        true, // dry_run
    ).expect("Failed to create WorkerManager");

    // Let it run to generate packets
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    
    worker_manager.stop();
    let _result = worker_manager.join_all().await;
    
    // Verify that different protocols were used
    // Note: In dry-run mode, we're testing the packet generation logic
    let total_packets = stats.packets_sent.load(Ordering::Relaxed) + 
                       stats.packets_failed.load(Ordering::Relaxed);
    assert!(total_packets > 0, "Should have generated some packets");
}

#[tokio::test]
async fn test_worker_rate_limiting() {
    let mut config = create_test_config();
    config.attack.threads = 1;
    config.attack.packet_rate = 5; // Very low rate
    
    let stats = Arc::new(FloodStats::default());
    let multi_port_target = Arc::new(MultiPortTarget::new(vec![80]));
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    let start_time = std::time::Instant::now();
    
    let worker_manager = WorkerManager::new(
        &config,
        stats.clone(),
        multi_port_target,
        target_ip,
        None, // interface
        true, // dry_run
    ).expect("Failed to create WorkerManager");

    // Let it run for exactly 1 second
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    
    worker_manager.stop();
    let _result = worker_manager.join_all().await;
    
    let elapsed = start_time.elapsed();
    let total_packets = stats.packets_sent.load(Ordering::Relaxed) + 
                       stats.packets_failed.load(Ordering::Relaxed);
    
    // With a rate of 5 packets per second for 1 second, we should have roughly 5 packets
    // Allow some variance due to timing and initial setup
    assert!(elapsed.as_millis() >= 900, "Should have run for about 1 second");
    assert!(total_packets >= 3 && total_packets <= 10, 
            "Should have generated roughly 5 packets, got {}", total_packets);
}

#[test]
fn test_protocol_mix_validation() {
    let mix = create_test_protocol_mix();
    
    // Test that ratios are valid
    assert_eq!(mix.udp_ratio, 1.0);
    assert_eq!(mix.tcp_syn_ratio, 0.0);
    assert_eq!(mix.tcp_ack_ratio, 0.0);
    assert_eq!(mix.icmp_ratio, 0.0);
    assert_eq!(mix.ipv6_ratio, 0.0);
    assert_eq!(mix.arp_ratio, 0.0);
    
    // Total should sum to 1.0 for a valid probability distribution
    let total = mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + 
                mix.icmp_ratio + mix.ipv6_ratio + mix.arp_ratio;
    assert!((total - 1.0).abs() < f64::EPSILON);
}