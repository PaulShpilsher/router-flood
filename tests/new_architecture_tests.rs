//! Tests for the new architecture improvements

use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ConfigBuilder;
use router_flood::transport::{MockTransport, TransportLayer};
use std::net::IpAddr;

#[test]
fn test_config_builder_valid() {
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(vec![80, 443])
        .threads(4)
        .packet_rate(100)
        .build();
    
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.target.ip, "192.168.1.1");
    assert_eq!(config.target.ports, vec![80, 443]);
    assert_eq!(config.attack.threads, 4);
    assert_eq!(config.attack.packet_rate, 100);
}

#[test]
fn test_config_builder_invalid_ip() {
    let config = ConfigBuilder::new()
        .target_ip("8.8.8.8") // Public IP should fail
        .build();
    
    assert!(config.is_err());
}

#[test]
fn test_config_builder_invalid_limits() {
    let config = ConfigBuilder::new()
        .threads(200) // Exceeds MAX_THREADS
        .packet_rate(50000) // Exceeds MAX_PACKET_RATE
        .build();
    
    assert!(config.is_err());
}

#[test]
fn test_packet_builder_strategy_pattern() {
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Test UDP packet building
    let mut buffer = vec![0u8; 1500];
    let result = packet_builder.build_packet_into_buffer(
        &mut buffer, 
        PacketType::Udp, 
        target_ip, 
        80
    );
    
    assert!(result.is_ok());
    let (size, protocol) = result.unwrap();
    assert!(size > 0);
    assert_eq!(protocol, "UDP");
}

#[test]
fn test_packet_builder_protocol_selection() {
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 1.0, // 100% UDP for predictable testing
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let ipv4_target: IpAddr = "192.168.1.1".parse().unwrap();
    
    // With 100% UDP ratio, should always select UDP for IPv4
    for _ in 0..10 {
        let packet_type = packet_builder.next_packet_type_for_ip(ipv4_target);
        assert_eq!(packet_type, PacketType::Udp);
    }
}

#[test]
fn test_packet_builder_ipv6_selection() {
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 0.0,
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 1.0, // This doesn't matter for IPv6 targets
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let ipv6_target: IpAddr = "fe80::1".parse().unwrap();
    
    // For IPv6 targets, should only select IPv6 packet types
    for _ in 0..10 {
        let packet_type = packet_builder.next_packet_type_for_ip(ipv6_target);
        assert!(matches!(packet_type, 
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp
        ));
    }
}

#[test]
fn test_mock_transport() {
    let transport = MockTransport::new();
    
    assert!(transport.is_available());
    assert_eq!(transport.name(), "Mock");
    assert_eq!(transport.packets_sent(), 0);
    
    // Test packet sending
    let data = vec![0u8; 100];
    let target: IpAddr = "192.168.1.1".parse().unwrap();
    let result = transport.send_packet(&data, target, router_flood::transport::ChannelType::IPv4);
    
    assert!(result.is_ok());
    assert_eq!(transport.packets_sent(), 1);
}

#[test]
fn test_mock_transport_with_failures() {
    let transport = MockTransport::with_failures();
    
    let data = vec![0u8; 100];
    let target: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Send 100 packets, should get some failures
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for _ in 0..100 {
        let result = transport.send_packet(&data, target, router_flood::transport::ChannelType::IPv4);
        if result.is_ok() {
            success_count += 1;
        } else {
            failure_count += 1;
        }
    }
    
    // Should have some successes and some failures
    assert!(success_count > 0);
    assert!(failure_count > 0);
    assert_eq!(success_count + failure_count, 100);
}

#[test]
fn test_packet_type_display() {
    assert_eq!(format!("{}", PacketType::Udp), "UDP");
    assert_eq!(format!("{}", PacketType::TcpSyn), "TCP-SYN");
    assert_eq!(format!("{}", PacketType::TcpAck), "TCP-ACK");
    assert_eq!(format!("{}", PacketType::Icmp), "ICMP");
    assert_eq!(format!("{}", PacketType::Ipv6Udp), "IPv6-UDP");
    assert_eq!(format!("{}", PacketType::Ipv6Tcp), "IPv6-TCP");
    assert_eq!(format!("{}", PacketType::Ipv6Icmp), "IPv6-ICMP");
    assert_eq!(format!("{}", PacketType::Arp), "ARP");
}

#[test]
fn test_packet_type_properties() {
    assert!(PacketType::Udp.is_ipv4());
    assert!(!PacketType::Udp.is_ipv6());
    
    assert!(PacketType::Ipv6Udp.is_ipv6());
    assert!(!PacketType::Ipv6Udp.is_ipv4());
    
    assert_eq!(PacketType::Udp.protocol_name(), "UDP");
    assert_eq!(PacketType::TcpSyn.protocol_name(), "TCP");
    assert_eq!(PacketType::Ipv6Udp.protocol_name(), "IPv6");
}