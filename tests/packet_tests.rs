//! Packet module unit tests

use router_flood::packet::{PacketBuilder, PacketTarget, PacketType, PacketSizeRange};
use router_flood::config::ProtocolMix;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn test_packet_target_creation() {
    let ipv4 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target = PacketTarget::new(ipv4, 8080);
    
    match target.ip {
        IpAddr::V4(addr) => assert_eq!(addr, Ipv4Addr::new(192, 168, 1, 1)),
        _ => panic!("Expected IPv4 address"),
    }
    assert_eq!(target.port, 8080);
}

#[test]
fn test_packet_builder_creation() {
    let protocol_mix = ProtocolMix {
        udp_ratio: 0.4,
        tcp_syn_ratio: 0.3,
        tcp_ack_ratio: 0.2,
        icmp_ratio: 0.1,
        custom_ratio: 0.0,
    };
    
    let _builder = PacketBuilder::new(PacketSizeRange::new(64, 1400), protocol_mix);
}

#[test]
fn test_build_udp_packet() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 128), protocol_mix);
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target_port = 8080;
    
    let result = builder.build_packet(PacketType::Udp, target_ip, target_port);
    assert!(result.is_ok());
    
    let (packet, protocol) = result.unwrap();
    assert!(!packet.is_empty());
    assert_eq!(protocol, "UDP");
}

#[test]
fn test_build_tcp_syn_packet() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 128), protocol_mix);
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let target_port = 443;
    
    let result = builder.build_packet(PacketType::TcpSyn, target_ip, target_port);
    assert!(result.is_ok());
    
    let (packet, protocol) = result.unwrap();
    assert!(!packet.is_empty());
    assert_eq!(protocol, "TCP");
}

#[test]
fn test_build_icmp_packet() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 128), protocol_mix);
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
    let target_port = 0; // ICMP doesn't use ports
    
    let result = builder.build_packet(PacketType::Icmp, target_ip, target_port);
    assert!(result.is_ok());
    
    let (packet, protocol) = result.unwrap();
    assert!(!packet.is_empty());
    assert_eq!(protocol, "ICMP");
}

#[test]
fn test_build_packet_into_buffer() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 128), protocol_mix);
    
    let mut buffer = vec![0u8; 1500];
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target_port = 8080;
    
    let result = builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        target_ip,
        target_port
    );
    
    assert!(result.is_ok());
    let (size, protocol) = result.unwrap();
    assert!(size > 0);
    assert!(size <= 1500);
    assert_eq!(protocol, "UDP");
}

#[test]
fn test_ipv6_packet_building() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 128), protocol_mix);
    
    let target_ip = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
    let target_port = 443;
    
    // Test IPv6 UDP
    let result = builder.build_packet(PacketType::Ipv6Udp, target_ip, target_port);
    assert!(result.is_ok());
    let (packet, protocol) = result.unwrap();
    assert!(!packet.is_empty());
    assert_eq!(protocol, "IPv6");
    
    // Test IPv6 TCP
    let result = builder.build_packet(PacketType::Ipv6Tcp, target_ip, target_port);
    assert!(result.is_ok());
    let (packet, protocol) = result.unwrap();
    assert!(!packet.is_empty());
    assert_eq!(protocol, "IPv6");
    
    // Test IPv6 ICMP
    let result = builder.build_packet(PacketType::Ipv6Icmp, target_ip, 0);
    assert!(result.is_ok());
    let (packet, protocol) = result.unwrap();
    assert!(!packet.is_empty());
    assert_eq!(protocol, "IPv6");
}

#[test]
fn test_packet_size_constraints() {
    // Test small packets
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(20, 64), protocol_mix);
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let result = builder.build_packet(PacketType::Udp, target_ip, 8080);
    assert!(result.is_ok());
    
    let (packet, _) = result.unwrap();
    // Packet includes headers, so might be slightly larger than payload size
    assert!(packet.len() <= 128);  // Allow for headers
    
    // Test large packets
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(1000, 1400), protocol_mix);
    let result = builder.build_packet(PacketType::Udp, target_ip, 8080);
    assert!(result.is_ok());
    
    let (packet, _) = result.unwrap();
    assert!(packet.len() >= 1000);
    assert!(packet.len() <= 1400);
}

#[test]
fn test_incompatible_protocol_ip_version() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 128), protocol_mix);
    
    // Try to build IPv4 packet type with IPv6 address
    let ipv6_target = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
    let result = builder.build_packet(PacketType::Udp, ipv6_target, 8080);
    assert!(result.is_err());
    
    // Try to build IPv6 packet type with IPv4 address
    let ipv4_target = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let result = builder.build_packet(PacketType::Ipv6Udp, ipv4_target, 8080);
    assert!(result.is_err());
}

#[test]
fn test_multiple_packet_types() {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 256), protocol_mix);
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    let packet_types = vec![
        PacketType::Udp,
        PacketType::TcpSyn,
        PacketType::TcpAck,
        PacketType::Icmp,
    ];
    
    for packet_type in packet_types {
        let result = builder.build_packet(packet_type, target_ip, 8080);
        assert!(result.is_ok(), "Failed to build {:?} packet", packet_type);
    }
}