//! Property-based testing for router-flood
//!
//! Property-based tests using proptest

#![allow(clippy::nonminimal_bool)]
//! the system behaves correctly under all conditions.

use proptest::prelude::*;
use router_flood::config::*;
use router_flood::packet::*;
use router_flood::validation::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Generate valid private IPv4 addresses
fn private_ipv4_strategy() -> impl Strategy<Value = IpAddr> {
    prop_oneof![
        // 192.168.0.0/16
        (192u8..=192, 168u8..=168, 0u8..=255, 1u8..=254)
            .prop_map(|(a, b, c, d)| IpAddr::V4(Ipv4Addr::new(a, b, c, d))),
        // 10.0.0.0/8
        (10u8..=10, 0u8..=255, 0u8..=255, 1u8..=254)
            .prop_map(|(a, b, c, d)| IpAddr::V4(Ipv4Addr::new(a, b, c, d))),
        // 172.16.0.0/12
        (172u8..=172, 16u8..=31, 0u8..=255, 1u8..=254)
            .prop_map(|(a, b, c, d)| IpAddr::V4(Ipv4Addr::new(a, b, c, d))),
    ]
}

/// Generate valid port numbers
fn valid_port_strategy() -> impl Strategy<Value = u16> {
    1u16..=65535
}

/// Generate valid protocol mix ratios that sum to 1.0
fn protocol_mix_strategy() -> impl Strategy<Value = ProtocolMix> {
    (0.0f64..=1.0, 0.0f64..=1.0, 0.0f64..=1.0, 0.0f64..=1.0, 0.0f64..=1.0)
        .prop_map(|(udp, tcp_syn, tcp_ack, icmp, ipv6)| {
            let total = udp + tcp_syn + tcp_ack + icmp + ipv6;
            if total == 0.0 {
                // Fallback to default if all zeros
                ProtocolMix {
                    udp_ratio: 1.0,
                    tcp_syn_ratio: 0.0,
                    tcp_ack_ratio: 0.0,
                    icmp_ratio: 0.0,
                    ipv6_ratio: 0.0,
                    arp_ratio: 0.0,
                }
            } else {
                // Normalize to sum to 1.0
                ProtocolMix {
                    udp_ratio: udp / total,
                    tcp_syn_ratio: tcp_syn / total,
                    tcp_ack_ratio: tcp_ack / total,
                    icmp_ratio: icmp / total,
                    ipv6_ratio: ipv6 / total,
                    arp_ratio: 0.0, // Keep ARP at 0 for simplicity
                }
            }
        })
}

/// Generate valid packet size ranges
fn packet_size_range_strategy() -> impl Strategy<Value = (usize, usize)> {
    (20usize..=1400, 20usize..=1500)
        .prop_filter("min must be less than max", |(min, max)| min < max)
}

