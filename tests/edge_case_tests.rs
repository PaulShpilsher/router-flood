//! Edge case tests for critical functionality

use router_flood::security::validation::{validate_target_ip, validate_comprehensive_security};
use router_flood::stats::Stats;
use router_flood::packet::{PacketBuilder, PacketType, PacketSizeRange};
use router_flood::config::ProtocolMix;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[cfg(test)]
mod validation_edge_cases {
    use super::*;

    #[test]
    fn test_boundary_thread_counts() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080];
        
        // Test exact boundary: MAX_THREADS = 100
        assert!(validate_comprehensive_security(&ip, &ports, 100, 1000).is_ok());
        assert!(validate_comprehensive_security(&ip, &ports, 101, 1000).is_err());
        
        // Zero threads currently allowed (bug or feature?)
        assert!(validate_comprehensive_security(&ip, &ports, 0, 1000).is_ok());
    }

    #[test]
    fn test_boundary_packet_rates() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080];
        
        // Test exact boundary: MAX_PACKET_RATE = 10000
        assert!(validate_comprehensive_security(&ip, &ports, 50, 10000).is_ok());
        assert!(validate_comprehensive_security(&ip, &ports, 50, 10001).is_err());
        
        // Zero rate is allowed
        assert!(validate_comprehensive_security(&ip, &ports, 50, 0).is_ok());
    }

    #[test]
    fn test_special_ipv4_addresses() {
        // Test 0.0.0.0
        let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        assert!(validate_target_ip(&ip).is_err());
        
        // Test 255.255.255.255 (broadcast)
        let ip = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));
        assert!(validate_target_ip(&ip).is_err());
        
        // Test 127.0.0.1 (loopback)
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert!(validate_target_ip(&ip).is_err());
        
        // Test 224.0.0.1 (multicast)
        let ip = IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1));
        assert!(validate_target_ip(&ip).is_err());
    }

    #[test]
    fn test_ipv6_edge_cases() {
        // Test IPv6 loopback
        let ip = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert!(validate_target_ip(&ip).is_err());
        
        // Test IPv6 unspecified
        let ip = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
        assert!(validate_target_ip(&ip).is_err());
        
        // Test edge of link-local range
        let ip = IpAddr::V6(Ipv6Addr::new(0xfebf, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff));
        assert!(validate_target_ip(&ip).is_ok());
        
        // Just outside link-local range
        let ip = IpAddr::V6(Ipv6Addr::new(0xfec0, 0, 0, 0, 0, 0, 0, 0));
        assert!(validate_target_ip(&ip).is_err());
    }

    #[test]
    fn test_port_edge_cases() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        
        // Port 0 is technically valid
        assert!(validate_comprehensive_security(&ip, &vec![0], 50, 1000).is_ok());
        
        // Port 65535 is valid
        assert!(validate_comprehensive_security(&ip, &vec![65535], 50, 1000).is_ok());
        
        // Empty ports array is allowed
        assert!(validate_comprehensive_security(&ip, &vec![], 50, 1000).is_ok());
        
        // Large number of ports
        let many_ports: Vec<u16> = (1..=1000).collect();
        assert!(validate_comprehensive_security(&ip, &many_ports, 50, 1000).is_ok());
    }
}

#[cfg(test)]
mod stats_edge_cases {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_stats_overflow_protection() {
        let stats = Stats::new(None);
        
        // Try to overflow packet counter
        for _ in 0..1000 {
            stats.increment_sent(u64::MAX / 1000, "UDP");
        }
        
        // Should not panic, counters should handle overflow gracefully
        assert!(stats.packets_sent() > 0);
    }

