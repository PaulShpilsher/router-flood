//! Basic smoke tests for router-flood
//!
//! These tests verify that the basic functionality works after refactoring.

use router_flood::config::{Config, Target, LoadConfig, Safety, Monitoring, Export, ExportFormat, ProtocolMix};
use router_flood::error::Result;
use router_flood::stats::Stats;
use router_flood::packet::{PacketBuilder, PacketType, PacketTarget};
use router_flood::network::target::MultiPortTarget;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

#[test]
fn test_config_creation() {
    let config = Config {
        target: Target {
            ip: "192.168.1.1".to_string(),
            ports: vec![80, 443],
            protocol_mix: ProtocolMix {
                udp: 40,
                tcp: 40,
                icmp: 10,
                ipv6: 5,
                arp: 5,
            },
        },
        attack: LoadConfig {
            threads: 4,
            packet_rate: 100.0,
            duration: Some(60),
            payload_size: 64,
            randomize_payload: true,
        },
        safety: Safety {
            dry_run: true,
            validate_config: true,
            rate_limit: Some(1000),
            stop_on_error: true,
            allow_public_ip: false,
        },
        monitoring: Monitoring {
            print_stats: true,
            stats_interval_seconds: 5,
        },
        export: Export {
            enabled: false,
            format: ExportFormat::Json,
            path: "stats.json".to_string(),
            interval_seconds: 60,
        },
    };
    
    assert_eq!(config.attack.threads, 4);
    assert_eq!(config.target.ports.len(), 2);
}

#[test]
fn test_stats_creation_and_updates() {
    let stats = Arc::new(Stats::new(None));
    
    // Test incrementing stats
    stats.increment_sent(100, "UDP");
    stats.increment_sent(200, "TCP");
    stats.increment_failed();
    
    assert!(stats.packets_sent() >= 2);
    assert!(stats.packets_failed() >= 1);
    assert!(stats.bytes_sent() >= 300);
}

#[test]
fn test_packet_target_creation() {
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target = PacketTarget::new(ip, 80);
    
    assert_eq!(target.ip, ip);
    assert_eq!(target.port, 80);
}

#[test]
fn test_multi_port_target() {
    let ports = vec![80, 443, 8080];
    let target = Arc::new(MultiPortTarget::new(ports.clone()));
    
    // Test round-robin port selection
    let mut selected_ports = vec![];
    for _ in 0..6 {
        selected_ports.push(target.next_port());
    }
    
    // Should cycle through all ports
    assert!(selected_ports.contains(&80));
    assert!(selected_ports.contains(&443));
    assert!(selected_ports.contains(&8080));
}

#[test]
fn test_packet_builder_creation() {
    let packet_size_range = (64, 1400);
    let protocol_mix = ProtocolMix {
        udp: 40,
        tcp: 40,
        icmp: 10,
        ipv6: 5,
        arp: 5,
    };
    
    let builder = PacketBuilder::new(packet_size_range, protocol_mix);
    
    // Builder should be created successfully
    // Actual packet building would require more setup
    assert_eq!(builder.packet_size_range(), packet_size_range);
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use router_flood::security::validation::{validate_target_ip, validate_comprehensive_security};
    
    #[test]
    fn test_private_ip_validation() {
        // Valid private IPs
        let private_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        assert!(validate_target_ip(&private_ip).is_ok());
        
        let private_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        assert!(validate_target_ip(&private_ip).is_ok());
        
        // Invalid public IP
        let public_ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        assert!(validate_target_ip(&public_ip).is_err());
    }
    
    #[test]
    fn test_comprehensive_validation() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ports = vec![80, 443];
        let threads = 4;
        let rate = 100;
        
        let result = validate_comprehensive_security(&ip, &ports, threads, rate);
        assert!(result.is_ok());
        
        // Test with excessive threads
        let result = validate_comprehensive_security(&ip, &ports, 200, rate);
        assert!(result.is_err());
    }
}