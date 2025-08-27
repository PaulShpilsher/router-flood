//! Integration tests for the new architecture
//!
//! These tests verify that all components work together correctly
//! and that the new architecture maintains compatibility.

use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ConfigBuilder;
use router_flood::transport::MockTransport;
use router_flood::performance::LockFreeBufferPool;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread;

#[test]
fn test_end_to_end_packet_generation() {
    // Test the complete flow from configuration to packet generation
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(vec![80, 443, 8080])
        .threads(2)
        .packet_rate(100)
        .packet_size_range(64, 1400)
        .build()
        .expect("Config should be valid");
    
    let mut packet_builder = PacketBuilder::new(
        config.attack.packet_size_range,
        config.target.protocol_mix.clone(),
    );
    
    let target_ip: IpAddr = config.target.ip.parse().unwrap();
    
    // Generate packets for each supported type
    let packet_types = [
        PacketType::Udp,
        PacketType::TcpSyn,
        PacketType::TcpAck,
        PacketType::Icmp,
    ];
    
    for packet_type in packet_types {
        for &port in &config.target.ports {
            let mut buffer = vec![0u8; 1500];
            let result = packet_builder.build_packet_into_buffer(
                &mut buffer,
                packet_type,
                target_ip,
                port,
            );
            
            assert!(result.is_ok(), "Failed to build {:?} packet for port {}", packet_type, port);
            
            let (size, protocol_name) = result.unwrap();
            assert!(size > 0, "Packet size should be positive");
            assert!(size <= 1500, "Packet size should not exceed MTU");
            assert!(!protocol_name.is_empty(), "Protocol name should not be empty");
        }
    }
}

#[test]
fn test_mock_transport_integration() {
    let transport = MockTransport::new();
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Test sending different types of packets
    let test_data = vec![
        (vec![0u8; 100], router_flood::transport::ChannelType::IPv4),
        (vec![1u8; 200], router_flood::transport::ChannelType::IPv6),
        (vec![2u8; 64], router_flood::transport::ChannelType::Layer2),
    ];
    
    for (data, channel_type) in test_data {
        let result = transport.send_packet(&data, target_ip, channel_type);
        assert!(result.is_ok(), "Mock transport should succeed");
    }
    
    assert_eq!(transport.packets_sent(), 3, "Should have sent 3 packets");
}