    #[test]
    fn test_stats_concurrent_reset() {
        let stats = Arc::new(Stats::new(None));
        let mut handles = vec![];
        
        // Multiple threads incrementing and resetting
        for i in 0..10 {
            let stats_clone = Arc::clone(&stats);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    stats_clone.increment_sent(64, "UDP");
                    if (i + j) % 50 == 0 {
                        stats_clone.reset();
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Final state should be consistent (no crashes)
        let _ = stats.packets_sent();
        let _ = stats.bytes_sent();
    }

    #[test]
    fn test_stats_with_zero_bytes() {
        let stats = Stats::new(None);
        
        // Increment with zero bytes
        stats.increment_sent(0, "UDP");
        assert_eq!(stats.packets_sent(), 1);
        assert_eq!(stats.bytes_sent(), 0);
        
        // Increment with max bytes
        stats.reset();
        stats.increment_sent(u64::MAX, "TCP");
        assert_eq!(stats.packets_sent(), 1);
        assert_eq!(stats.bytes_sent(), u64::MAX);
    }
}

#[cfg(test)]
mod packet_edge_cases {
    use super::*;

    #[test]
    fn test_packet_builder_with_extreme_sizes() {
        let protocol_mix = ProtocolMix::default();
        
        // Minimum size packet
        let mut builder = PacketBuilder::new(PacketSizeRange::new(1, 1), protocol_mix.clone());
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let result = builder.build_packet(PacketType::Udp, ip, 8080);
        assert!(result.is_ok());
        
        // Maximum extreme size - should be clamped internally
        let mut builder = PacketBuilder::new(PacketSizeRange::new(65000, 65535), protocol_mix);
        let result = builder.build_packet(PacketType::Udp, ip, 8080);
        assert!(result.is_ok());
    }

    #[test]
    fn test_packet_builder_with_equal_range() {
        let protocol_mix = ProtocolMix::default();
        
        // Equal min and max - should use that exact size
        let mut builder = PacketBuilder::new(PacketSizeRange::new(100, 100), protocol_mix);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        let result = builder.build_packet(PacketType::Udp, ip, 8080);
        assert!(result.is_ok());
        if let Ok((packet, _)) = result {
            assert!(!packet.is_empty());
        }
    }

    #[test]
    fn test_packet_builder_buffer_boundaries() {
        let protocol_mix = ProtocolMix::default();
        let mut builder = PacketBuilder::new(PacketSizeRange::new(100, 200), protocol_mix);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        // Test with large buffer
        let mut buffer = vec![0u8; 1500];
        let result = builder.build_packet_into_buffer(&mut buffer, PacketType::Udp, ip, 8080);
        assert!(result.is_ok());
        
        // Test with exact minimum size buffer
        let mut exact_buffer = vec![0u8; 100];
        let result = builder.build_packet_into_buffer(&mut exact_buffer, PacketType::Udp, ip, 8080);
        // Should work or fail gracefully
        let _ = result; // Don't assert, just ensure no panic
    }

    #[test]
    fn test_protocol_mix_normalization() {
        // Test with ratios that don't sum to 1.0
        let mut mix = ProtocolMix::default();
        mix.udp_ratio = 10.0;
        mix.tcp_syn_ratio = 10.0;
        mix.tcp_ack_ratio = 10.0;
        mix.icmp_ratio = 10.0;
        mix.custom_ratio = 10.0;
        
        // Should handle this without panicking
        let _builder = PacketBuilder::new(PacketSizeRange::new(100, 200), mix);
        assert!(true); // If we get here, it didn't panic
    }
}

#[cfg(test)]
mod integration_edge_cases {
    use super::*;

    #[test]
    fn test_validation_with_max_values() {
        // Test with all maximum allowed values
        let ip: IpAddr = "10.255.255.255".parse().unwrap();
        let ports: Vec<u16> = vec![65535; 100]; // Many max ports
        let threads = 100; // MAX_THREADS
        let rate = 10000; // MAX_PACKET_RATE
        
        let result = validate_comprehensive_security(&ip, &ports, threads, rate);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rapid_stats_operations() {
        let stats = Stats::new(None);
        
        // Rapid increment and read
        for _ in 0..10000 {
            stats.increment_sent(1, "UDP");
            let _ = stats.packets_sent();
            stats.increment_failed();
            let _ = stats.packets_failed();
        }
        
        // Should complete without issues
        assert!(stats.packets_sent() > 0);
    }
}