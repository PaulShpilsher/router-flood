//! Packet building workflow integration tests

use router_flood::packet::{PacketBuilder, PacketType, PacketTarget, PacketSizeRange};
use router_flood::config::ProtocolMix;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn test_complete_packet_workflow() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 1400), protocol_mix);
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target_port = 8080;
    
    // Build different packet types
    let packet_types = vec![
        PacketType::Udp,
        PacketType::TcpSyn,
        PacketType::TcpAck,
        PacketType::Icmp,
    ];
    
    for packet_type in packet_types {
        let result = builder.build_packet(packet_type, target_ip, target_port);
        assert!(result.is_ok());
        
        let (packet, protocol) = result.unwrap();
        assert!(!packet.is_empty());
        assert!(!protocol.is_empty());
    }
}

#[test]
fn test_ipv4_to_ipv6_transition() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(100, 200), protocol_mix);
    
    // Build IPv4 packets
    let ipv4_target = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ipv4_result = builder.build_packet(PacketType::Udp, ipv4_target, 80);
    assert!(ipv4_result.is_ok());
    
    // Build IPv6 packets
    let ipv6_target = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
    let ipv6_result = builder.build_packet(PacketType::Ipv6Udp, ipv6_target, 80);
    assert!(ipv6_result.is_ok());
    
    // Verify incompatible combinations fail
    assert!(builder.build_packet(PacketType::Udp, ipv6_target, 80).is_err());
    assert!(builder.build_packet(PacketType::Ipv6Udp, ipv4_target, 80).is_err());
}

#[test]
fn test_packet_size_consistency() {
    let protocol_mix = ProtocolMix::default();
    let target_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    
    // Test with fixed size range
    let mut builder = PacketBuilder::new(PacketSizeRange::new(100, 100), protocol_mix.clone());
    
    for _ in 0..10 {
        let result = builder.build_packet(PacketType::Udp, target_ip, 8080);
        if let Ok((packet, _)) = result {
            // Size should be consistent (within reasonable bounds for headers)
            assert!(packet.len() >= 100);
            assert!(packet.len() <= 200); // Allow for headers
        }
    }
    
    // Test with variable size range
    let mut builder = PacketBuilder::new(PacketSizeRange::new(100, 1000), protocol_mix);
    
    for _ in 0..10 {
        let result = builder.build_packet(PacketType::Udp, target_ip, 8080);
        if let Ok((packet, _)) = result {
            assert!(packet.len() >= 100);
            assert!(packet.len() <= 1100); // Allow for headers
        }
    }
}

#[test]
fn test_packet_target_workflow() {
    let target = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)),
        443
    );
    
    assert_eq!(target.port, 443);
    match target.ip {
        IpAddr::V4(addr) => assert_eq!(addr, Ipv4Addr::new(172, 16, 0, 1)),
        _ => panic!("Expected IPv4"),
    }
}

#[test]
fn test_zero_copy_vs_allocation() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(100, 200), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    // Test allocation method
    let alloc_result = builder.build_packet(PacketType::Udp, target_ip, 8080);
    assert!(alloc_result.is_ok());
    let (packet1, protocol1) = alloc_result.unwrap();
    
    // Test zero-copy method
    let mut buffer = vec![0u8; 1500];
    let zero_copy_result = builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        target_ip,
        8080
    );
    assert!(zero_copy_result.is_ok());
    let (size, protocol2) = zero_copy_result.unwrap();
    
    // Both methods should produce valid results
    assert!(!packet1.is_empty());
    assert!(size > 0);
    assert_eq!(protocol1, protocol2);
}

#[test]
fn test_multiple_ports_workflow() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 128), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    let ports = vec![80, 443, 8080, 8443, 3000, 5000];
    
    for port in ports {
        let result = builder.build_packet(PacketType::Udp, target_ip, port);
        assert!(result.is_ok(), "Failed to build packet for port {}", port);
    }
}