#[test]
fn test_buffer_pool_performance() {
    let pool = Arc::new(LockFreeBufferPool::new(1400, 50));
    let mut handles = vec![];
    
    // Test concurrent access from multiple threads
    for thread_id in 0..4 {
        let pool_clone = Arc::clone(&pool);
        
        let handle = thread::spawn(move || {
            let mut local_buffers = Vec::new();
            
            // Get buffers
            for _ in 0..10 {
                if let Some(buffer) = pool_clone.get_buffer() {
                    assert_eq!(buffer.len(), 1400, "Buffer size should be correct");
                    local_buffers.push(buffer);
                }
            }
            
            // Return buffers
            for buffer in local_buffers {
                pool_clone.return_buffer(buffer);
            }
            
            thread_id
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        let thread_id = handle.join().unwrap();
        println!("Thread {} completed successfully", thread_id);
    }
    
    // Pool should still be functional
    let buffer = pool.get_buffer().unwrap();
    assert_eq!(buffer.len(), 1400);
    pool.return_buffer(buffer);
}

#[test]
fn test_configuration_validation_edge_cases() {
    // Test various edge cases for configuration validation
    
    // Valid edge case: minimum values
    let min_config = ConfigBuilder::new()
        .target_ip("192.168.0.1")
        .target_ports(vec![1])
        .threads(1)
        .packet_rate(1)
        .packet_size_range(20, 21)
        .build();
    assert!(min_config.is_ok(), "Minimum valid config should succeed");
    
    // Valid edge case: maximum values (within limits)
    let max_config = ConfigBuilder::new()
        .target_ip("192.168.255.254")
        .target_ports((1..=100).collect())
        .threads(50)
        .packet_rate(5000)
        .packet_size_range(1000, 1400)
        .build();
    assert!(max_config.is_ok(), "Maximum valid config should succeed");
    
    // Invalid: public IP
    let public_ip_config = ConfigBuilder::new()
        .target_ip("8.8.8.8")
        .build();
    assert!(public_ip_config.is_err(), "Public IP should be rejected");
    
    // Invalid: zero threads
    let zero_threads_config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .threads(0)
        .build();
    assert!(zero_threads_config.is_err(), "Zero threads should be rejected");
    
    // Invalid: packet size range
    let invalid_size_config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .packet_size_range(1000, 500) // min > max
        .build();
    assert!(invalid_size_config.is_err(), "Invalid size range should be rejected");
}

#[test]
fn test_protocol_selection_accuracy() {
    // Test that protocol selection respects the configured ratios
    let protocol_mix = router_flood::config_original::ProtocolMix {
        udp_ratio: 1.0, // 100% UDP
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // With 100% UDP ratio, should always select UDP
    for _ in 0..100 {
        let packet_type = packet_builder.next_packet_type_for_ip(target_ip);
        assert_eq!(packet_type, PacketType::Udp, "Should always select UDP with 100% ratio");
    }
}

#[test]
fn test_ipv6_compatibility() {
    let protocol_mix = router_flood::config_original::ProtocolMix {
        udp_ratio: 0.0,
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 1.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let ipv6_target: IpAddr = "fe80::1".parse().unwrap();
    
    // Test IPv6 packet generation
    for _ in 0..50 {
        let packet_type = packet_builder.next_packet_type_for_ip(ipv6_target);
        
        // Should only select IPv6 packet types
        assert!(
            matches!(packet_type, PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp),
            "Should only select IPv6 packet types for IPv6 target"
        );
        
        // Test packet building
        let mut buffer = vec![0u8; 1500];
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            ipv6_target,
            80,
        );
        
        assert!(result.is_ok(), "IPv6 packet building should succeed");
        
        let (size, protocol_name) = result.unwrap();
        assert!(size >= 48, "IPv6 packet should be at least 48 bytes (IPv6 header + protocol)");
        assert_eq!(protocol_name, "IPv6", "Protocol name should be IPv6");
    }
}

#[test]
fn test_error_handling_robustness() {
    let protocol_mix = router_flood::config_original::ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    
    // Test with buffer too small
    let mut small_buffer = vec![0u8; 10];
    let result = packet_builder.build_packet_into_buffer(
        &mut small_buffer,
        PacketType::Udp,
        "192.168.1.1".parse().unwrap(),
        80,
    );
    
    assert!(result.is_err(), "Should fail with buffer too small");
    
    // Test with incompatible packet type and IP version
    let mut buffer = vec![0u8; 1500];
    let result = packet_builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp, // IPv4 packet type
        "fe80::1".parse().unwrap(), // IPv6 address
        80,
    );
    
    assert!(result.is_err(), "Should fail with incompatible packet type and IP version");
}

#[test]
fn test_performance_optimizations() {
    use std::time::Instant;
    
    let protocol_mix = router_flood::config_original::ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Test zero-copy performance
    let start = Instant::now();
    let mut buffer = vec![0u8; 1500];
    
    for _ in 0..1000 {
        let _ = packet_builder.build_packet_into_buffer(
            &mut buffer,
            PacketType::Udp,
            target_ip,
            80,
        );
    }
    
    let zero_copy_duration = start.elapsed();
    
    // Test allocation performance
    let start = Instant::now();
    
    for _ in 0..1000 {
        let _ = packet_builder.build_packet(
            PacketType::Udp,
            target_ip,
            80,
        );
    }
    
    let allocation_duration = start.elapsed();
    
    println!("Zero-copy: {:?}, Allocation: {:?}", zero_copy_duration, allocation_duration);
    
    // Zero-copy should be faster (though this might not always be true in debug builds)
    // This is more of a performance monitoring test than a strict assertion
    assert!(zero_copy_duration < allocation_duration * 2, 
           "Zero-copy should be reasonably competitive with allocation");
}