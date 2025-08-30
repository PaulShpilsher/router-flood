//! Advanced usage examples for Router Flood

#![allow(clippy::uninlined_format_args)]
//!
//! This example demonstrates advanced features including SIMD optimizations,
//! Prometheus metrics, and performance tuning.

use router_flood::{
    config::ConfigBuilder,
    performance::{SimdPacketBuilder, NumaBufferPool, CpuAffinityManager},
    monitoring::PrometheusExporter,
    security::{CapabilityManager, TamperProofAuditLog},
    error::Result,
};

use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Router Flood - Advanced Usage Examples");
    println!("==========================================");

    // Example 1: SIMD Performance Optimization
    simd_performance_example().await?;

    // Example 2: Advanced Buffer Management
    buffer_pool_example().await?;

    // Example 3: CPU Affinity and NUMA Optimization
    cpu_affinity_example().await?;

    // Example 4: Prometheus Metrics Export
    prometheus_metrics_example().await?;

    // Example 5: Security and Audit Logging
    security_audit_example().await?;

    // Example 6: Performance Benchmarking
    performance_benchmark_example().await?;

    println!("\n✅ All advanced examples completed successfully!");
    Ok(())
}

/// Example 1: SIMD-optimized packet generation
async fn simd_performance_example() -> Result<()> {
    println!("\n⚡ Example 1: SIMD Performance Optimization");
    println!("-------------------------------------------");

    let mut simd_builder = SimdPacketBuilder::new();
    let perf_info = simd_builder.get_performance_info();

    println!("SIMD Capabilities:");
    println!("  Available: {}", perf_info.simd_available);
    println!("  Instruction Set: {}", perf_info.instruction_set);
    println!("  Vector Width: {} bytes", perf_info.vector_width);

    // Demonstrate SIMD payload generation
    let payload_sizes = [64, 256, 1024, 1400];
    
    for &size in &payload_sizes {
        let start = Instant::now();
        let mut buffer = vec![0u8; size];
        
        // Fill with SIMD-optimized random data
        simd_builder.fill_payload_simd(&mut buffer)?;
        
        let duration = start.elapsed();
        println!("  {} bytes payload: {:.2}μs ({:.1} MB/s)", 
            size, 
            duration.as_micros(),
            (size as f64) / (duration.as_secs_f64() * 1_000_000.0)
        );
    }

    // Demonstrate UDP packet building with SIMD
    let mut packet_buffer = vec![0u8; 1500];
    let start = Instant::now();
    
    let packet_size = simd_builder.build_udp_packet_simd(
        &mut packet_buffer,
        [192, 168, 1, 100], // Source IP
        [192, 168, 1, 1],   // Destination IP
        12345,              // Source port
        80,                 // Destination port
        1000,               // Payload size
    )?;
    
    let duration = start.elapsed();
    println!("\nSIMD UDP Packet Generation:");
    println!("  Packet size: {} bytes", packet_size);
    println!("  Generation time: {:.2}μs", duration.as_micros());
    println!("  Throughput: {:.1} Mpps", 1.0 / duration.as_secs_f64() / 1_000_000.0);

    Ok(())
}

/// Example 2: Advanced buffer pool management
async fn buffer_pool_example() -> Result<()> {
    println!("\n🗄️  Example 2: Advanced Buffer Management");
    println!("----------------------------------------");

    // Create buffer pool with custom size classes
    let size_classes = vec![64, 128, 256, 512, 1024, 1500];
    let pool = NumaBufferPool::with_size_classes(size_classes, 100);

    // Warm up the pool
    println!("Warming up buffer pool...");
    pool.warm_up(10)?;

    let initial_stats = pool.get_stats();
    println!("Initial pool statistics:");
    println!("  Total allocated: {}", initial_stats.total_allocated);
    println!("  Memory usage: {:.2} KB", initial_stats.memory_usage as f64 / 1024.0);

    // Simulate buffer usage patterns
    let mut buffers = Vec::new();
    let start = Instant::now();

    // Allocate buffers of various sizes
    for _ in 0..1000 {
        for &size in &[64, 256, 512, 1024] {
            if let Some(buffer) = pool.get_buffer(size) {
                buffers.push(buffer);
            }
        }
    }

    let allocation_time = start.elapsed();

    // Return buffers to pool
    let start = Instant::now();
    for buffer in buffers {
        pool.return_buffer(buffer);
    }
    let return_time = start.elapsed();

    let final_stats = pool.get_stats();
    println!("\nBuffer pool performance:");
    println!("  Allocation time: {:.2}ms", allocation_time.as_millis());
    println!("  Return time: {:.2}ms", return_time.as_millis());
    println!("  Hit rate: {:.1}%", final_stats.hit_rate);
    println!("  Total hits: {}", final_stats.total_hits);
    println!("  Total misses: {}", final_stats.total_misses);

    // Show size class statistics
    println!("\nSize class statistics:");
    for size_stat in &final_stats.size_class_stats {
        println!("  {} bytes: {} allocated, {} available, {} hits",
            size_stat.size,
            size_stat.allocated,
            size_stat.available,
            size_stat.hits
        );
    }

    Ok(())
}

