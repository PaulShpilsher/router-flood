//! Basic packet module tests
//!
//! Tests for packet type definitions and basic functionality.

use router_flood::packet::*;

#[test]
fn test_packet_type_display() {
    // Test that PacketType enum variants can be created and displayed
    assert_eq!(PacketType::Udp.to_string(), "UDP");
    assert_eq!(PacketType::TcpSyn.to_string(), "TCP-SYN");
    assert_eq!(PacketType::TcpAck.to_string(), "TCP-ACK");
    assert_eq!(PacketType::Icmp.to_string(), "ICMP");
    assert_eq!(PacketType::Ipv6Udp.to_string(), "IPv6-UDP");
    assert_eq!(PacketType::Ipv6Tcp.to_string(), "IPv6-TCP");
    assert_eq!(PacketType::Ipv6Icmp.to_string(), "IPv6-ICMP");
    assert_eq!(PacketType::Arp.to_string(), "ARP");
}

#[test]
fn test_packet_builder_creation() {
    use router_flood::config::ProtocolMix;
    
    let protocol_mix = ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    // Test that PacketBuilder can be created
    let _builder = PacketBuilder::new((64, 1500), protocol_mix);
    
    // Should not panic during creation
}

#[test]
fn test_packet_builder_packet_generation() {
    use router_flood::config::ProtocolMix;
    
    let protocol_mix = ProtocolMix {
        udp_ratio: 1.0, // Only UDP for predictable testing
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut builder = PacketBuilder::new((64, 1500), protocol_mix);
    
    // Test that we can generate packet types
    for _ in 0..10 {
        let packet_type = builder.next_packet_type();
        // With 100% UDP ratio, should always return UDP
        assert_eq!(packet_type, PacketType::Udp);
    }
}

#[test]
fn test_zero_copy_packet_building() {
    use router_flood::config::ProtocolMix;
    use std::net::{IpAddr, Ipv4Addr};
    
    let protocol_mix = ProtocolMix {
        udp_ratio: 1.0,
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut builder = PacketBuilder::new((64, 1500), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target_port = 80;
    
    // Test zero-copy packet building
    let mut buffer = vec![0u8; 1500]; // Large enough buffer
    
    let result = builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        target_ip,
        target_port,
    );
    
    assert!(result.is_ok());
    let (packet_size, protocol_name) = result.unwrap();
    
    assert_eq!(protocol_name, "UDP");
    assert!(packet_size > 28); // At least IP (20) + UDP (8) headers
    assert!(packet_size <= buffer.len());
    
    // Verify that the buffer was actually written to
    let packet_data = &buffer[..packet_size];
    assert!(packet_data.iter().any(|&b| b != 0)); // Should have non-zero bytes
}

#[test]
fn test_zero_copy_buffer_too_small() {
    use router_flood::config::ProtocolMix;
    use std::net::{IpAddr, Ipv4Addr};
    
    let protocol_mix = ProtocolMix {
        udp_ratio: 1.0,
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut builder = PacketBuilder::new((64, 1500), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target_port = 80;
    
    // Test with buffer that's too small
    let mut small_buffer = vec![0u8; 10]; // Too small for any packet
    
    let result = builder.build_packet_into_buffer(
        &mut small_buffer,
        PacketType::Udp,
        target_ip,
        target_port,
    );
    
    assert!(result.is_err());
    let error_msg = result.err().unwrap();
    assert!(error_msg.contains("Buffer too small"));
}

#[test]
fn test_zero_copy_vs_allocation() {
    use router_flood::config::ProtocolMix;
    use std::net::{IpAddr, Ipv4Addr};
    
    let protocol_mix = ProtocolMix {
        tcp_syn_ratio: 1.0, // Use TCP for consistent packet size
        udp_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut builder1 = PacketBuilder::new((64, 1500), protocol_mix.clone());
    let mut builder2 = PacketBuilder::new((64, 1500), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target_port = 443;
    
    // Test traditional allocation method
    let (allocated_packet, protocol1) = builder1.build_packet(
        PacketType::TcpSyn,
        target_ip,
        target_port,
    ).unwrap();
    
    // Test zero-copy method
    let mut buffer = vec![0u8; 1500];
    let (packet_size, protocol2) = builder2.build_packet_into_buffer(
        &mut buffer,
        PacketType::TcpSyn,
        target_ip,
        target_port,
    ).unwrap();
    
    assert_eq!(protocol1, protocol2);
    assert_eq!(allocated_packet.len(), packet_size);
    
    // Note: We can't directly compare packet contents because RNG makes them different,
    // but we can verify they have the same structure
    assert_eq!(allocated_packet.len(), packet_size);
}
