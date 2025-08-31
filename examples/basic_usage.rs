//! Basic usage example for Router Flood

use router_flood::{
    config::{ConfigBuilder, default_config},
    stats::Stats,
    error::Result,
};

fn main() -> Result<()> {
    println!("🚀 Router Flood - Basic Usage Example");
    println!("=====================================\n");

    // Example 1: Default Configuration
    println!("1. Loading default configuration:");
    let config = default_config();
    println!("   Target: {}", config.target.ip);
    println!("   Ports: {:?}", config.target.ports);
    println!("   Threads: {}", config.attack.threads);
    println!("   Dry-run: {}\n", config.safety.dry_run);

    // Example 2: Building Custom Configuration
    println!("2. Building custom configuration:");
    let custom_config = ConfigBuilder::new()
        .target_ip("192.168.1.100")
        .target_ports(vec![80, 443, 8080])
        .threads(4)
        .packet_rate(1000.0)
        .dry_run(true)
        .build()?;
    
    println!("   Target: {}", custom_config.target.ip);
    println!("   Ports: {:?}", custom_config.target.ports);
    println!("   Packet rate: {} pps\n", custom_config.attack.packet_rate);

    // Example 3: Statistics Tracking
    println!("3. Statistics tracking:");
    let stats = Stats::new(None);
    
    // Simulate some packet activity
    stats.increment_sent(100, "UDP");
    stats.increment_sent(150, "TCP");
    stats.increment_failed();
    
    println!("   Packets sent: {}", stats.packets_sent());
    println!("   Packets failed: {}", stats.packets_failed());
    println!("   Bytes sent: {}", stats.bytes_sent());
    println!("   Session ID: {}\n", stats.session_id);

    // Example 4: Safety Features
    println!("4. Safety features:");
    println!("   ✅ Always operates in dry-run mode by default");
    println!("   ✅ Rate limiting enabled: {}", config.safety.rate_limit);
    if let Some(bandwidth) = config.safety.max_bandwidth_mbps {
        println!("   ✅ Max bandwidth: {} Mbps", bandwidth);
    }
    println!("   ✅ Private IP ranges only\n");

    println!("Example completed successfully!");
    Ok(())
}