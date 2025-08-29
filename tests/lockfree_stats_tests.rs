//! Comprehensive tests for lock-free statistics module

use router_flood::stats::lockfree::{
    LockFreeStats, LockFreeLocalStats, PerCpuStats, ProtocolId, StatsSnapshot
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_protocol_id_all_variants() {
    // Test all protocol conversions
    assert_eq!(ProtocolId::from_str("UDP"), Some(ProtocolId::Udp));
    assert_eq!(ProtocolId::from_str("TCP"), Some(ProtocolId::Tcp));
    assert_eq!(ProtocolId::from_str("ICMP"), Some(ProtocolId::Icmp));
    assert_eq!(ProtocolId::from_str("IPv6"), Some(ProtocolId::Ipv6));
    assert_eq!(ProtocolId::from_str("ARP"), Some(ProtocolId::Arp));
    assert_eq!(ProtocolId::from_str("INVALID"), None);
    
    // Test string representation
    assert_eq!(ProtocolId::Udp.as_str(), "UDP");
    assert_eq!(ProtocolId::Tcp.as_str(), "TCP");
    assert_eq!(ProtocolId::Icmp.as_str(), "ICMP");
    assert_eq!(ProtocolId::Ipv6.as_str(), "IPv6");
    assert_eq!(ProtocolId::Arp.as_str(), "ARP");
}

#[test]
fn test_lock_free_stats_concurrent_updates() {
    let stats = Arc::new(LockFreeStats::new());
    let num_threads = 10;
    let updates_per_thread = 1000;
    
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let stats_clone = stats.clone();
        let handle = thread::spawn(move || {
            for _ in 0..updates_per_thread {
                let protocol = match thread_id % 5 {
                    0 => ProtocolId::Udp,
                    1 => ProtocolId::Tcp,
                    2 => ProtocolId::Icmp,
                    3 => ProtocolId::Ipv6,
                    _ => ProtocolId::Arp,
                };
                stats_clone.increment_sent(100, protocol);
                if thread_id % 2 == 0 {
                    stats_clone.increment_failed();
                }
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let snapshot = stats.snapshot();
    assert_eq!(snapshot.packets_sent, (num_threads * updates_per_thread) as u64);
    assert_eq!(snapshot.packets_failed, (num_threads / 2 * updates_per_thread) as u64);
    assert_eq!(snapshot.bytes_sent, (num_threads * updates_per_thread * 100) as u64);
    
    // Check protocol distribution
    // Each protocol is used by 2 threads (10 threads / 5 protocols = 2)
    let threads_per_protocol = 2;
    let expected_per_protocol = (threads_per_protocol * updates_per_thread) as u64;
    for i in 0..ProtocolId::COUNT {
        assert_eq!(snapshot.protocol_counts[i], expected_per_protocol);
    }
}

#[test]
fn test_stats_snapshot_calculations() {
    let stats = LockFreeStats::new();
    
    // Add some data
    stats.increment_sent(1000, ProtocolId::Udp);
    stats.increment_sent(2000, ProtocolId::Tcp);
    stats.increment_failed();
    
    // Wait a bit to have meaningful elapsed time
    thread::sleep(Duration::from_millis(100));
    
    let snapshot = stats.snapshot();
    
    assert_eq!(snapshot.packets_sent, 2);
    assert_eq!(snapshot.packets_failed, 1);
    assert_eq!(snapshot.bytes_sent, 3000);
    assert!(snapshot.elapsed_secs > 0.0);
    
    // Check rate calculations
    let pps = snapshot.packets_per_second();
    let mbps = snapshot.megabits_per_second();
    
    assert!(pps > 0.0);
    assert!(mbps > 0.0);
}

#[test]
fn test_local_stats_flush_on_threshold() {
    let global = Arc::new(LockFreeStats::new());
    let mut local = LockFreeLocalStats::new(global.clone(), 5); // Batch size of 5
    
    // Add 4 packets - should not flush yet
    for _ in 0..4 {
        local.increment_sent(100, ProtocolId::Udp);
    }
    assert_eq!(global.snapshot().packets_sent, 0);
    
    // Add 5th packet - should trigger flush
    local.increment_sent(100, ProtocolId::Udp);
    assert_eq!(global.snapshot().packets_sent, 5);
    
    // Add more packets
    for _ in 0..3 {
        local.increment_failed();
    }
    assert_eq!(global.snapshot().packets_failed, 0);
    
    // Force flush
    local.flush();
    assert_eq!(global.snapshot().packets_failed, 3);
}

#[test]
fn test_local_stats_drop_flushes() {
    let global = Arc::new(LockFreeStats::new());
    
    {
        let mut local = LockFreeLocalStats::new(global.clone(), 100);
        local.increment_sent(500, ProtocolId::Tcp);
        local.increment_sent(500, ProtocolId::Udp);
        local.increment_failed();
        // local drops here, should flush
    }
    
    let snapshot = global.snapshot();
    assert_eq!(snapshot.packets_sent, 2);
    assert_eq!(snapshot.packets_failed, 1);
    assert_eq!(snapshot.bytes_sent, 1000);
}

#[test]
fn test_per_cpu_stats_distribution() {
    let per_cpu = PerCpuStats::new();
    
    // Get multiple local stats (simulating different threads)
    let stats1 = per_cpu.get_local();
    let stats2 = per_cpu.get_local();
    let stats3 = per_cpu.get_local();
    
    // Update different stats
    stats1.increment_sent(100, ProtocolId::Udp);
    stats2.increment_sent(200, ProtocolId::Tcp);
    stats3.increment_failed();
    
    // Aggregate all
    let aggregate = per_cpu.aggregate();
    assert_eq!(aggregate.packets_sent, 2);
    assert_eq!(aggregate.packets_failed, 1);
    assert_eq!(aggregate.bytes_sent, 300);
    assert_eq!(aggregate.protocol_counts[ProtocolId::Udp as usize], 1);
    assert_eq!(aggregate.protocol_counts[ProtocolId::Tcp as usize], 1);
}

#[test]
fn test_per_cpu_stats_concurrent_aggregation() {
    let per_cpu = Arc::new(PerCpuStats::new());
    let num_threads = 20;
    let updates_per_thread = 100;
    
    let mut handles = vec![];
    
    for _ in 0..num_threads {
        let per_cpu_clone = per_cpu.clone();
        let handle = thread::spawn(move || {
            let local = per_cpu_clone.get_local();
            for i in 0..updates_per_thread {
                let protocol = match i % 3 {
                    0 => ProtocolId::Udp,
                    1 => ProtocolId::Tcp,
                    _ => ProtocolId::Icmp,
                };
                local.increment_sent(50, protocol);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let aggregate = per_cpu.aggregate();
    assert_eq!(aggregate.packets_sent, (num_threads * updates_per_thread) as u64);
    assert_eq!(aggregate.bytes_sent, (num_threads * updates_per_thread * 50) as u64);
}

#[test]
fn test_stats_zero_elapsed_time() {
    let stats = LockFreeStats::new();
    let snapshot = stats.snapshot();
    
    // Even with zero elapsed time, methods should not panic
    assert_eq!(snapshot.packets_per_second(), 0.0);
    assert_eq!(snapshot.megabits_per_second(), 0.0);
}

#[test]
fn test_protocol_counts_independence() {
    let stats = LockFreeStats::new();
    
    // Increment different protocols
    stats.increment_sent(100, ProtocolId::Udp);
    stats.increment_sent(200, ProtocolId::Tcp);
    stats.increment_sent(300, ProtocolId::Icmp);
    stats.increment_sent(400, ProtocolId::Ipv6);
    stats.increment_sent(500, ProtocolId::Arp);
    
    let snapshot = stats.snapshot();
    
    // Verify each protocol was counted independently
    assert_eq!(snapshot.protocol_counts[ProtocolId::Udp as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Tcp as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Icmp as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Ipv6 as usize], 1);
    assert_eq!(snapshot.protocol_counts[ProtocolId::Arp as usize], 1);
    
    // Verify totals
    assert_eq!(snapshot.packets_sent, 5);
    assert_eq!(snapshot.bytes_sent, 1500);
}