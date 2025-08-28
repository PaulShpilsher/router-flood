//! Demonstration of the new architecture improvements

#![allow(clippy::uninlined_format_args)]
//! 
//! This example shows how the refactored code would work with:
//! - Strategy pattern for packet building
//! - Configuration builder with validation
//! - Trait-based abstractions
//! 
//! Run with: cargo run --example new_architecture_demo

use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ConfigBuilder;
use router_flood::transport::{MockTransport, TransportLayer};
use std::net::IpAddr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Router Flood - New Architecture Demonstration");
    println!("================================================");
    
    // 1. Demonstrate the new configuration builder with validation
    println!("\n1. Configuration Builder with Validation:");
    
    let config_result = ConfigBuilder::new()
        .target_ip("192.168.1.1")  // Valid private IP
        .target_ports(vec![80, 443, 8080])
        .threads(4)
        .packet_rate(100)
        .packet_size_range(64, 1400)
        .dry_run(true)
        .build();
    
    match config_result {
        Ok(config) => {
            println!("✅ Configuration built successfully!");
            println!("   Target: {}", config.target.ip);
            println!("   Ports: {:?}", config.target.ports);
            println!("   Threads: {}", config.attack.threads);
            println!("   Rate: {} pps", config.attack.packet_rate);
        }
        Err(e) => {
            println!("❌ Configuration validation failed: {}", e);
        }
    }
    
    // 2. Demonstrate invalid configuration
    println!("\n2. Invalid Configuration Handling:");
    
    let invalid_config = ConfigBuilder::new()
        .target_ip("8.8.8.8")  // Invalid - public IP
        .threads(200)          // Invalid - exceeds limit
        .packet_rate(50000)    // Invalid - exceeds limit
        .build();
    
    match invalid_config {
        Ok(_) => println!("❌ This should have failed!"),
        Err(e) => println!("✅ Correctly rejected invalid config: {}", e),
    }
    
    // 3. Demonstrate the new packet builder with strategy pattern
    println!("\n3. Strategy Pattern Packet Building:");
    
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".parse()?;
    
    // Build different packet types
    let packet_types = [
        PacketType::Udp,
        PacketType::TcpSyn,
        PacketType::Icmp,
    ];
    
    for packet_type in packet_types {
        let mut buffer = vec![0u8; 1500];
        match packet_builder.build_packet_into_buffer(&mut buffer, packet_type, target_ip, 80) {
            Ok((size, protocol)) => {
                println!("✅ Built {} packet: {} bytes ({})", packet_type, size, protocol);
            }
            Err(e) => {
                println!("❌ Failed to build {} packet: {}", packet_type, e);
            }
        }
    }
    
    // 4. Demonstrate protocol selection based on target IP
    println!("\n4. Intelligent Protocol Selection:");
    
    let ipv4_target: IpAddr = "192.168.1.1".parse()?;
    let ipv6_target: IpAddr = "fe80::1".parse()?;
    
    for _ in 0..5 {
        let ipv4_packet = packet_builder.next_packet_type_for_ip(ipv4_target);
        let ipv6_packet = packet_builder.next_packet_type_for_ip(ipv6_target);
        
        println!("   IPv4 target -> {} | IPv6 target -> {}", ipv4_packet, ipv6_packet);
    }
    
    // 5. Demonstrate mock transport for testing
    println!("\n5. Mock Transport for Testing:");
    
    let mock_transport = MockTransport::new();
    println!("✅ Mock transport created for testing");
    println!("   Available: {}", mock_transport.is_available());
    println!("   Name: {}", mock_transport.name());
    println!("   Packets sent: {}", mock_transport.packets_sent());
    
    println!("\n🎉 Architecture demonstration completed!");
    println!("\nKey Improvements Demonstrated:");
    println!("• Strategy pattern for extensible packet building");
    println!("• Builder pattern with comprehensive validation");
    println!("• Trait-based abstractions for testing");
    println!("• Intelligent protocol selection");
    println!("• Type safety and error handling");
    
    Ok(())
}