//! Alternative to fuzzing using property-based testing
//!
//! This provides similar coverage to cargo-fuzz but works with stable Rust.

use proptest::prelude::*;
use router_flood::packet::*;
use router_flood::config::ProtocolMix;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Generate valid private IPv4 addresses
fn private_ipv4() -> impl Strategy<Value = Ipv4Addr> {
    prop_oneof![
        // 192.168.0.0/16
        (0u8..=255, 1u8..=254).prop_map(|(b, c)| Ipv4Addr::new(192, 168, b, c)),
        // 10.0.0.0/8  
        (0u8..=255, 0u8..=255, 1u8..=254).prop_map(|(a, b, c)| Ipv4Addr::new(10, a, b, c)),
        // 172.16.0.0/12
        (16u8..=31, 0u8..=255, 1u8..=254).prop_map(|(a, b, c)| Ipv4Addr::new(172, a, b, c)),
    ]
}

// Generate link-local IPv6 addresses
fn link_local_ipv6() -> impl Strategy<Value = Ipv6Addr> {
    any::<[u16; 7]>().prop_map(|segments| {
        let mut addr = [0u16; 8];
        addr[0] = 0xfe80; // Link-local prefix
        addr[1..].copy_from_slice(&segments);
        Ipv6Addr::from(addr)
    })
}