/// Example 3: CPU affinity and NUMA optimization
async fn cpu_affinity_example() -> Result<()> {
    println!("\n🖥️  Example 3: CPU Affinity and NUMA Optimization");
    println!("------------------------------------------------");

    let cpu_manager = CpuAffinityManager::new()?;
    let topology = cpu_manager.topology();

    println!("System topology:");
    println!("  Total CPUs: {}", topology.total_cpus);
    println!("  NUMA nodes: {}", topology.numa_nodes.len());
    println!("  Hyperthreading: {}", if topology.hyperthreading_enabled { "Enabled" } else { "Disabled" });

    // Show NUMA node details
    for node in &topology.numa_nodes {
        println!("\nNUMA Node {}:", node.node_id);
        println!("  CPUs: {:?}", node.cpus);
        if let Some(total_mem) = node.memory_total {
            println!("  Memory: {:.2} GB total", total_mem as f64 / (1024.0 * 1024.0 * 1024.0));
        }
        if let Some(free_mem) = node.memory_free {
            println!("  Memory: {:.2} GB free", free_mem as f64 / (1024.0 * 1024.0 * 1024.0));
        }
    }

    // Demonstrate worker assignment strategies
    let worker_counts = [1, 2, 4, 8, 16];
    
    for &workers in &worker_counts {
        if workers <= topology.total_cpus {
            println!("\nWorker assignment for {} workers:", workers);
            let mut cpu_manager_clone = CpuAffinityManager::new()?;
            let assignments = cpu_manager_clone.assign_workers(workers)?;
            
            for assignment in assignments {
                println!("  Worker {} → CPU {} (NUMA Node {})",
                    assignment.worker_id,
                    assignment.cpu_id,
                    assignment.numa_node
                );
            }

            let recommendations = cpu_manager.get_performance_recommendations(workers);
            if !recommendations.is_empty() {
                println!("  Recommendations:");
                for rec in recommendations {
                    println!("    • {}", rec);
                }
            }
        }
    }

    // Demonstrate setting CPU affinity (if supported)
    #[cfg(target_os = "linux")]
    {
        println!("\nTesting CPU affinity setting:");
        match cpu_manager.set_thread_affinity(0) {
            Ok(()) => println!("  ✅ Successfully set affinity to CPU 0"),
            Err(e) => println!("  ❌ Failed to set affinity: {}", e),
        }
    }

    Ok(())
}

/// Example 4: Prometheus metrics export
async fn prometheus_metrics_example() -> Result<()> {
    println!("\n📊 Example 4: Prometheus Metrics Export");
    println!("---------------------------------------");

    // Create Prometheus exporter with labels
    let exporter = PrometheusExporter::new("router_flood")
        .with_label("instance", "example")
        .with_label("version", "1.0.0")
        .with_label("environment", "development");

    // Create sample session statistics
    let session_stats = router_flood::stats::collector::SessionStats {
        session_id: "example-session".to_string(),
        timestamp: chrono::Utc::now(),
        packets_sent: 15000,
        packets_failed: 25,
        bytes_sent: 1500000,
        duration_secs: 60.0,
        packets_per_second: 250.0,
        megabits_per_second: 2.0,
        protocol_breakdown: {
            let mut breakdown = std::collections::HashMap::new();
            breakdown.insert("UDP".to_string(), 9000);
            breakdown.insert("TCP".to_string(), 4500);
            breakdown.insert("ICMP".to_string(), 1500);
            breakdown
        },
        system_stats: None,
    };

    // Create sample system statistics
    let system_stats = router_flood::stats::collector::SystemStats {
        cpu_usage: 45.5,
        memory_usage: 1024 * 1024 * 512, // 512 MB
        memory_total: 1024 * 1024 * 1024 * 8, // 8 GB
        network_sent: 1500000,
        network_received: 50000,
    };

    // Export session metrics
    println!("Exporting session metrics...");
    let session_metrics = exporter.export_session_stats(&session_stats)?;
    println!("Session metrics exported ({} bytes)", session_metrics.len());

    // Export system metrics
    println!("Exporting system metrics...");
    let system_metrics = exporter.export_system_stats(&system_stats)?;
    println!("System metrics exported ({} bytes)", system_metrics.len());

    // Export combined metrics
    println!("Exporting combined metrics...");
    let combined_metrics = exporter.export_combined_metrics(&session_stats, Some(&system_stats))?;
    
    // Save to file
    let metrics_file = "/tmp/router_flood_metrics.txt";
    exporter.save_to_file(&session_stats, Some(&system_stats), metrics_file).await?;
    println!("✅ Metrics saved to: {}", metrics_file);

    // Show sample metrics
    println!("\nSample metrics output:");
    let lines: Vec<&str> = combined_metrics.lines().take(10).collect();
    for line in lines {
        if !line.trim().is_empty() {
            println!("  {}", line);
        }
    }
    println!("  ... ({} total lines)", combined_metrics.lines().count());

    Ok(())
}

