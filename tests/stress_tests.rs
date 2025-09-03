//! Stress testing scenarios for router-flood

use router_flood::packet::{PacketBuilder, PacketType, PacketSizeRange};
use router_flood::config::{Config, ProtocolMix};
use router_flood::Stats;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr};

#[test]
#[ignore] // Run with: cargo test --test stress_tests -- --ignored
fn stress_test_concurrent_packet_generation() {
    let thread_count = 8;
    let packets_per_thread = 10000;
    let barrier = Arc::new(Barrier::new(thread_count + 1));
    let stats = Arc::new(Stats::new(None));
    
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            let barrier_clone = Arc::clone(&barrier);
            let stats_clone = Arc::clone(&stats);
            
            thread::spawn(move || {
                let protocol_mix = ProtocolMix::default();
                let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 1400), protocol_mix);
                let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, thread_id as u8));
                
                // Wait for all threads to be ready
                barrier_clone.wait();
                
                for i in 0..packets_per_thread {
                    let packet_type = match i % 4 {
                        0 => PacketType::Udp,
                        1 => PacketType::TcpSyn,
                        2 => PacketType::TcpAck,
                        _ => PacketType::Icmp,
                    };
                    
                    if let Ok((packet, _)) = builder.build_packet(packet_type, target_ip, 8080 + (i as u16)) {
                        stats_clone.increment_sent(packet.len() as u64, "udp");
                    }
                }
            })
        })
        .collect();
    
    let start = Instant::now();
    barrier.wait(); // Start all threads simultaneously
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_packets = thread_count * packets_per_thread;
    let packets_per_second = total_packets as f64 / duration.as_secs_f64();
    
    println!("Stress test completed:");
    println!("  Threads: {}", thread_count);
    println!("  Total packets: {}", total_packets);
    println!("  Duration: {:?}", duration);
    println!("  Packets/sec: {:.2}", packets_per_second);
    
    assert!(packets_per_second > 10000.0, "Performance too low");
}


#[test]
#[ignore]
fn stress_test_statistics_accuracy() {
    let stats = Arc::new(Stats::new(None));
    let thread_count = 10;
    let operations_per_thread = 100000;
    let barrier = Arc::new(Barrier::new(thread_count + 1));
    
    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let stats_clone = Arc::clone(&stats);
            let barrier_clone = Arc::clone(&barrier);
            
            thread::spawn(move || {
                barrier_clone.wait();
                
                for i in 0..operations_per_thread {
                    stats_clone.increment_sent((1000 + (i % 500)) as u64, "test");
                    if i % 100 == 0 {
                        stats_clone.increment_failed();
                    }
                }
            })
        })
        .collect();
    
    barrier.wait();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Get final stats
    let total_packets = stats.packets_sent();
    let total_errors = stats.packets_failed();
    let expected_packets = (thread_count * operations_per_thread) as u64;
    let expected_errors = (thread_count * operations_per_thread / 100) as u64;
    
    assert_eq!(total_packets, expected_packets, "Packet count mismatch");
    assert_eq!(total_errors, expected_errors, "Error count mismatch");
    
    println!("Statistics stress test passed:");
    println!("  Threads: {}", thread_count);
    println!("  Total packets: {}", total_packets);
    println!("  Total errors: {}", total_errors);
    println!("  Bytes sent: {}", stats.bytes_sent());
}

#[test]
#[ignore]
fn stress_test_rapid_config_changes() {
    let iterations = 1000;
    let mut configs = Vec::new();
    
    for i in 0..iterations {
        let mut config = Config::default();
        config.attack.threads = (i % 32) + 1;
        config.attack.packet_rate = 100.0 + (i as f64 * 10.0);
        config.attack.payload_size = 64 + (i % 1400);
        config.target.ports = vec![8000 + (i as u16), 9000 + (i as u16)];
        
        // Validate each config
        if let Ok(validated) = router_flood::config::validate_config(&config) {
            configs.push(validated);
        }
    }
    
    assert!(configs.len() > 900, "Too many configs failed validation");
    println!("Rapid config changes test:");
    println!("  Iterations: {}", iterations);
    println!("  Valid configs: {}", configs.len());
}

#[test]
#[ignore]
fn stress_test_packet_size_variations() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(20, 9000), protocol_mix); // Large range
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    let mut size_distribution = std::collections::HashMap::new();
    
    for _ in 0..10000 {
        if let Ok((packet, _)) = builder.build_packet(PacketType::Udp, target_ip, 8080) {
            let size_bucket = packet.len() / 100 * 100; // Round to nearest 100
            *size_distribution.entry(size_bucket).or_insert(0) += 1;
        }
    }
    
    println!("Packet size distribution:");
    let mut sizes: Vec<_> = size_distribution.keys().cloned().collect();
    sizes.sort();
    for size in sizes {
        println!("  {}-{} bytes: {} packets", 
                 size, size + 99, size_distribution[&size]);
    }
    
    // Verify we get a good distribution
    assert!(size_distribution.len() > 5, "Not enough size variation");
}

#[test]
#[ignore]
fn stress_test_sustained_high_throughput() {
    let duration = Duration::from_secs(10);
    let stats = Arc::new(Stats::new(None));
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 1400), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    let start = Instant::now();
    let mut packet_count = 0u64;
    
    while start.elapsed() < duration {
        for _ in 0..1000 {
            if let Ok((packet, _)) = builder.build_packet(PacketType::Udp, target_ip, 8080) {
                stats.increment_sent(packet.len() as u64, "udp");
            }
            packet_count += 1;
        }
    }
    
    let elapsed = start.elapsed();
    let packets_per_second = packet_count as f64 / elapsed.as_secs_f64();
    let mbps = (stats.bytes_sent() as f64 * 8.0) / (elapsed.as_secs_f64() * 1_000_000.0);
    
    println!("Sustained throughput test:");
    println!("  Duration: {:?}", elapsed);
    println!("  Packets sent: {}", packet_count);
    println!("  Packets/sec: {:.2}", packets_per_second);
    println!("  Throughput: {:.2} Mbps", mbps);
    
    assert!(packets_per_second > 50000.0, "Throughput too low");
}