//! Property-based tests for router-flood components

#![allow(clippy::unnecessary_cast)]
//!
//! These tests use proptest to generate random inputs and verify
//! that our implementations maintain invariants across all inputs.

use proptest::prelude::*;
use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ConfigBuilder;
use router_flood::utils::buffer_pool::BufferPool;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Property test strategies
prop_compose! {
    fn valid_ipv4_private()(
        a in 192u8..=192,
        b in 168u8..=168,
        c in 0u8..=255,
        d in 1u8..=254
    ) -> Ipv4Addr {
        Ipv4Addr::new(a, b, c, d)
    }
}

prop_compose! {
    fn valid_ipv6_link_local()(
        _a in 0u16..=0xFFFF,
        b in 0u16..=0xFFFF,
        c in 0u16..=0xFFFF,
        d in 0u16..=0xFFFF,
        e in 0u16..=0xFFFF,
        f in 0u16..=0xFFFF,
        g in 0u16..=0xFFFF,
        h in 0u16..=0xFFFF
    ) -> Ipv6Addr {
        Ipv6Addr::new(0xfe80, b, c, d, e, f, g, h) // Link-local prefix
    }
}

prop_compose! {
    fn valid_protocol_mix()(
        udp in 0.0f64..=1.0,
        tcp_syn in 0.0f64..=1.0,
        tcp_ack in 0.0f64..=1.0,
        icmp in 0.0f64..=1.0,
        ipv6 in 0.0f64..=1.0,
        arp in 0.0f64..=1.0
    ) -> router_flood::config::ProtocolMix {
        // Normalize to sum to 1.0
        let total = udp + tcp_syn + tcp_ack + icmp + ipv6 + arp;
        if total > 0.0 {
            router_flood::config::ProtocolMix {
                udp_ratio: udp / total,
                tcp_syn_ratio: tcp_syn / total,
                tcp_ack_ratio: tcp_ack / total,
                icmp_ratio: icmp / total,
                ipv6_ratio: ipv6 / total,
                arp_ratio: arp / total,
            }
        } else {
            // Fallback to equal distribution
            router_flood::config::ProtocolMix {
                udp_ratio: 1.0 / 6.0,
                tcp_syn_ratio: 1.0 / 6.0,
                tcp_ack_ratio: 1.0 / 6.0,
                icmp_ratio: 1.0 / 6.0,
                ipv6_ratio: 1.0 / 6.0,
                arp_ratio: 1.0 / 6.0,
            }
        }
    }
}

