//! Example showing how to monitor and export statistics
//!
//! Run with: cargo run --example monitoring

use router_flood::{
    config::ConfigBuilder,
    stats::Stats,
    error::Result,
};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("Router Flood - Monitoring and Statistics Example\n");
    
    // Create configuration
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(vec![80, 443])
        .threads(4)
        .packet_rate(500.0)
        .dry_run(true)  // Safe mode for example
        .build()?;
    
    println!("Configuration:");
    println!("  Target: {}", config.target.ip);
    println!("  Ports: {:?}", config.target.ports);
    println!("  Threads: {}", config.attack.threads);
    println!("  Rate: {} pps per thread", config.attack.packet_rate);
    println!("  Mode: Dry-run (simulation)\n");
    
    // Create statistics collector
    let stats = Arc::new(Stats::new(None));
    let stats_monitor = Arc::clone(&stats);
    
    // Spawn monitoring thread
    let monitor_handle = thread::spawn(move || {
        println!("Starting real-time monitoring...\n");
        let start = Instant::now();
        
        for i in 0..10 {
            thread::sleep(Duration::from_secs(1));
            
            // Get current statistics
            let packets_sent = stats_monitor.packets_sent();
            let packets_failed = stats_monitor.packets_failed();
            let bytes_sent = stats_monitor.bytes_sent();
            let elapsed = start.elapsed().as_secs();
            
            // Calculate rates
            let pps = if elapsed > 0 {
                packets_sent / elapsed
            } else {
                0
            };
            
            let mbps = if elapsed > 0 {
                (bytes_sent as f64 * 8.0) / (elapsed as f64 * 1_000_000.0)
            } else {
                0.0
            };
            
            // Display statistics
            println!("╔══════════════════════════════════════════════════╗");
            println!("║ Update #{:<3} - Elapsed: {:>3}s                     ║", i + 1, elapsed);
            println!("╠══════════════════════════════════════════════════╣");
            println!("║ Packets Sent:     {:>10}                    ║", format_number(packets_sent));
            println!("║ Packets Failed:   {:>10}                    ║", format_number(packets_failed));
            println!("║ Bytes Sent:       {:>10}                    ║", format_bytes(bytes_sent));
            println!("║ Rate (pps):       {:>10}                    ║", format_number(pps));
            println!("║ Throughput:       {:>10.2} Mbps              ║", mbps);
            println!("╚══════════════════════════════════════════════════╝\n");
        }
        
        println!("Monitoring complete.");
    });
    
    // Simulate packet sending
    println!("Simulating packet generation...\n");
    thread::sleep(Duration::from_millis(500));
    
    for _ in 0..10 {
        // Simulate sending packets from multiple threads
        for _ in 0..config.attack.threads {
            for _ in 0..50 {  // Batch of 50 packets
                // Simulate successful packet
                if rand::random::<f32>() > 0.02 {  // 98% success rate
                    let packet_size = 64 + (rand::random::<u64>() % 1336);  // 64-1400 bytes
                    stats.increment_sent(packet_size, "TCP");
                } else {
                    // Simulate failed packet
                    stats.increment_failed();
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    // Wait for monitoring to complete
    monitor_handle.join().unwrap();
    
    // Final statistics
    println!("\n═══════════════════════════════════════════════════");
    println!("                 FINAL STATISTICS                   ");
    println!("═══════════════════════════════════════════════════\n");
    
    let total_sent = stats.packets_sent();
    let total_failed = stats.packets_failed();
    let total_bytes = stats.bytes_sent();
    let success_rate = if total_sent + total_failed > 0 {
        (total_sent as f64 / (total_sent + total_failed) as f64) * 100.0
    } else {
        0.0
    };
    
    println!("Session ID: {}", stats.session_id);
    println!("Total Packets Sent: {}", format_number(total_sent));
    println!("Total Packets Failed: {}", format_number(total_failed));
    println!("Total Bytes Sent: {}", format_bytes(total_bytes));
    println!("Success Rate: {:.2}%", success_rate);
    println!("Average Packet Size: {} bytes", 
        if total_sent > 0 { total_bytes / total_sent } else { 0 }
    );
    
    // Export options
    println!("\n═══════════════════════════════════════════════════");
    println!("                  EXPORT OPTIONS                    ");
    println!("═══════════════════════════════════════════════════\n");
    
    println!("Statistics can be exported in multiple formats:");
    println!();
    println!("1. JSON Export (--export json):");
    println!("   - Machine-readable format");
    println!("   - Includes all metrics and metadata");
    println!("   - Suitable for automated analysis");
    println!();
    println!("2. CSV Export (--export csv):");
    println!("   - Spreadsheet-compatible format");
    println!("   - Time-series data");
    println!("   - Easy to graph and analyze");
    println!();
    println!("3. Both Formats (--export both):");
    println!("   - Exports both JSON and CSV");
    println!("   - Best for comprehensive analysis");
    
    println!("\n═══════════════════════════════════════════════════");
    
    Ok(())
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.2} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} B", bytes)
    }
}