// Generate valid protocol mix
fn valid_protocol_mix() -> impl Strategy<Value = ProtocolMix> {
    (0.0f64..=1.0, 0.0f64..=1.0, 0.0f64..=1.0, 0.0f64..=1.0, 0.0f64..=1.0, 0.0f64..=1.0)
        .prop_map(|(udp, tcp_syn, tcp_ack, icmp, ipv6, arp)| {
            let total = udp + tcp_syn + tcp_ack + icmp + ipv6 + arp;
            if total == 0.0 {
                ProtocolMix {
                    udp_ratio: 1.0,
                    tcp_syn_ratio: 0.0,
                    tcp_ack_ratio: 0.0,
                    icmp_ratio: 0.0,
                    ipv6_ratio: 0.0,
                    arp_ratio: 0.0,
                }
            } else {
                ProtocolMix {
                    udp_ratio: udp / total,
                    tcp_syn_ratio: tcp_syn / total,
                    tcp_ack_ratio: tcp_ack / total,
                    icmp_ratio: icmp / total,
                    ipv6_ratio: ipv6 / total,
                    arp_ratio: arp / total,
                }
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        max_shrink_iters: 1000,
        timeout: 5000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn test_packet_builder_never_panics_ipv4(
        min_size in 20usize..=500,
        max_size in 500usize..=1400,
        protocol_mix in valid_protocol_mix(),
        target_ip in private_ipv4(),
        target_port in 1u16..=65535,
        buffer_size in 50usize..=2000,
    ) {
        let packet_size_range = (min_size, max_size.max(min_size + 1));
        let mut builder = PacketBuilder::new(packet_size_range, protocol_mix);
        let target_ip = IpAddr::V4(target_ip);
        
        // Test packet type generation
        let packet_type = builder.next_packet_type_for_ip(target_ip);
        
        // Test regular packet building
        let result = builder.build_packet(packet_type, target_ip, target_port);
        // Should either succeed or fail gracefully, never panic
        match result {
            Ok((data, protocol)) => {
                prop_assert!(!data.is_empty());
                prop_assert!(!protocol.is_empty());
            }
            Err(_) => {
                // Errors are acceptable for edge cases
            }
        }
        
        // Test zero-copy building
        let mut buffer = vec![0u8; buffer_size];
        let result = builder.build_packet_into_buffer(&mut buffer, packet_type, target_ip, target_port);
        match result {
            Ok((size, protocol)) => {
                prop_assert!(size > 0);
                prop_assert!(size <= buffer_size);
                prop_assert!(!protocol.is_empty());
            }
            Err(_) => {
                // Errors are acceptable for small buffers
            }
        }
    }

    #[test]
    fn test_packet_builder_never_panics_ipv6(
        min_size in 20usize..=500,
        max_size in 500usize..=1400,
        protocol_mix in valid_protocol_mix(),
        target_ip in link_local_ipv6(),
        target_port in 1u16..=65535,
        buffer_size in 50usize..=2000,
    ) {
        let packet_size_range = (min_size, max_size.max(min_size + 1));
        let mut builder = PacketBuilder::new(packet_size_range, protocol_mix);
        let target_ip = IpAddr::V6(target_ip);
        
        // Test packet type generation
        let packet_type = builder.next_packet_type_for_ip(target_ip);
        
        // Test regular packet building
        let result = builder.build_packet(packet_type, target_ip, target_port);
        match result {
            Ok((data, protocol)) => {
                prop_assert!(!data.is_empty());
                prop_assert!(!protocol.is_empty());
            }
            Err(_) => {
                // Errors are acceptable for edge cases
            }
        }
        
        // Test zero-copy building
        let mut buffer = vec![0u8; buffer_size];
        let result = builder.build_packet_into_buffer(&mut buffer, packet_type, target_ip, target_port);
        match result {
            Ok((size, protocol)) => {
                prop_assert!(size > 0);
                prop_assert!(size <= buffer_size);
                prop_assert!(!protocol.is_empty());
            }
            Err(_) => {
                // Errors are acceptable for small buffers
            }
        }
    }

    #[test]
    fn test_all_packet_types_never_panic(
        min_size in 20usize..=500,
        max_size in 500usize..=1400,
        protocol_mix in valid_protocol_mix(),
        target_ip in private_ipv4(),
        target_port in 1u16..=65535,
    ) {
        let packet_size_range = (min_size, max_size.max(min_size + 1));
        let mut builder = PacketBuilder::new(packet_size_range, protocol_mix);
        let target_ip = IpAddr::V4(target_ip);
        
        // Test all packet types
        let packet_types = [
            PacketType::Udp,
            PacketType::TcpSyn,
            PacketType::TcpAck,
            PacketType::Icmp,
            PacketType::Arp,
        ];
        
        for &packet_type in &packet_types {
            let result = builder.build_packet(packet_type, target_ip, target_port);
            // Should never panic, may succeed or fail gracefully
            match result {
                Ok((data, protocol)) => {
                    prop_assert!(!data.is_empty());
                    prop_assert!(!protocol.is_empty());
                }
                Err(_) => {
                    // Acceptable for some combinations
                }
            }
        }
    }

    #[test]
    fn test_edge_case_buffer_sizes(
        protocol_mix in valid_protocol_mix(),
        target_ip in private_ipv4(),
        target_port in 1u16..=65535,
        buffer_size in 1usize..=100,
    ) {
        let packet_size_range = (64, 1400);
        let mut builder = PacketBuilder::new(packet_size_range, protocol_mix);
        let target_ip = IpAddr::V4(target_ip);
        let packet_type = PacketType::Udp;
        
        // Test with very small buffers
        let mut buffer = vec![0u8; buffer_size];
        let result = builder.build_packet_into_buffer(&mut buffer, packet_type, target_ip, target_port);
        
        // Should never panic, even with tiny buffers
        match result {
            Ok((size, _)) => {
                prop_assert!(size <= buffer_size);
            }
            Err(_) => {
                // Expected for very small buffers
            }
        }
    }
}

#[cfg(test)]
mod deterministic_tests {
    use super::*;

    #[test]
    fn test_packet_builder_basic_functionality() {
        let protocol_mix = ProtocolMix {
            udp_ratio: 1.0,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 0.0,
            arp_ratio: 0.0,
        };
        
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        
        // Should be able to build UDP packets
        let result = builder.build_packet(PacketType::Udp, target_ip, 80);
        assert!(result.is_ok());
        
        let (data, protocol) = result.unwrap();
        assert!(!data.is_empty());
        assert_eq!(protocol, "UDP");
    }

    #[test]
    fn test_packet_builder_with_tiny_buffer() {
        let protocol_mix = ProtocolMix {
            udp_ratio: 1.0,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 0.0,
            arp_ratio: 0.0,
        };
        
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        
        // Test with buffer too small for any packet
        let mut tiny_buffer = vec![0u8; 10];
        let result = builder.build_packet_into_buffer(&mut tiny_buffer, PacketType::Udp, target_ip, 80);
        
        // Should fail gracefully, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_packet_builder_all_types() {
        let protocol_mix = ProtocolMix {
            udp_ratio: 0.2,
            tcp_syn_ratio: 0.2,
            tcp_ack_ratio: 0.2,
            icmp_ratio: 0.2,
            ipv6_ratio: 0.1,
            arp_ratio: 0.1,
        };
        
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        
        // Test all IPv4 packet types
        for packet_type in [PacketType::Udp, PacketType::TcpSyn, PacketType::TcpAck, PacketType::Icmp, PacketType::Arp] {
            let result = builder.build_packet(packet_type, target_ip, 80);
            // Should not panic, may succeed or fail depending on implementation
            match result {
                Ok((data, _)) => assert!(!data.is_empty()),
                Err(_) => {} // Acceptable
            }
        }
    }
}