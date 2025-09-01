//! Examples of different stress testing scenarios
//!
//! Run with: cargo run --example stress_testing

use router_flood::{
    config::{ConfigBuilder, ProtocolMix},
    error::Result,
};

fn main() -> Result<()> {
    println!("Router Flood - Stress Testing Scenarios\n");
    
    // Scenario 1: Web Server Load Test
    println!("1. Web Server Load Test Configuration:");
    let web_config = ConfigBuilder::new()
        .target_ip("192.168.1.100")
        .target_ports(vec![80, 443, 8080])
        .threads(8)
        .packet_rate(1000.0)
        .duration(300)
        .build()?;
    
    println!("   Target: {} ports {:?}", web_config.target.ip, web_config.target.ports);
    println!("   Load: {} threads × {} pps = {} total pps", 
        web_config.attack.threads, 
        web_config.attack.packet_rate,
        web_config.attack.threads as f64 * web_config.attack.packet_rate
    );
    println!("   Duration: 5 minutes\n");
    
    // Scenario 2: DNS Server Stress Test
    println!("2. DNS Server Stress Test:");
    let dns_config = ConfigBuilder::new()
        .target_ip("192.168.1.53")
        .target_ports(vec![53])
        .threads(4)
        .packet_rate(3000.0)
        // Note: protocol_mix would need to be set via config file
        // as ConfigBuilder doesn't have this method yet
        .build()?;
    
    println!("   Target: DNS server at {}", dns_config.target.ip);
    println!("   Protocol mix: 80% UDP, 20% TCP");
    println!("   Load: {} pps total\n", 
        dns_config.attack.threads as f64 * dns_config.attack.packet_rate
    );
    
    // Scenario 3: Database Server Test
    println!("3. Database Server Resilience Test:");
    let db_ports = vec![
        3306,  // MySQL/MariaDB
        5432,  // PostgreSQL
        27017, // MongoDB
        6379,  // Redis
    ];
    
    let db_config = ConfigBuilder::new()
        .target_ip("192.168.1.200")
        .target_ports(db_ports.clone())
        .threads(6)
        .packet_rate(500.0)
        .build()?;
    
    println!("   Testing database ports: {:?}", db_ports);
    println!("   Moderate load: {} total pps", 
        db_config.attack.threads as f64 * db_config.attack.packet_rate
    );
    println!("   Suitable for: Connection pool testing\n");
    
    // Scenario 4: Gradual Load Increase
    println!("4. Gradual Load Increase Pattern:");
    let load_stages = vec![
        (2, 100.0, "Baseline"),
        (4, 500.0, "Normal load"),
        (6, 1000.0, "High load"),
        (8, 2000.0, "Stress test"),
        (10, 5000.0, "Breaking point"),
    ];
    
    for (threads, rate, description) in load_stages {
        let total_pps = threads as f64 * rate;
        println!("   Stage: {} - {} threads × {} pps = {} total pps",
            description, threads, rate, total_pps
        );
    }
    println!();
    
    // Scenario 5: Bandwidth-Limited Test
    println!("5. Bandwidth-Limited Test:");
    let _bandwidth_config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(vec![80, 443])
        .threads(4)
        .packet_rate(1000.0)
        // Note: max_bandwidth_mbps would need to be set via config file
        .build()?;
    
    println!("   Bandwidth cap: 10 Mbps");
    println!("   Prevents network saturation");
    println!("   Ideal for: Shared network environments\n");
    
    // Scenario 6: Multi-Service Scan
    println!("6. Multi-Service Comprehensive Test:");
    let services = vec![
        (22, "SSH"),
        (80, "HTTP"),
        (443, "HTTPS"),
        (3306, "MySQL"),
        (5432, "PostgreSQL"),
        (6379, "Redis"),
        (8080, "HTTP Alt"),
        (9000, "Custom App"),
    ];
    
    let ports: Vec<u16> = services.iter().map(|(p, _)| *p).collect();
    let multi_config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(ports)
        .threads(8)
        .packet_rate(200.0)
        .build()?;
    
    println!("   Testing services:");
    for (port, service) in &services {
        println!("     - Port {}: {}", port, service);
    }
    println!("   Total load: {} pps distributed across all ports\n",
        multi_config.attack.threads as f64 * multi_config.attack.packet_rate
    );
    
    // Scenario 7: Custom Protocol Mix
    println!("7. Custom Protocol Mix for Realistic Traffic:");
    let realistic_mix = ProtocolMix {
        udp_ratio: 0.3,      // 30% UDP (DNS, streaming)
        tcp_syn_ratio: 0.4,   // 40% TCP SYN (new connections)
        tcp_ack_ratio: 0.25,  // 25% TCP ACK (established connections)
        icmp_ratio: 0.05,     // 5% ICMP (ping, traceroute)
        custom_ratio: 0.0,
    };
    
    println!("   UDP: {}% - Simulates DNS, VoIP, streaming", (realistic_mix.udp_ratio * 100.0) as u8);
    println!("   TCP SYN: {}% - New connection attempts", (realistic_mix.tcp_syn_ratio * 100.0) as u8);
    println!("   TCP ACK: {}% - Established connections", (realistic_mix.tcp_ack_ratio * 100.0) as u8);
    println!("   ICMP: {}% - Network diagnostics", (realistic_mix.icmp_ratio * 100.0) as u8);
    println!();
    
    // Safety recommendations
    println!("⚠️  Safety Recommendations:");
    println!("   1. Always start with --dry-run to validate configuration");
    println!("   2. Begin with low packet rates and gradually increase");
    println!("   3. Monitor target system resources during testing");
    println!("   4. Set reasonable duration limits");
    println!("   5. Export statistics for post-test analysis");
    println!("   6. Ensure you have authorization before testing");
    
    Ok(())
}