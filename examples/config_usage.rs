//! Example demonstrating configuration usage patterns
//!
//! This example shows how to use the configuration system
//! for building network testing tools.

use router_flood::config::default_config;
use router_flood::error::Result;

fn main() -> Result<()> {
    // Load default configuration
    let config = default_config();
    
    println!("=== Router Flood Configuration Example ===\n");
    
    // Display target configuration
    println!("Target Configuration:");
    println!("  IP: {}", config.target.ip);
    println!("  Ports: {:?}", config.target.ports);
    if let Some(ref iface) = config.target.interface {
        println!("  Interface: {}", iface);
    }
    println!();
    
    // Display load configuration
    println!("Load Configuration:");
    println!("  Threads: {}", config.attack.threads);
    println!("  Packet Rate: {} pps", config.attack.packet_rate);
    if let Some(duration) = config.attack.duration {
        println!("  Duration: {} seconds", duration);
    }
    println!("  Payload Size: {} bytes", config.attack.payload_size);
    println!();
    
    // Display protocol mix
    println!("Protocol Mix:");
    println!("  UDP: {:.1}%", config.target.protocol_mix.udp_ratio * 100.0);
    println!("  TCP SYN: {:.1}%", config.target.protocol_mix.tcp_syn_ratio * 100.0);
    println!("  TCP ACK: {:.1}%", config.target.protocol_mix.tcp_ack_ratio * 100.0);
    println!("  ICMP: {:.1}%", config.target.protocol_mix.icmp_ratio * 100.0);
    println!("  Custom: {:.1}%", config.target.protocol_mix.custom_ratio * 100.0);
    println!();
    
    // Display safety configuration
    println!("Safety Configuration:");
    println!("  Dry Run: {}", config.safety.dry_run);
    println!("  Rate Limit: {}", config.safety.rate_limit);
    if let Some(bandwidth) = config.safety.max_bandwidth_mbps {
        println!("  Max Bandwidth: {} Mbps", bandwidth);
    }
    println!("  Allow Localhost: {}", config.safety.allow_localhost);
    println!("  Require Confirmation: {}", config.safety.require_confirmation);
    println!();
    
    // Validate configuration
    println!("Configuration Validation:");
    
    // Check protocol ratios sum to 1.0
    let total_ratio = config.target.protocol_mix.udp_ratio 
        + config.target.protocol_mix.tcp_syn_ratio
        + config.target.protocol_mix.tcp_ack_ratio
        + config.target.protocol_mix.icmp_ratio
        + config.target.protocol_mix.custom_ratio;
    
    if (total_ratio - 1.0).abs() < 0.001 {
        println!("  ✅ Protocol ratios valid (sum = {:.3})", total_ratio);
    } else {
        println!("  ❌ Protocol ratios invalid (sum = {:.3}, expected 1.0)", total_ratio);
    }
    
    // Check safety settings
    if config.safety.dry_run {
        println!("  ✅ Running in dry-run mode (safe)");
    } else {
        println!("  ⚠️  Not in dry-run mode");
    }
    
    if config.safety.rate_limit {
        println!("  ✅ Rate limiting enabled");
    } else {
        println!("  ⚠️  Rate limiting disabled");
    }
    
    println!("\nNote: This example demonstrates reading and validating configuration.");
    println!("In production, use the configuration trait system for better abstraction.");
    
    Ok(())
}