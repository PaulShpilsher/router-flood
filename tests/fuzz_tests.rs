//! Fuzzing tests for packet generation and parsing

use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::{Config, ProtocolMix, LoadConfig, Safety};
use router_flood::security::validation::validate_target_ip;
use proptest::prelude::*;
use proptest::prop_oneof;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Fuzzing packet builder with random inputs
proptest! {
    #[test]
    fn fuzz_packet_builder_doesnt_panic(
        min_size in 20..=9000usize,
        max_size in 20..=9000usize,
        udp_ratio in 0.0..=1.0f64,
        tcp_syn_ratio in 0.0..=1.0f64,
        tcp_ack_ratio in 0.0..=1.0f64,
        icmp_ratio in 0.0..=1.0f64,
        custom_ratio in 0.0..=1.0f64,
        ip_octets in prop::array::uniform4(0u8..=255),
        port in 1u16..=65535,
        packet_type_num in 0u8..=4
    ) {
        let protocol_mix = ProtocolMix {
            udp_ratio,
            tcp_syn_ratio,
            tcp_ack_ratio,
            icmp_ratio,
            custom_ratio,
        };
        
        let size_range = if min_size <= max_size {
            (min_size, max_size)
        } else {
            (max_size, min_size)
        };
        
        let mut builder = PacketBuilder::new(size_range, protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3]));
        
        let packet_type = match packet_type_num % 8 {
            0 => PacketType::Udp,
            1 => PacketType::TcpSyn,
            2 => PacketType::TcpAck,
            3 => PacketType::Icmp,
            4 => PacketType::Ipv6Udp,
            5 => PacketType::Ipv6Tcp,
            6 => PacketType::Ipv6Icmp,
            _ => PacketType::Arp,
        };
        
        // Should not panic regardless of input
        match builder.build_packet(packet_type, target_ip, port) {
            Ok((packet, _)) => {
                // Basic sanity checks
                prop_assert!(packet.len() >= 20, "Packet too small");
                prop_assert!(packet.len() <= 9000, "Packet too large");
            },
            Err(_) => {
                // Error is acceptable for invalid inputs
                prop_assert!(true);
            }
        }
    }
}

// Fuzzing configuration validation
proptest! {
    #[test]
    fn fuzz_config_validation_doesnt_panic(
        threads in 0..=1000usize,
        packet_rate in prop::num::f64::ANY,
        payload_size in 0..=100000usize,
        duration in prop::option::of(0..=100000u64),
        burst_mode in prop::bool::ANY,
        dry_run in prop::bool::ANY,
        rate_limit in prop::bool::ANY,
        max_bandwidth in prop::option::of(prop::num::f64::ANY),
        allow_localhost in prop::bool::ANY,
        require_confirmation in prop::bool::ANY
    ) {
        let config = Config {
            target: router_flood::config::Target {
                ip: "192.168.1.1".to_string(),
                ports: vec![80, 443],
                protocol_mix: ProtocolMix::default(),
                interface: None,
            },
            attack: LoadConfig {
                threads,
                packet_rate,
                payload_size,
                duration,
                burst_mode,
                burst_pattern: None,
            },
            safety: Safety {
                dry_run,
                rate_limit,
                max_bandwidth_mbps: max_bandwidth,
                allow_localhost,
                require_confirmation,
            },
            monitoring: router_flood::config::Monitoring {
                enabled: true,
                interval_ms: 1000,
                verbose: false,
                show_stats: true,
            },
            export: router_flood::config::Export {
                enabled: false,
                format: router_flood::config::ExportFormat::Json,
                path: "./stats".to_string(),
                interval_seconds: 60,
                include_system_stats: false,
            },
        };
        
        // Validation should handle any input gracefully
        let result = router_flood::config::validate_config(&config);
        
        // It should either succeed or return an error, never panic
        prop_assert!(result.is_ok() || result.is_err());
    }
}

// Fuzzing IP validation with random strings
proptest! {
    #[test]
    fn fuzz_ip_validation_with_random_strings(
        s in ".*"
    ) {
        // Should handle any string input gracefully
        let result = s.parse::<IpAddr>();
        if let Ok(ip) = result {
            let validation = validate_target_ip(&ip);
            prop_assert!(validation.is_ok() || validation.is_err());
        }
    }
}