proptest! {
    #[test]
    fn test_private_ip_validation_always_succeeds(
        ip in private_ipv4_strategy()
    ) {
        // Private IPs should always pass validation
        prop_assert!(validate_target_ip(&ip).is_ok());
    }

    #[test]
    fn test_public_ip_validation_always_fails(
        a in 1u8..=223,
        b in 0u8..=255,
        c in 0u8..=255,
        d in 1u8..=254
    ) {
        // Skip private ranges
        prop_assume!(!(a == 192 && b == 168));
        prop_assume!(!(a == 10));
        prop_assume!(!(a == 172 && (16..=31).contains(&b)));
        prop_assume!(!(a == 127)); // Skip loopback
        prop_assume!(!(a >= 224)); // Skip multicast/reserved
        
        let ip = IpAddr::V4(Ipv4Addr::new(a, b, c, d));
        prop_assert!(validate_target_ip(&ip).is_err());
    }

    #[test]
    fn test_packet_builder_never_panics(
        packet_size_range in packet_size_range_strategy(),
        protocol_mix in protocol_mix_strategy(),
        target_ip in private_ipv4_strategy(),
        target_port in valid_port_strategy()
    ) {
        let mut builder = PacketBuilder::new(packet_size_range, protocol_mix);
        
        // Should never panic when generating packet types
        let packet_type = builder.next_packet_type_for_ip(target_ip);
        
        // Should never panic when building packets
        let result = builder.build_packet(packet_type, target_ip, target_port);
        
        // Either succeeds or fails gracefully
        prop_assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_zero_copy_packet_building_never_panics(
        packet_size_range in packet_size_range_strategy(),
        protocol_mix in protocol_mix_strategy(),
        target_ip in private_ipv4_strategy(),
        target_port in valid_port_strategy(),
        buffer_size in 100usize..=2000
    ) {
        let mut builder = PacketBuilder::new(packet_size_range, protocol_mix);
        let packet_type = builder.next_packet_type_for_ip(target_ip);
        
        let mut buffer = vec![0u8; buffer_size];
        let result = builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            target_ip,
            target_port,
        );
        
        // Should either succeed or fail gracefully (never panic)
        prop_assert!(result.is_ok() || result.is_err());
        
        if let Ok((packet_size, _)) = result {
            // Packet size should never exceed buffer size
            prop_assert!(packet_size <= buffer_size);
            // Packet size should be reasonable
            prop_assert!(packet_size >= 20);
        }
    }

    #[test]
    fn test_config_validation_comprehensive(
        threads in 1usize..=100,
        packet_rate in 1u64..=10000,
        target_ip in private_ipv4_strategy(),
        ports in prop::collection::vec(valid_port_strategy(), 1..=10),
        protocol_mix in protocol_mix_strategy(),
        packet_size_range in packet_size_range_strategy()
    ) {
        let mut config = get_default_config();
        config.attack.threads = threads;
        config.attack.packet_rate = packet_rate;
        config.target.ip = target_ip.to_string();
        config.target.ports = ports;
        config.target.protocol_mix = protocol_mix;
        config.attack.packet_size_range = packet_size_range;
        
        // Comprehensive validation should succeed for valid inputs
        let result = validate_comprehensive_security(
            &target_ip,
            &config.target.ports,
            config.attack.threads,
            config.attack.packet_rate,
        );
        
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_protocol_mix_ratios_always_valid(
        protocol_mix in protocol_mix_strategy()
    ) {
        // All ratios should be non-negative
        prop_assert!(protocol_mix.udp_ratio >= 0.0);
        prop_assert!(protocol_mix.tcp_syn_ratio >= 0.0);
        prop_assert!(protocol_mix.tcp_ack_ratio >= 0.0);
        prop_assert!(protocol_mix.icmp_ratio >= 0.0);
        prop_assert!(protocol_mix.ipv6_ratio >= 0.0);
        prop_assert!(protocol_mix.arp_ratio >= 0.0);
        
        // Sum should be approximately 1.0 (within floating point precision)
        let total = protocol_mix.udp_ratio + protocol_mix.tcp_syn_ratio + 
                   protocol_mix.tcp_ack_ratio + protocol_mix.icmp_ratio + 
                   protocol_mix.ipv6_ratio + protocol_mix.arp_ratio;
        prop_assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_config_schema_validation_robustness(
        threads in 1usize..=100,
        packet_rate in 1u64..=10000,
        stats_interval in 1u64..=3600,
        export_interval in prop::option::of(1u64..=7200),
        packet_size_range in packet_size_range_strategy()
    ) {
        let mut config = get_default_config();
        config.attack.threads = threads;
        config.attack.packet_rate = packet_rate;
        config.monitoring.stats_interval = stats_interval;
        config.monitoring.export_interval = export_interval;
        config.attack.packet_size_range = packet_size_range;
        
        // Schema validation should succeed for valid configurations
        let result = ConfigSchema::validate(&config);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_port_parsing_robustness(
        ports in prop::collection::vec(valid_port_strategy(), 1..=20)
    ) {
        let ports_str = ports.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        
        let result = router_flood::cli::parse_ports(&ports_str);
        prop_assert!(result.is_ok());
        
        if let Ok(parsed_ports) = result {
            prop_assert_eq!(parsed_ports.len(), ports.len());
            for (original, parsed) in ports.iter().zip(parsed_ports.iter()) {
                prop_assert_eq!(original, parsed);
            }
        }
    }
}

/// Regression tests for specific edge cases found through property testing
#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_edge_case_minimum_packet_size() {
        let protocol_mix = ProtocolMix {
            udp_ratio: 1.0,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 0.0,
            arp_ratio: 0.0,
        };
        
        let mut builder = PacketBuilder::new((20, 21), protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        // Should handle minimum packet size gracefully
        let result = builder.build_packet(PacketType::Udp, target_ip, 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case_maximum_packet_size() {
        let protocol_mix = ProtocolMix {
            udp_ratio: 1.0,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 0.0,
            arp_ratio: 0.0,
        };
        
        let mut builder = PacketBuilder::new((1400, 1500), protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        // Should handle maximum packet size gracefully
        let result = builder.build_packet(PacketType::Udp, target_ip, 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case_single_protocol() {
        // Test with only one protocol enabled
        let protocol_mix = ProtocolMix {
            udp_ratio: 1.0,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 0.0,
            arp_ratio: 0.0,
        };
        
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        // Should always return UDP packets
        for _ in 0..100 {
            let packet_type = builder.next_packet_type_for_ip(target_ip);
            assert_eq!(packet_type, PacketType::Udp);
        }
    }

    #[test]
    fn test_edge_case_ipv6_target() {
        let protocol_mix = ProtocolMix {
            udp_ratio: 0.5,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 0.5,
            arp_ratio: 0.0,
        };
        
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
        let target_ip = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
        
        // Should handle IPv6 targets appropriately
        let packet_type = builder.next_packet_type_for_ip(target_ip);
        // Should generate IPv6 packets for IPv6 targets
        assert!(matches!(packet_type, PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp));
    }
}