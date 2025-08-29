//! Example demonstrating configuration usage patterns
//!
//! This example shows how to use the configuration system
//! for building network testing tools.

use router_flood::config::get_default_config;
use router_flood::error::Result;

fn main() -> Result<()> {
    // Load default configuration
    let config = get_default_config();
    
    println!("=== Router Flood Configuration Example ===\n");
    
    // Display target configuration
    println!("Target Configuration:");
    println!("  IP: {}", config.target.ip);
    println!("  Ports: {:?}", config.target.ports);
    if let Some(ref iface) = config.target.interface {
        println!("  Interface: {}", iface);
    }
    println!();
    
    // Display attack configuration
    println!("Attack Configuration:");
    println!("  Threads: {}", config.attack.threads);
    println!("  Packet Rate: {} pps", config.attack.packet_rate);
    if let Some(duration) = config.attack.duration {
        println!("  Duration: {} seconds", duration);
    }
    println!("  Packet Size Range: {:?}", config.attack.packet_size_range);
    println!();
    
    // Display protocol mix
    println!("Protocol Mix:");
    println!("  UDP: {:.1}%", config.target.protocol_mix.udp_ratio * 100.0);
    println!("  TCP SYN: {:.1}%", config.target.protocol_mix.tcp_syn_ratio * 100.0);
    println!("  TCP ACK: {:.1}%", config.target.protocol_mix.tcp_ack_ratio * 100.0);
    println!("  ICMP: {:.1}%", config.target.protocol_mix.icmp_ratio * 100.0);
    println!("  IPv6: {:.1}%", config.target.protocol_mix.ipv6_ratio * 100.0);
    println!("  ARP: {:.1}%", config.target.protocol_mix.arp_ratio * 100.0);
    println!();
    
    // Display safety configuration
    println!("Safety Configuration:");
    println!("  Max Threads: {}", config.safety.max_threads);
    println!("  Max Packet Rate: {}", config.safety.max_packet_rate);
    println!("  Require Private Ranges: {}", config.safety.require_private_ranges);
    println!("  Dry Run: {}", config.safety.dry_run);
    println!("  Perfect Simulation: {}", config.safety.perfect_simulation);
    println!();
    
    // Validate configuration
    println!("Configuration Validation:");
    
    // Check protocol ratios sum to 1.0
    let total_ratio = config.target.protocol_mix.udp_ratio 
        + config.target.protocol_mix.tcp_syn_ratio
        + config.target.protocol_mix.tcp_ack_ratio
        + config.target.protocol_mix.icmp_ratio
        + config.target.protocol_mix.ipv6_ratio
        + config.target.protocol_mix.arp_ratio;
    
    if (total_ratio - 1.0).abs() < 0.001 {
        println!("  ✅ Protocol ratios valid (sum = {:.3})", total_ratio);
    } else {
        println!("  ❌ Protocol ratios invalid (sum = {:.3}, expected 1.0)", total_ratio);
    }
    
    // Check safety limits
    if config.attack.threads <= config.safety.max_threads {
        println!("  ✅ Thread count within limits");
    } else {
        println!("  ❌ Thread count exceeds safety limit");
    }
    
    if config.attack.packet_rate <= config.safety.max_packet_rate {
        println!("  ✅ Packet rate within limits");
    } else {
        println!("  ❌ Packet rate exceeds safety limit");
    }
    
    println!("\nNote: This example demonstrates reading and validating configuration.");
    println!("In production, use the configuration trait system for better abstraction.");
    
    Ok(())
}