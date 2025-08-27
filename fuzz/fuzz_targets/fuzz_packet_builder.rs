//! Fuzz testing for packet building functionality
//!
//! This fuzzer tests the packet building system with random inputs
//! to ensure it never panics or produces invalid packets.

#![no_main]

use libfuzzer_sys::fuzz_target;
use router_flood::packet::*;
use router_flood::config::ProtocolMix;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug)]
struct FuzzInput {
    packet_size_min: u16,
    packet_size_max: u16,
    protocol_ratios: [u8; 6], // Will be normalized to create ProtocolMix
    target_ip_type: u8, // 0 = IPv4, 1 = IPv6
    target_ip_bytes: [u8; 16], // IPv4 uses first 4 bytes
    target_port: u16,
    packet_type_selector: u8,
    buffer_size: u16,
}

fuzz_target!(|input: FuzzInput| {
    // Ensure packet size range is valid
    let min_size = (input.packet_size_min as usize).max(20).min(1400);
    let max_size = (input.packet_size_max as usize).max(min_size + 1).min(1500);
    let packet_size_range = (min_size, max_size);
    
    // Create normalized protocol mix
    let total: u32 = input.protocol_ratios.iter().map(|&x| x as u32).sum();
    let protocol_mix = if total == 0 {
        // Fallback to UDP only if all ratios are zero
        ProtocolMix {
            udp_ratio: 1.0,
            tcp_syn_ratio: 0.0,
            tcp_ack_ratio: 0.0,
            icmp_ratio: 0.0,
            ipv6_ratio: 0.0,
            arp_ratio: 0.0,
        }
    } else {
        ProtocolMix {
            udp_ratio: input.protocol_ratios[0] as f64 / total as f64,
            tcp_syn_ratio: input.protocol_ratios[1] as f64 / total as f64,
            tcp_ack_ratio: input.protocol_ratios[2] as f64 / total as f64,
            icmp_ratio: input.protocol_ratios[3] as f64 / total as f64,
            ipv6_ratio: input.protocol_ratios[4] as f64 / total as f64,
            arp_ratio: input.protocol_ratios[5] as f64 / total as f64,
        }
    };
    
    // Create target IP
    let target_ip = if input.target_ip_type % 2 == 0 {
        // IPv4 - force into private range for safety
        let a = if input.target_ip_bytes[0] == 0 { 192 } else { input.target_ip_bytes[0] };
        let b = if a == 192 { 168 } else if a == 10 { input.target_ip_bytes[1] } else { 16 + (input.target_ip_bytes[1] % 16) };
        IpAddr::V4(Ipv4Addr::new(
            a,
            b,
            input.target_ip_bytes[2],
            input.target_ip_bytes[3].max(1), // Avoid .0 addresses
        ))
    } else {
        // IPv6 - use link-local for safety
        let mut addr = [0u16; 8];
        addr[0] = 0xfe80; // Link-local prefix
        for i in 1..8 {
            addr[i] = u16::from_be_bytes([
                input.target_ip_bytes[i * 2 - 2],
                input.target_ip_bytes[i * 2 - 1],
            ]);
        }
        IpAddr::V6(Ipv6Addr::from(addr))
    };
    
    let target_port = if input.target_port == 0 { 80 } else { input.target_port };
    
    // Test packet builder creation (should never panic)
    let mut builder = PacketBuilder::new(packet_size_range, protocol_mix);
    
    // Test packet type generation (should never panic)
    let packet_type = builder.next_packet_type_for_ip(target_ip);
    
    // Test regular packet building (should never panic)
    let _ = builder.build_packet(packet_type, target_ip, target_port);
    
    // Test zero-copy packet building with various buffer sizes
    let buffer_size = (input.buffer_size as usize).max(50).min(2000);
    let mut buffer = vec![0u8; buffer_size];
    let _ = builder.build_packet_into_buffer(&mut buffer, packet_type, target_ip, target_port);
    
    // Test with specific packet types to ensure robustness
    let packet_types = [
        PacketType::Udp,
        PacketType::TcpSyn,
        PacketType::TcpAck,
        PacketType::Icmp,
        PacketType::Ipv6Udp,
        PacketType::Ipv6Tcp,
        PacketType::Ipv6Icmp,
        PacketType::Arp,
    ];
    
    let selected_type = packet_types[input.packet_type_selector as usize % packet_types.len()];
    let _ = builder.build_packet(selected_type, target_ip, target_port);
    
    // Test with edge case buffer sizes
    for &size in &[1, 10, 20, 28, 64, 1500, 2000] {
        let mut small_buffer = vec![0u8; size];
        let _ = builder.build_packet_into_buffer(&mut small_buffer, selected_type, target_ip, target_port);
    }
});