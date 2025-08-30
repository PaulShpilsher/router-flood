//! Simplified property-based tests that compile correctly

use proptest::prelude::*;
use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ConfigBuilder;
use router_flood::performance::LockFreeBufferPool;
use std::net::Ipv4Addr;

// Simple property test strategies that work
prop_compose! {
    fn valid_ipv4_private()(
        c in 0u8..=255,
        d in 1u8..=254
    ) -> Ipv4Addr {
        Ipv4Addr::new(192, 168, c, d)
    }
}

proptest! {
    #[test]
    fn test_packet_building_robustness(
        packet_type in prop::sample::select(&[
            PacketType::Udp, PacketType::TcpSyn, PacketType::TcpAck, PacketType::Icmp
        ]),
        target_ip in valid_ipv4_private(),
        target_port in 1u16..=65535,
        buffer_size in 100usize..=2000
    ) {
        let protocol_mix = router_flood::config::ProtocolMix {
            udp_ratio: 0.6,
            tcp_syn_ratio: 0.25,
            tcp_ack_ratio: 0.05,
            icmp_ratio: 0.05,
            ipv6_ratio: 0.03,
            arp_ratio: 0.02,
        };
        
        let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
        let mut buffer = vec![0u8; buffer_size];
        
        // Should never panic, even with edge case inputs
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            std::net::IpAddr::V4(target_ip),
            target_port,
        );
        
        // If successful, packet size should be reasonable
        if let Ok((size, protocol_name)) = result {
            prop_assert!(size > 0);
            prop_assert!(size <= buffer_size);
            prop_assert!(!protocol_name.is_empty());
            
            // Verify protocol name matches packet type
            match packet_type {
                PacketType::Udp => prop_assert_eq!(protocol_name, "UDP"),
                PacketType::TcpSyn | PacketType::TcpAck => prop_assert_eq!(protocol_name, "TCP"),
                PacketType::Icmp => prop_assert_eq!(protocol_name, "ICMP"),
                _ => {}
            }
        }
    }
    
    #[test]
    fn test_config_validation_properties(
        threads in 1usize..=50,
        packet_rate in 1u64..=5000,
        min_size in 20usize..=500,
        max_size in 500usize..=1400,
        target_ip in valid_ipv4_private()
    ) {
        let result = ConfigBuilder::new()
            .target_ip(&target_ip)
            .threads(threads)
            .packet_rate(packet_rate)
            .packet_size_range(min_size, max_size)
            .build();
        
        // Valid inputs should always succeed
        prop_assert!(result.is_ok());
        
        if let Ok(config) = result {
            prop_assert_eq!(config.attack.threads, threads);
            prop_assert_eq!(config.attack.packet_rate, packet_rate);
            prop_assert_eq!(config.attack.packet_size_range, (min_size, max_size));
        }
    }
    
    #[test]
    fn test_buffer_pool_properties(
        buffer_size in 100usize..=2000,
        pool_size in 1usize..=20,
        operations in 1usize..=50
    ) {
        let pool = LockFreeBufferPool::new(buffer_size, pool_size);
        let mut buffers = Vec::new();
        
        // Get some buffers
        for _ in 0..operations.min(pool_size) {
            if let Some(buffer) = pool.get_buffer() {
                prop_assert_eq!(buffer.len(), buffer_size);
                buffers.push(buffer);
            }
        }
        
        // Return all buffers
        for buffer in buffers {
            pool.return_buffer(buffer);
        }
        
        // Should be able to get buffers again
        for _ in 0..operations.min(pool_size) {
            if let Some(buffer) = pool.get_buffer() {
                prop_assert_eq!(buffer.len(), buffer_size);
                pool.return_buffer(buffer);
            }
        }
    }
}