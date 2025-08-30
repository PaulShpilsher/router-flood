//! Simple verification tests for performance improvements

#![allow(clippy::uninlined_format_args)]

use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ConfigBuilder;
use router_flood::performance::{LockFreeBufferPool, SharedBufferPool};
use std::net::IpAddr;

#[test]
fn test_optimized_packet_building_performance() {
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".to_string().parse().unwrap();
    
    // Test zero-copy packet building with inline optimizations
    let mut buffer = vec![0u8; 1500];
    let result = packet_builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        target_ip,
        80,
    );
    
    assert!(result.is_ok());
    let (size, protocol_name) = result.unwrap();
    assert!(size > 0);
    assert_eq!(protocol_name, "UDP");
    
    println!("✅ Performance: Inline optimized packet building works");
}

#[test]
fn test_optimized_config_builder_performance() {
    // Test the enhanced configuration builder
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(vec![80, 443, 8080])
        .threads(4)
        .packet_rate(100)
        .packet_size_range(64, 1400)
        .build();
    
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.target.ip, "192.168.1.1".to_string());
    assert_eq!(config.target.ports, vec![80, 443, 8080]);
    
    println!("✅ Performance: Enhanced configuration builder works");
}

#[test]
fn test_lock_free_buffer_pool_performance() {
    let pool = LockFreeBufferPool::new(1400, 10);
    
    // Test basic operations
    let buffer1 = pool.get_buffer();
    assert!(buffer1.is_some());
    assert_eq!(buffer1.as_ref().unwrap().len(), 1400);
    
    let buffer2 = pool.get_buffer();
    assert!(buffer2.is_some());
    
    // Return buffers
    pool.return_buffer(buffer1.unwrap());
    pool.return_buffer(buffer2.unwrap());
    
    // Should be able to get buffers again
    let buffer3 = pool.get_buffer();
    assert!(buffer3.is_some());
    
    println!("✅ Performance: Lock-free buffer pool works");
}

#[test]
fn test_shared_buffer_pool_performance() {
    let pool = SharedBufferPool::new(1024, 5);
    let pool_clone = pool.clone();
    
    let buffer = pool.get_buffer();
    assert!(buffer.is_some());
    assert_eq!(buffer.as_ref().unwrap().len(), 1024);
    
    pool_clone.return_buffer(buffer.unwrap());
    
    let buffer2 = pool.get_buffer();
    assert!(buffer2.is_some());
    
    println!("✅ Performance: Shared buffer pool works");
}

#[test]
fn test_protocol_selection_optimization() {
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 1.0, // 100% UDP for predictable testing
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".to_string().parse().unwrap();
    
    // With optimized protocol selection, should consistently select UDP
    for _ in 0..10 {
        let packet_type = packet_builder.next_packet_type_for_ip(target_ip);
        assert_eq!(packet_type, PacketType::Udp);
    }
    
    println!("✅ Performance: Optimized protocol selection works");
}

#[test]
fn test_const_optimizations() {
    // Test const function optimizations
    assert_eq!(PacketType::Udp.protocol_name(), "UDP");
    assert!(PacketType::Udp.is_ipv4());
    assert!(!PacketType::Udp.is_ipv6());
    
    assert_eq!(PacketType::Ipv6Udp.protocol_name(), "IPv6");
    assert!(!PacketType::Ipv6Udp.is_ipv4());
    assert!(PacketType::Ipv6Udp.is_ipv6());
    
    println!("✅ Performance: Const function optimizations work");
}

// Adapters have been removed as part of cleanup
// #[test]
// fn test_adapter_compatibility() {
//     // Adapters no longer needed after removing duplicate modules
//     println!("✅ Performance: Adapters removed - using unified types");
// }

#[test]
fn test_performance_constants() {
    use router_flood::performance::lookup_tables::lookup_tables::*;
    use router_flood::performance::lookup_tables::bit_ops::*;
    
    // Test lookup table optimizations
    let udp_index = packet_type_to_index(PacketType::Udp);
    let min_size = min_size_by_index(udp_index);
    assert_eq!(min_size, PacketType::Udp.min_packet_size());
    
    // Test bit operations
    assert!(is_power_of_2(8));
    assert!(!is_power_of_2(7));
    assert_eq!(fast_mod_pow2(15, 8), 7);
    assert_eq!(next_power_of_2(15), 16);
    
    println!("✅ Performance: Performance constants and bit ops work");
}

#[test]
fn test_performance_integration() {
    // Test that all performance components work together
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .threads(2)
        .packet_rate(50)
        .build()
        .expect("Config should be valid");
    
    let mut packet_builder = PacketBuilder::new(
        config.attack.packet_size_range,
        config.target.protocol_mix.clone(),
    );
    
    let pool = SharedBufferPool::new(1500, 10);
    let target_ip: IpAddr = config.target.ip.parse().unwrap();
    
    // Generate packets using the buffer pool
    for _ in 0..5 {
        if let Some(mut buffer) = pool.get_buffer() {
            let packet_type = packet_builder.next_packet_type_for_ip(target_ip);
            let result = packet_builder.build_packet_into_buffer(
                &mut buffer,
                packet_type,
                target_ip,
                80,
            );
            
            assert!(result.is_ok());
            pool.return_buffer(buffer);
        }
    }
    
    println!("✅ Performance: Full integration test passed");
}