// Fuzzing with malformed IP addresses
proptest! {
    #[test]
    fn fuzz_malformed_ip_addresses(
        octets in prop::collection::vec(0u32..=999, 1..=10)
    ) {
        let ip_string = octets.iter()
            .map(|o| o.to_string())
            .collect::<Vec<_>>()
            .join(".");
        
        // Should handle malformed IPs gracefully
        let result = ip_string.parse::<IpAddr>();
        prop_assert!(result.is_ok() || result.is_err());
    }
}

// Fuzzing packet generation with extreme values
proptest! {
    #[test]
    fn fuzz_extreme_packet_values(
        size in prop_oneof![
            Just(0usize),
            Just(1usize),
            Just(20usize),
            Just(65535usize),
            Just(usize::MAX),
        ],
        port in prop_oneof![
            Just(0u16),
            Just(1u16),
            Just(65535u16),
        ]
    ) {
        let protocol_mix = ProtocolMix::default();
        let size_range = (size.min(9000), size.min(9000));
        let mut builder = PacketBuilder::new(size_range, protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        // Should handle extreme values gracefully
        if let Ok((packet, _)) = builder.build_packet(PacketType::Udp, target_ip, port) {
            prop_assert!(packet.len() >= 20);
        } else {
            // Error is acceptable for extreme values
            prop_assert!(true);
        }
    }
}

// Fuzzing protocol mix normalization
proptest! {
    #[test]
    fn fuzz_protocol_mix_normalization(
        ratios in prop::collection::vec(prop::num::f64::ANY, 5..=5)
    ) {
        let protocol_mix = ProtocolMix {
            udp_ratio: ratios[0],
            tcp_syn_ratio: ratios[1],
            tcp_ack_ratio: ratios[2],
            icmp_ratio: ratios[3],
            custom_ratio: ratios[4],
        };
        
        // Create builder - normalization should handle any input
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
        let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        // Should produce valid packets regardless of ratios
        for _ in 0..10 {
            if let Ok((packet, _)) = builder.build_packet(PacketType::Udp, target_ip, 8080) {
                prop_assert!(packet.len() >= 20);
            }
        }
    }
}

// Fuzzing with IPv6 addresses
proptest! {
    #[test]
    fn fuzz_ipv6_packet_generation(
        segments in prop::array::uniform8(0u16..=0xFFFF),
        port in 1u16..=65535
    ) {
        let protocol_mix = ProtocolMix::default();
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
        
        let target_ip = IpAddr::V6(Ipv6Addr::new(
            segments[0], segments[1], segments[2], segments[3],
            segments[4], segments[5], segments[6], segments[7]
        ));
        
        // Should handle IPv6 addresses
        if let Ok((packet, _)) = builder.build_packet(PacketType::Udp, target_ip, port) {
            prop_assert!(packet.len() >= 20);
        } else {
            // IPv6 errors are acceptable
            prop_assert!(true);
        }
    }
}

// Fuzzing buffer operations
proptest! {
    #[test]
    fn fuzz_buffer_operations(
        buffer_size in 1..=10000usize,
        write_positions in prop::collection::vec(0..=10000usize, 0..=100),
        write_values in prop::collection::vec(0u8..=255, 0..=100)
    ) {
        let mut buffer = vec![0u8; buffer_size];
        
        // Attempt writes at various positions
        for (pos, val) in write_positions.iter().zip(write_values.iter()) {
            if *pos < buffer.len() {
                buffer[*pos] = *val;
            }
        }
        
        // Buffer should remain valid
        prop_assert_eq!(buffer.len(), buffer_size);
    }
}

// Fuzzing concurrent packet generation
proptest! {
    #[test]
    fn fuzz_concurrent_packet_generation(
        thread_count in 1..=20usize,
        packets_per_thread in 1..=100usize
    ) {
        
        use std::thread;
        
        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let packets = packets_per_thread;
                thread::spawn(move || {
                    let protocol_mix = ProtocolMix::default();
                    let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
                    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
                    
                    for _ in 0..packets {
                        let _ = builder.build_packet(PacketType::Udp, target_ip, 8080);
                    }
                })
            })
            .collect();
        
        // All threads should complete without panic
        for handle in handles {
            handle.join().unwrap();
        }
        
        prop_assert!(true);
    }
}