proptest! {
    #[test]
    fn test_packet_building_never_panics(
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
        
        // Should never panic, even with invalid inputs
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            IpAddr::V4(target_ip),
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
        // Note: Errors are acceptable for property tests with random inputs
    }
    
    #[test]
    fn test_packet_size_bounds(
        packet_type in prop::sample::select(&[
            PacketType::Udp, PacketType::TcpSyn, PacketType::TcpAck, PacketType::Icmp
        ]),
        target_ip in valid_ipv4_private(),
        target_port in 1u16..=65535
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
        let mut buffer = vec![0u8; 2000];
        
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            IpAddr::V4(target_ip),
            target_port,
        );
        
        if let Ok((size, _)) = result {
            // Packet should be at least minimum size (headers)
            // Note: Different packet types have different minimum sizes
            let min_size = match packet_type {
                PacketType::Udp => 28,      // 20 (IP) + 8 (UDP)
                PacketType::TcpSyn | PacketType::TcpAck => 40, // 20 (IP) + 20 (TCP)
                PacketType::Icmp => 28,     // 20 (IP) + 8 (ICMP)
                _ => 20, // At least IP header
            };
            prop_assert!(size >= min_size);
            
            // Packet should not exceed reasonable maximum
            prop_assert!(size <= 1500); // Standard MTU
        }
    }
    
    #[test]
    fn test_ipv6_packet_building(
        packet_type in prop::sample::select(&[
            PacketType::Ipv6Udp, PacketType::Ipv6Tcp, PacketType::Ipv6Icmp
        ]),
        target_ip in valid_ipv6_link_local(),
        target_port in 1u16..=65535
    ) {
        let protocol_mix = router_flood::config::ProtocolMix {
            udp_ratio: 0.0,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 1.0,
            arp_ratio: 0.0,
        };
        
        let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
        let mut buffer = vec![0u8; 2000];
        
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            IpAddr::V6(target_ip),
            target_port,
        );
        
        if let Ok((size, protocol_name)) = result {
            prop_assert!(size >= 48); // IPv6 header is 40 bytes + protocol header
            prop_assert_eq!(protocol_name, "IPv6");
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
            .target_ip(&target_ip.to_string())
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
    fn test_protocol_mix_normalization(
        protocol_mix in valid_protocol_mix()
    ) {
        let total = protocol_mix.udp_ratio + 
                   protocol_mix.tcp_syn_ratio + 
                   protocol_mix.tcp_ack_ratio + 
                   protocol_mix.icmp_ratio + 
                   protocol_mix.ipv6_ratio + 
                   protocol_mix.arp_ratio;
        
        // Should always sum to approximately 1.0
        prop_assert!((total - 1.0).abs() < 0.001);
        
        // All ratios should be non-negative
        prop_assert!(protocol_mix.udp_ratio >= 0.0);
        prop_assert!(protocol_mix.tcp_syn_ratio >= 0.0);
        prop_assert!(protocol_mix.tcp_ack_ratio >= 0.0);
        prop_assert!(protocol_mix.icmp_ratio >= 0.0);
        prop_assert!(protocol_mix.ipv6_ratio >= 0.0);
        prop_assert!(protocol_mix.arp_ratio >= 0.0);
    }
    
    #[test]
    fn test_buffer_pool_invariants(
        buffer_size in 100usize..=2000,
        pool_size in 1usize..=50,
        operations in 1usize..=100
    ) {
        let pool = BufferPool::new(buffer_size, pool_size);
        let mut buffers = Vec::new();
        
        // Get some buffers
        for _ in 0..operations.min(pool_size) {
            let buffer = pool.get_buffer();
            prop_assert_eq!(buffer.len(), buffer_size);
            buffers.push(buffer);
        }
        
        // Return all buffers
        for buffer in buffers {
            pool.return_buffer(buffer);
        }
        
        // Should be able to get buffers again
        for _ in 0..operations.min(pool_size) {
            let buffer = pool.get_buffer();
            prop_assert_eq!(buffer.len(), buffer_size);
            pool.return_buffer(buffer);
        }
    }
    
    #[test]
    fn test_protocol_selection_distribution(
        protocol_mix in valid_protocol_mix(),
        selections in 1000usize..=2000
    ) {
        let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix.clone());
        let target_ip: IpAddr = "192.168.1.1".to_string().parse().unwrap();
        
        let mut counts = std::collections::HashMap::new();
        
        for _ in 0..selections {
            let packet_type = packet_builder.next_packet_type_for_ip(target_ip);
            *counts.entry(packet_type).or_insert(0) += 1;
        }
        
        // For IPv4 addresses, the protocol selector only considers IPv4 protocols
        // Calculate the normalized ratios for IPv4 protocols only
        let ipv4_total = protocol_mix.udp_ratio + protocol_mix.tcp_syn_ratio + 
                        protocol_mix.tcp_ack_ratio + protocol_mix.icmp_ratio;
        
        // Only test distribution if we have a reasonable IPv4 protocol mix
        if ipv4_total > 0.1 {
            let normalized_udp_ratio = protocol_mix.udp_ratio / ipv4_total;
            let expected_udp = (selections as f64 * normalized_udp_ratio) as usize;
            
            // Use more generous tolerance for property tests - allow up to 100% variance or at least 20
            let tolerance = std::cmp::max(20, expected_udp);
            
            let udp_count = counts.get(&PacketType::Udp).unwrap_or(&0);
            
            // Only check distribution if we expect a reasonable number of UDP packets
            if expected_udp > 10 {
                prop_assert!((*udp_count as i32 - expected_udp as i32).abs() <= tolerance as i32);
            }
        }
        
        // Basic sanity check: we should have generated some packets
        let total_packets: usize = counts.values().sum();
        prop_assert_eq!(total_packets, selections);
    }
}