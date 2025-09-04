//! Property-based tests using proptest

use proptest::prelude::*;
use router_flood::security::validation::{validate_target_ip, validate_comprehensive_security};
use router_flood::stats::Stats;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Strategy for generating private IPv4 addresses
fn private_ipv4_strategy() -> impl Strategy<Value = Ipv4Addr> {
    prop_oneof![
        // 192.168.0.0/16
        (0u8..=255, 0u8..=255).prop_map(|(b, c)| Ipv4Addr::new(192, 168, b, c)),
        // 10.0.0.0/8
        (0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(a, b, c)| Ipv4Addr::new(10, a, b, c)),
        // 172.16.0.0/12
        (16u8..=31, 0u8..=255, 0u8..=255).prop_map(|(a, b, c)| Ipv4Addr::new(172, a, b, c)),
    ]
}

// Strategy for generating public IPv4 addresses
fn public_ipv4_strategy() -> impl Strategy<Value = Ipv4Addr> {
    (1u8..=254, 1u8..=254, 1u8..=254, 1u8..=254)
        .prop_filter("not private range", |(a, b, c, d)| {
            let _ip = Ipv4Addr::new(*a, *b, *c, *d);
            // Exclude private ranges
            !(*a == 192 && *b == 168) &&
            !(*a == 10) &&
            !(*a == 172 && *b >= 16 && *b <= 31) &&
            !(*a == 127) && // loopback
            !(*a == 0) &&   // unspecified
            !(*a >= 224)    // multicast
        })
        .prop_map(|(a, b, c, d)| Ipv4Addr::new(a, b, c, d))
}

proptest! {
    #[test]
    fn test_private_ipv4_always_valid(ip in private_ipv4_strategy()) {
        let addr = IpAddr::V4(ip);
        prop_assert!(validate_target_ip(&addr).is_ok());
    }

    #[test]
    fn test_public_ipv4_always_invalid(ip in public_ipv4_strategy()) {
        let addr = IpAddr::V4(ip);
        prop_assert!(validate_target_ip(&addr).is_err());
    }

    #[test]
    fn test_comprehensive_validation_thread_bounds(
        threads in 0usize..=200,
        rate in 0u64..=20000,
    ) {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080];
        let result = validate_comprehensive_security(&ip, &ports, threads, rate);
        
        // Threads > 100 should fail, rate > 10000 should fail
        if threads > 100 || rate > 10000 {
            prop_assert!(result.is_err());
        } else {
            prop_assert!(result.is_ok());
        }
    }

    #[test]
    fn test_stats_increment_consistency(
        packet_count in 1usize..1000,
        bytes_per_packet in 1u64..1500,
    ) {
        let stats = Stats::new(None);
        
        for _ in 0..packet_count {
            stats.increment_sent(bytes_per_packet, "UDP");
        }
        
        prop_assert_eq!(stats.packets_sent() as usize, packet_count);
        prop_assert_eq!(stats.bytes_sent(), bytes_per_packet * packet_count as u64);
    }

    #[test]
    fn test_stats_reset_always_zeros(
        increments in prop::collection::vec((1u64..1000, 1u64..1500), 1..100)
    ) {
        let stats = Stats::new(None);
        
        // Add some data
        for (count, bytes) in increments {
            for _ in 0..count {
                stats.increment_sent(bytes, "TCP");
            }
        }
        
        // Reset should always result in zeros
        stats.reset();
        prop_assert_eq!(stats.packets_sent(), 0);
        prop_assert_eq!(stats.bytes_sent(), 0);
        prop_assert_eq!(stats.packets_failed(), 0);
    }

    #[test]
    fn test_port_validation_range(port in 0u16..=65535) {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![port];
        
        // All port numbers should be accepted
        let result = validate_comprehensive_security(&ip, &ports, 50, 5000);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_ipv6_private_ranges(
        first_segment in prop_oneof![
            0xfe80u16..=0xfebf,  // Link-local
            0xfc00u16..=0xfdff,  // Unique local
        ],
        s2 in 0u16..=0xffff,
        s3 in 0u16..=0xffff,
        s4 in 0u16..=0xffff,
    ) {
        let ip = IpAddr::V6(Ipv6Addr::new(first_segment, s2, s3, s4, 0, 0, 0, 1));
        prop_assert!(validate_target_ip(&ip).is_ok());
    }

    #[test]
    fn test_stats_increment_failed_count(failures in 0u64..1000) {
        let stats = Stats::new(None);
        
        for _ in 0..failures {
            stats.increment_failed();
        }
        
        prop_assert_eq!(stats.packets_failed(), failures);
    }
}