/// Example 5: Security and audit logging
async fn security_audit_example() -> Result<()> {
    println!("\n🔒 Example 5: Security and Audit Logging");
    println!("----------------------------------------");

    // Security context analysis
    let capability_manager = CapabilityManager::new()?;
    println!("Security analysis:");
    println!("{}", capability_manager.security_report());

    // Tamper-proof audit logging
    println!("Demonstrating tamper-proof audit logging...");
    
    let audit_file = "/tmp/router_flood_audit.log";
    let mut audit_log = TamperProofAuditLog::new(audit_file, "example-session")?;

    // Log various events
    let events = [
        ("SESSION_START", "Session initialized with example configuration"),
        ("CONFIG_LOADED", "Configuration loaded from template: basic"),
        ("SECURITY_CHECK", "Capability validation passed"),
        ("SIMULATION_START", "Packet generation started in dry-run mode"),
        ("STATS_EXPORT", "Statistics exported to Prometheus format"),
        ("SIMULATION_END", "Packet generation completed successfully"),
        ("SESSION_END", "Session terminated normally"),
    ];

    for (event_type, details) in &events {
        audit_log.write_entry(event_type, details).await?;
        println!("  📝 Logged: {} - {}", event_type, details);
    }

    // Verify audit log integrity
    println!("\nVerifying audit log integrity...");
    match audit_log.verify_integrity().await {
        Ok(true) => println!("  ✅ Audit log integrity verified"),
        Ok(false) => println!("  ❌ Audit log integrity check failed"),
        Err(e) => println!("  ❌ Integrity verification error: {}", e),
    }

    println!("✅ Audit log saved to: {}", audit_file);

    Ok(())
}

/// Example 6: Performance benchmarking
async fn performance_benchmark_example() -> Result<()> {
    println!("\n🏁 Example 6: Performance Benchmarking");
    println!("--------------------------------------");

    // Benchmark packet generation performance
    println!("Benchmarking packet generation...");

    let iterations = 10000;
    let mut simd_builder = SimdPacketBuilder::new();
    
    // Benchmark different packet sizes
    let packet_sizes = [64, 256, 512, 1024, 1400];
    
    for &size in &packet_sizes {
        let mut total_time = std::time::Duration::ZERO;
        let mut buffer = vec![0u8; size + 28]; // IP + UDP headers
        
        for _ in 0..iterations {
            let start = Instant::now();
            
            let _packet_size = simd_builder.build_udp_packet_simd(
                &mut buffer,
                [192, 168, 1, 100],
                [192, 168, 1, 1],
                12345,
                80,
                size,
            )?;
            
            total_time += start.elapsed();
        }
        
        let avg_time = total_time / iterations;
        let pps = 1.0 / avg_time.as_secs_f64();
        let mbps = (size as f64 * 8.0 * pps) / 1_000_000.0;
        
        println!("  {} bytes: {:.2}μs avg, {:.0} PPS, {:.2} Mbps",
            size,
            avg_time.as_micros(),
            pps,
            mbps
        );
    }

    // Benchmark buffer pool performance
    println!("\nBenchmarking buffer pool...");
    
    let pool = NumaBufferPool::new();
    pool.warm_up(100)?;
    
    let start = Instant::now();
    let mut buffers = Vec::new();
    
    // Allocate buffers
    for _ in 0..10000 {
        if let Some(buffer) = pool.get_buffer(1024) {
            buffers.push(buffer);
        }
    }
    
    let allocation_time = start.elapsed();
    
    // Return buffers
    let start = Instant::now();
    for buffer in buffers {
        pool.return_buffer(buffer);
    }
    let return_time = start.elapsed();
    
    let stats = pool.get_stats();
    
    println!("  Allocation: {:.2}ms ({:.0} ops/sec)",
        allocation_time.as_millis(),
        10000.0 / allocation_time.as_secs_f64()
    );
    println!("  Return: {:.2}ms ({:.0} ops/sec)",
        return_time.as_millis(),
        10000.0 / return_time.as_secs_f64()
    );
    println!("  Hit rate: {:.1}%", stats.hit_rate);

    // System performance summary
    println!("\nSystem performance summary:");
    let cpu_manager = CpuAffinityManager::new()?;
    let topology = cpu_manager.topology();
    
    println!("  CPU cores: {}", topology.total_cpus);
    println!("  NUMA nodes: {}", topology.numa_nodes.len());
    println!("  Hyperthreading: {}", if topology.hyperthreading_enabled { "Yes" } else { "No" });
    
    let simd_info = simd_builder.get_performance_info();
    println!("  SIMD support: {} ({})", 
        if simd_info.simd_available { "Yes" } else { "No" },
        simd_info.instruction_set
    );

    Ok(())
}

/// Helper function to create a test configuration
#[allow(dead_code)]
fn create_test_config() -> Result<router_flood::config::Config> {
    ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(vec![80, 443])
        .threads(4)
        .packet_rate(1000)
        .duration(Some(60))
        .dry_run(true)
        .build()
}