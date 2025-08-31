# 📚 Router Flood API Documentation

This document provides comprehensive API documentation for Router Flood's Rust library interface.

## 📋 Table of Contents

- [Core Modules](#core-modules)
- [Configuration API](#configuration-api)
- [Packet Building API](#packet-building-api)
- [Performance API](#performance-api)
- [Security API](#security-api)
- [Monitoring API](#monitoring-api)
- [Error Handling](#error-handling)
- [Examples](#examples)

## 🏗️ Core Modules

### Main Library Interface

```rust
use router_flood::*;

// Core functionality
use router_flood::config::{Config, ConfigBuilder, ProtocolMix};
use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::core::simulation::Simulation;
use router_flood::stats::{Stats, LockFreeStats};
// Abstractions module has been removed for simplicity
use router_flood::utils::raii::{WorkerGuard, ResourceGuard};
```

### Module Overview

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `config` | Configuration management | `Config`, `ConfigBuilder`, `ProtocolMix` |
| `packet` | Packet construction | `PacketBuilder`, `PacketType` |
| `core` | Core functionality | `Simulation`, `Target`, `Worker`, `Network` |
| `performance` | Performance optimizations | `BufferPool`, `CpuAffinity`, `SimdOptimizer` |
| `security` | Security and validation | `Validator`, `AuditLogger`, `Capabilities` |
| `stats` | Statistics and monitoring | `Stats`, `LockFreeStats`, `PerCpuStats` |
| `transport` | Network transport | `TransportChannel`, `WorkerChannels` |
| `utils` | Utilities | `BufferPool`, `RAII Guards`, `BatchedRng` |

## ⚙️ Configuration API

### ConfigBuilder

The `ConfigBuilder` provides a fluent API for creating configurations:

```rust
use router_flood::config::ConfigBuilder;

let config = ConfigBuilder::new()
    .target_ip("192.168.1.100")?
    .ports(vec![80, 443, 8080])
    .threads(8)
    .packet_rate(1000)
    .packet_size_range(64, 1400)
    .duration(300)
    .dry_run(false)
    .perfect_simulation(false)
    .build()?;
```

#### Methods

| Method | Parameters | Description |
|--------|------------|-------------|
| `new()` | - | Create a new builder |
| `target_ip(ip)` | `&str` | Set target IP address |
| `ports(ports)` | `Vec<u16>` | Set target ports |
| `threads(n)` | `usize` | Set number of worker threads |
| `packet_rate(rate)` | `u64` | Set packets per second per thread |
| `packet_size_range(min, max)` | `usize, usize` | Set packet size range |
| `duration(secs)` | `u64` | Set test duration |
| `dry_run(enabled)` | `bool` | Enable/disable dry-run mode |
| `perfect_simulation(enabled)` | `bool` | Enable/disable perfect simulation (100% success rate) |
| `build()` | - | Build the configuration |

### ProtocolMix

Configure protocol distribution:

```rust
use router_flood::config::ProtocolMix;

let protocol_mix = ProtocolMix {
    udp_ratio: 0.6,
    tcp_syn_ratio: 0.25,
    tcp_ack_ratio: 0.1,
    icmp_ratio: 0.05,
    ipv6_ratio: 0.0,
    arp_ratio: 0.0,
};
```

### Configuration Validation

```rust
use router_flood::config::validate_config;

// Validate configuration
let result = validate_config(&config);
match result {
    Ok(()) => println!("Configuration is valid"),
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

## 📦 Packet Building API

### PacketBuilder

High-performance packet construction:

```rust
use router_flood::packet::{PacketBuilder, PacketType, Target};
use std::net::IpAddr;

let mut packet_builder = PacketBuilder::new(
    (64, 1400),  // packet size range
    protocol_mix
);

// Build packet into buffer (zero-copy)
let mut buffer = vec![0u8; 1500];
let target_ip: IpAddr = "192.168.1.100".parse()?;
let target_port = 80;

let result = packet_builder.build_packet_into_buffer(
    &mut buffer,
    PacketType::Udp,
    target_ip,
    target_port
)?;

let (packet_size, protocol_name) = result;
```

### PacketType Enum

```rust
use router_flood::packet::PacketType;

// Available packet types
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
```

### Target Configuration

```rust
use router_flood::packet::Target;
use std::net::IpAddr;

let target = Target::new(
    "192.168.1.100".parse::<IpAddr>()?,
    80
);
```

## ⚡ Performance API

### Buffer Pool Management

```rust
use router_flood::performance::{LockFreeBufferPool, SharedBufferPool};

// Create a lock-free buffer pool
let pool = LockFreeBufferPool::new(1500, 1000); // buffer_size, pool_size

// Get a buffer
if let Some(mut buffer) = pool.buffer() {
    // Use buffer for packet construction
    // Buffer is automatically returned when dropped
}

// Get pool statistics
let stats = pool.get_stats();
println!("Hit rate: {:.2}%", stats.hit_rate);
```

### CPU Affinity Management

```rust
use router_flood::performance::CpuAffinity;

// Analyze system topology
let topology = CpuAffinity::analyze_system_topology()?;
println!("NUMA nodes: {}", topology.numa_nodes.len());

// Set CPU affinity for current thread
CpuAffinity::set_thread_affinity(0)?; // Pin to CPU 0

// Get optimal CPU assignments for workers
let assignments = CpuAffinity::get_optimal_assignments(8)?; // 8 workers
for (worker_id, cpu_id) in assignments.iter().enumerate() {
    println!("Worker {} -> CPU {}", worker_id, cpu_id);
}
```

### SIMD Optimizations

```rust
use router_flood::performance::SimdOptimizer;

// Check SIMD capabilities
let capabilities = SimdOptimizer::detect_capabilities();
println!("AVX2 support: {}", capabilities.avx2);
println!("SSE4.2 support: {}", capabilities.sse42);
println!("NEON support: {}", capabilities.neon);

// Use SIMD-optimized packet generation
let mut optimizer = SimdOptimizer::new();
let packets = optimizer.generate_packets_simd(&config, 1000)?;
```

## 🛡️ Security API

### Capability Management

```rust
use router_flood::security::Capabilities;

// Check required capabilities
let caps = Capabilities::check_current()?;
println!("CAP_NET_RAW: {}", caps.has_net_raw);
println!("Running as root: {}", caps.is_root);

// Validate security context
let security_report = Capabilities::generate_security_report()?;
println!("{}", security_report);
```

### Audit Logging

```rust
use router_flood::security::AuditLogger;

// Create audit logger
let mut audit_logger = AuditLogger::new("router_flood_audit.log")?;

// Log events
audit_logger.log_session_start(&config)?;
audit_logger.log_packet_generation(1000, "UDP")?;
audit_logger.log_session_end()?;

// Verify audit log integrity
let is_valid = audit_logger.verify_integrity()?;
println!("Audit log integrity: {}", if is_valid { "✅ Valid" } else { "❌ Compromised" });
```

### Input Validation

```rust
use router_flood::validation::{validate_ip_address, validate_port_range};

// Validate IP address
match validate_ip_address("192.168.1.100") {
    Ok(ip) => println!("Valid IP: {}", ip),
    Err(e) => eprintln!("Invalid IP: {}", e),
}

// Validate port range
let ports = vec![80, 443, 8080];
validate_port_range(&ports)?;
```

## 📊 Monitoring API

### Statistics Collection

```rust
use router_flood::stats::{Stats, LockFreeStats, PerCpuStats};
use router_flood::stats::lockfree::ProtocolId;
use std::sync::Arc;

// Traditional mutex-based statistics
let stats = Arc::new(Stats::default());
stats.increment_sent(64, "UDP");
stats.increment_failed();

// Lock-free statistics (2x faster)
let lockfree_stats = Arc::new(LockFreeStats::new());
lockfree_stats.increment_sent(64, ProtocolId::Udp);
lockfree_stats.increment_failed();

// Per-CPU statistics for maximum performance
let per_cpu_stats = Arc::new(PerCpuStats::new());
let local = per_cpu_stats.get_local();
local.increment_sent(64, ProtocolId::Udp);

// Aggregate per-CPU stats
let snapshot = per_cpu_stats.aggregate();
println!("Packets sent: {}", snapshot.packets_sent);
println!("Success rate: {:.2}%", snapshot.success_rate());
```

### Prometheus Integration

```rust
use router_flood::monitoring::PrometheusExporter;

// Create Prometheus exporter
let exporter = PrometheusExporter::new("0.0.0.0:9090")?;

// Export metrics
exporter.export_metrics(&stats)?;

// Start HTTP server for metrics
exporter.start_server().await?;
```

### Real-Time Monitoring

```rust
use router_flood::monitoring::RealTimeMonitor;

// Create real-time monitor
let monitor = RealTimeMonitor::new();

// Start monitoring
monitor.start_monitoring(stats_collector, Duration::from_secs(1)).await?;
```

## 🛡️ RAII Resource Management

### Resource Guards

```rust
use router_flood::utils::raii::*;
use router_flood::core::worker_manager::Workers;
use router_flood::transport::WorkerChannels;

// Worker management with automatic cleanup
let workers = Workers::new(config);
let _guard = WorkerGuard::new(manager, "main_worker");
// Worker automatically stops when guard is dropped

// Channel management
let channels = WorkerChannels {
    ipv4_sender: None,
    ipv6_sender: None,
    l2_sender: None,
};
let mut channel_guard = ChannelGuard::new(channels, "worker_1");
// Channels automatically closed when guard is dropped

// Composite resource management
let resource_guard = ResourceGuard::new()
    .with_workers(manager)
    .with_signal_handler()
    .with_stats(stats);
// All resources cleaned up in correct order when dropped
```

### Signal Handler Guard

```rust
use router_flood::utils::raii::SignalGuard;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Automatic signal handler registration/deregistration
let running = Arc::new(AtomicBool::new(true));
let _signal_guard = SignalGuard::new(running.clone());
// Signal handler automatically removed when guard is dropped
```

## ❌ Error Handling

### Error Types

```rust
use router_flood::error::{RouterFloodError, Result};

// Main error type
pub enum RouterFloodError {
    Config(ConfigError),
    Network(NetworkError),
    Validation(ValidationError),
    Packet(PacketError),
    Stats(StatsError),
    System(SystemError),
    Audit(AuditError),
    Io(std::io::Error),
}

// Result type alias
pub type Result<T> = std::result::Result<T, RouterFloodError>;
```

### Error Handling Patterns

```rust
use router_flood::error::{Result, RouterFloodError};

// Pattern matching on errors
match some_operation() {
    Ok(result) => println!("Success: {:?}", result),
    Err(RouterFloodError::Config(e)) => eprintln!("Configuration error: {}", e),
    Err(RouterFloodError::Network(e)) => eprintln!("Network error: {}", e),
    Err(RouterFloodError::Validation(e)) => eprintln!("Validation error: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}

// Using the ? operator
fn example_function() -> Result<()> {
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .build()?;
    
    let mut packet_builder = PacketBuilder::new((64, 1400), config.protocol_mix);
    // ... more operations
    
    Ok(())
}
```

## 📝 Examples

### Basic Usage Example

```rust
use router_flood::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Create configuration
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .ports(vec![80, 443])
        .threads(4)
        .packet_rate(1000)
        .duration(60)
        .build()?;

    // Create and run simulation
    let mut simulation = Simulation::new(config)?;
    simulation.run().await?;

    // Get final statistics
    let stats = simulation.get_statistics();
    println!("Total packets sent: {}", stats.packets_sent);
    println!("Success rate: {:.2}%", stats.success_rate);

    Ok(())
}
```

### Advanced Performance Example

```rust
use router_flood::*;
use router_flood::performance::{CpuAffinity, LockFreeBufferPool};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up CPU affinity
    let assignments = CpuAffinity::get_optimal_assignments(8)?;
    
    // Create high-performance buffer pool
    let buffer_pool = LockFreeBufferPool::new(1500, 10000);
    
    // Create configuration with performance optimizations
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .threads(8)
        .packet_rate(10000)
        .enable_simd(true)
        .buffer_pool(buffer_pool)
        .cpu_assignments(assignments)
        .build()?;

    // Run simulation with monitoring
    let mut simulation = Simulation::new(config)?;
    
    // Start Prometheus metrics
    let prometheus = PrometheusExporter::new("0.0.0.0:9090")?;
    prometheus.start_server().await?;
    
    // Run simulation
    simulation.run().await?;

    Ok(())
}
```

### Security-Focused Example

```rust
use router_flood::*;
use router_flood::security::{Capabilities, AuditLogger};

#[tokio::main]
async fn main() -> Result<()> {
    // Check security context
    let caps = Capabilities::check_current()?;
    if !caps.has_net_raw {
        eprintln!("CAP_NET_RAW capability required");
        return Ok(());
    }

    // Set up audit logging
    let mut audit_logger = AuditLogger::new("security_audit.log")?;
    
    // Create secure configuration
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?  // Private IP only
        .threads(4)  // Conservative thread count
        .packet_rate(500)  // Conservative rate
        .enable_audit_logging(true)
        .audit_logger(audit_logger)
        .build()?;

    // Validate configuration
    validate_config(&config)?;
    
    // Run with security monitoring
    let mut simulation = Simulation::new(config)?;
    simulation.run().await?;

    // Verify audit log integrity
    let is_valid = audit_logger.verify_integrity()?;
    println!("Audit integrity: {}", if is_valid { "✅" } else { "❌" });

    Ok(())
}
```

### Testing and Validation Example

```rust
use router_flood::*;

#[tokio::test]
async fn test_packet_generation() -> Result<()> {
    // Create test configuration with perfect simulation
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .dry_run(true)  // Safe testing mode
        .perfect_simulation(true)  // 100% success rate for clean testing
        .threads(1)
        .packet_rate(100)
        .duration(5)
        .build()?;

    // Create packet builder
    let mut packet_builder = PacketBuilder::new(
        (64, 1400),
        config.protocol_mix
    );

    // Test packet generation
    let mut buffer = vec![0u8; 1500];
    let result = packet_builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        "192.168.1.100".parse()?,
        80
    )?;

    let (size, protocol) = result;
    assert!(size > 0);
    assert_eq!(protocol, "UDP");

    Ok(())
}
```

## 🧪 Dry-Run and Simulation Modes

### Realistic Simulation (Default)

```rust
use router_flood::*;

// Create configuration with realistic simulation (98% success rate)
let config = ConfigBuilder::new()
    .target_ip("192.168.1.100")?
    .dry_run(true)
    .perfect_simulation(false)  // Default: realistic simulation
    .build()?;

let mut simulation = Simulation::new(config)?;
let result = simulation.run().await?;

// Expect some simulated failures for realistic training
assert!(result.success_rate >= 95.0 && result.success_rate <= 100.0);
```

### Perfect Simulation

```rust
use router_flood::*;

// Create configuration with perfect simulation (100% success rate)
let config = ConfigBuilder::new()
    .target_ip("192.168.1.100")?
    .dry_run(true)
    .perfect_simulation(true)  // Perfect simulation mode
    .build()?;

let mut simulation = Simulation::new(config)?;
let result = simulation.run().await?;

// Expect perfect success rate for clean validation
assert_eq!(result.success_rate, 100.0);
assert_eq!(result.packets_failed, 0);
```

### Use Case Examples

```rust
// Configuration validation in CI/CD
let ci_config = ConfigBuilder::new()
    .target_ip("192.168.1.100")?
    .dry_run(true)
    .perfect_simulation(true)  // Clean validation without noise
    .duration(10)
    .build()?;

// Educational training with realistic failures
let training_config = ConfigBuilder::new()
    .target_ip("192.168.1.100")?
    .dry_run(true)
    .perfect_simulation(false)  // Realistic simulation for learning
    .duration(60)
    .build()?;

// Production testing (actual packets)
let production_config = ConfigBuilder::new()
    .target_ip("192.168.1.100")?
    .dry_run(false)  // Real packets
    // perfect_simulation is ignored when dry_run is false
    .build()?;
```

## 🔗 Integration Examples

### With Tokio

```rust
use router_flood::*;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .build()?;

    let mut simulation = Simulation::new(config)?;
    
    // Run simulation with periodic status updates
    let mut interval = interval(Duration::from_secs(5));
    
    tokio::select! {
        result = simulation.run() => {
            result?;
            println!("Simulation completed");
        }
        _ = async {
            loop {
                interval.tick().await;
                let stats = simulation.get_current_stats();
                println!("Current rate: {} PPS", stats.current_rate);
            }
        } => {}
    }

    Ok(())
}
```

### With Serde for Configuration

```rust
use router_flood::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestConfig {
    target_ip: String,
    ports: Vec<u16>,
    threads: usize,
    packet_rate: u64,
}

fn load_config_from_json(json_str: &str) -> Result<Config> {
    let test_config: TestConfig = serde_json::from_str(json_str)?;
    
    ConfigBuilder::new()
        .target_ip(&test_config.target_ip)?
        .ports(test_config.ports)
        .threads(test_config.threads)
        .packet_rate(test_config.packet_rate)
        .build()
}
```

## 📚 Additional Resources

- [Rust Documentation](https://docs.rs/router-flood)
- [Examples Directory](./examples/)
- [Integration Tests](./tests/integration_tests.rs)
- [Performance Benchmarks](./benches/)
- [Security Guidelines](./SECURITY.md)

---

**Note**: This API documentation covers the main public interfaces. For complete details, refer to the generated Rust documentation with `cargo doc --open`.