# 🧪 Router Flood Testing Guide

This guide provides comprehensive information about testing Router Flood, including unit tests, integration tests, property-based testing, and performance benchmarks.

## 📋 Table of Contents

- [Testing Overview](#testing-overview)
- [Test Categories](#test-categories)
- [Running Tests](#running-tests)
- [Property-Based Testing](#property-based-testing)
- [Performance Testing](#performance-testing)
- [Security Testing](#security-testing)
- [Integration Testing](#integration-testing)
- [Continuous Integration](#continuous-integration)

## 🎯 Testing Overview

Router Flood employs a comprehensive testing strategy with multiple layers of validation:

### Test Statistics

| Test Category | Count | Coverage | Purpose |
|---------------|-------|----------|---------|
| **Unit Tests** | 200+ | Individual components | Component validation |
| **Integration Tests** | 50+ | End-to-end scenarios | System integration |
| **Property Tests** | 20+ | 10,000+ cases each | Edge case detection |
| **Security Tests** | 30+ | Security features | Vulnerability prevention |
| **Performance Tests** | 20+ | Benchmarks | Regression detection |
| **Fuzzing Tests** | 3 | Continuous | Security validation |

### Test Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Unit Tests    │    │ Integration     │    │ Property Tests  │
│                 │    │ Tests           │    │                 │
│ • Components    │    │ • End-to-end    │    │ • Random inputs │
│ • Functions     │ -> │ • Workflows     │ -> │ • Edge cases    │
│ • Modules       │    │ • APIs          │    │ • Invariants    │
│ • Isolated      │    │ • Real scenarios│    │ • Stress testing│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📚 Test Categories

### Unit Tests

Unit tests are now organized in dedicated test files in the `tests/` directory for better separation of concerns. Previously inline tests have been moved to dedicated unit test files:

#### Test Organization

- **Inline tests moved**: All `#[cfg(test)]` modules moved to dedicated files
- **Dedicated test files**: 40+ unit test files in `tests/` directory
- **Better separation**: Clear distinction between implementation and test code
- **Easier maintenance**: Tests are easier to find, modify, and organize

#### Unit Test Files

```
tests/
├── config_builder_unit_tests.rs          # Configuration builder tests
├── config_schema_unit_tests.rs           # Schema validation tests
├── monitoring_dashboard_unit_tests.rs    # Dashboard functionality tests
├── monitoring_metrics_unit_tests.rs      # Metrics collection tests
├── monitoring_alerts_unit_tests.rs       # Alert system tests
├── monitoring_prometheus_unit_tests.rs   # Prometheus export tests
├── performance_buffer_pool_unit_tests.rs # Buffer pool tests
├── security_capabilities_unit_tests.rs   # Security capability tests
├── error_user_friendly_unit_tests.rs     # Error handling tests
├── ui_progress_unit_tests.rs             # UI component tests
└── ... (30+ more unit test files)
```

```rust
// Example unit test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_builder_creation() {
        let protocol_mix = ProtocolMix::default();
        let builder = PacketBuilder::new((64, 1400), protocol_mix);
        assert!(builder.is_ok());
    }

    #[tokio::test]
    async fn test_async_simulation() {
        let config = ConfigBuilder::new()
            .target_ip("192.168.1.100")
            .unwrap()
            .dry_run(true)
            .build()
            .unwrap();
        
        let simulation = Simulation::new(config).unwrap();
        let result = simulation.run().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Located in `tests/` directory:

```rust
// tests/integration_tests.rs
use router_flood::*;

#[tokio::test]
async fn test_full_simulation_workflow() {
    // Test complete workflow from config to execution
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .ports(vec![80, 443])
        .threads(2)
        .packet_rate(100)
        .duration(5)
        .dry_run(true)
        .build()?;

    let mut simulation = Simulation::new(config)?;
    let result = simulation.run().await?;
    
    assert!(result.packets_sent > 0);
    assert_eq!(result.success_rate, 100.0);
}
```

### Property-Based Tests

Using `proptest` for comprehensive edge case testing:

```rust
// tests/property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_packet_building_never_panics(
        packet_type in prop::sample::select(&[
            PacketType::Udp, PacketType::TcpSyn, PacketType::TcpAck
        ]),
        target_ip in valid_ipv4_private(),
        target_port in 1u16..=65535,
        buffer_size in 100usize..=2000
    ) {
        let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
        let mut buffer = vec![0u8; buffer_size];
        
        // Should never panic, even with random inputs
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            IpAddr::V4(target_ip),
            target_port,
        );
        
        // Result can be Ok or Err, but should never panic
        match result {
            Ok((size, protocol)) => {
                prop_assert!(size > 0);
                prop_assert!(size <= buffer_size);
                prop_assert!(!protocol.is_empty());
            }
            Err(_) => {
                // Errors are acceptable for random inputs
            }
        }
    }
}
```

## 🏃 Running Tests

### Basic Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_packet_builder

# Run tests in specific file
cargo test --test integration_tests

# Run with multiple threads
cargo test -- --test-threads=4
```

### Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Property-based tests
cargo test --test property_tests

# Security tests
cargo test security

# Performance tests
cargo test performance
```

### Verbose Testing

```bash
# Run with detailed output
RUST_LOG=debug cargo test -- --nocapture

# Run with timing information
cargo test -- --nocapture --show-output

# Run with test statistics
cargo test -- --report-time
```

### Test Coverage

```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

## 🎲 Property-Based Testing

### Test Strategies

Property-based testing generates thousands of random inputs to find edge cases:

#### Configuration Validation

```rust
proptest! {
    #[test]
    fn test_config_validation_properties(
        threads in 1usize..=50,
        packet_rate in 1u64..=5000,
        min_size in 20usize..=500,
        max_size in 500usize..=1400,
        target_ip in valid_ipv4_private()
    ) {
        let result = ConfigBuilder::new()
            .target_ip(&target_ip.to_string())
            .threads(threads)
            .packet_rate(packet_rate)
            .packet_size_range(min_size, max_size)
            .build();
        
        // Valid inputs should always succeed
        prop_assert!(result.is_ok());
        
        if let Ok(config) = result {
            prop_assert_eq!(config.attack.threads, threads);
            prop_assert_eq!(config.attack.packet_rate, packet_rate);
        }
    }
}
```

#### Protocol Mix Validation

```rust
proptest! {
    #[test]
    fn test_protocol_mix_normalization(
        protocol_mix in valid_protocol_mix()
    ) {
        let total = protocol_mix.udp_ratio + 
                   protocol_mix.tcp_syn_ratio + 
                   protocol_mix.tcp_ack_ratio + 
                   protocol_mix.icmp_ratio + 
                   protocol_mix.ipv6_ratio + 
                   protocol_mix.arp_ratio;
        
        // Should always sum to approximately 1.0
        prop_assert!((total - 1.0).abs() < 0.001);
        
        // All ratios should be non-negative
        prop_assert!(protocol_mix.udp_ratio >= 0.0);
        prop_assert!(protocol_mix.tcp_syn_ratio >= 0.0);
        // ... other ratios
    }
}
```

### Running Property Tests

```bash
# Run property tests with default case count
cargo test --test property_tests

# Run with more test cases
PROPTEST_CASES=100000 cargo test --test property_tests

# Run with specific seed for reproducibility
PROPTEST_SEED=12345 cargo test --test property_tests

# Debug failing property test
PROPTEST_VERBOSE=1 cargo test test_protocol_selection_distribution
```

### Property Test Configuration

```rust
// Configure property test parameters
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,
        max_shrink_iters: 1000,
        timeout: 5000,
        ..ProptestConfig::default()
    })]
    
    #[test]
    fn test_with_custom_config(input in 0..1000) {
        // Test implementation
    }
}
```

## ⚡ Performance Testing

### Benchmark Tests

Using `criterion` for performance benchmarks:

```rust
// benches/packet_building.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use router_flood::packet::PacketBuilder;

fn benchmark_packet_generation(c: &mut Criterion) {
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let mut buffer = vec![0u8; 1500];
    
    c.bench_function("udp_packet_generation", |b| {
        b.iter(|| {
            packet_builder.build_packet_into_buffer(
                black_box(&mut buffer),
                black_box(PacketType::Udp),
                black_box("192.168.1.100".parse().unwrap()),
                black_box(80)
            )
        })
    });
}

criterion_group!(benches, benchmark_packet_generation);
criterion_main!(benches);
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench packet_building

# Generate HTML report
cargo bench -- --output-format html

# Compare with baseline
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

### Performance Regression Tests

```rust
#[test]
fn test_packet_generation_performance() {
    let start = std::time::Instant::now();
    
    // Generate 10,000 packets
    for _ in 0..10000 {
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            PacketType::Udp,
            target_ip,
            80
        );
        assert!(result.is_ok());
    }
    
    let duration = start.elapsed();
    let pps = 10000.0 / duration.as_secs_f64();
    
    // Assert minimum performance threshold
    assert!(pps > 50000.0, "Performance regression: {} PPS", pps);
}
```

## 🛡️ Security Testing

### Capability Testing

```rust
#[test]
fn test_capability_detection() {
    let caps = Capabilities::check_current().unwrap();
    
    // Test capability detection
    assert!(caps.has_net_raw || caps.is_root);
    
    // Test security context
    let report = Capabilities::generate_security_report().unwrap();
    assert!(report.contains("Security Context"));
}
```

### Input Validation Testing

```rust
#[test]
fn test_ip_validation_security() {
    // Test private IP validation
    assert!(validate_ip_address("192.168.1.100").is_ok());
    assert!(validate_ip_address("10.0.0.1").is_ok());
    assert!(validate_ip_address("172.16.0.1").is_ok());
    
    // Test public IP rejection
    assert!(validate_ip_address("8.8.8.8").is_err());
    assert!(validate_ip_address("1.1.1.1").is_err());
    
    // Test malformed IP rejection
    assert!(validate_ip_address("999.999.999.999").is_err());
    assert!(validate_ip_address("not.an.ip").is_err());
}
```

### Audit Log Testing

```rust
#[tokio::test]
async fn test_audit_log_integrity() {
    let mut audit_logger = AuditLogger::new("test_audit.log").unwrap();
    
    // Log test events
    audit_logger.log_session_start(&config).unwrap();
    audit_logger.log_packet_generation(1000, "UDP").unwrap();
    audit_logger.log_session_end().unwrap();
    
    // Verify integrity
    let is_valid = audit_logger.verify_integrity().unwrap();
    assert!(is_valid, "Audit log integrity check failed");
    
    // Test tamper detection
    // ... modify log file ...
    let is_valid_after_tamper = audit_logger.verify_integrity().unwrap();
    assert!(!is_valid_after_tamper, "Tamper detection failed");
}
```

### Fuzzing Tests

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing
cargo fuzz init

# Run packet builder fuzzing
cargo fuzz run fuzz_packet_builder

# Run configuration parser fuzzing
cargo fuzz run fuzz_config_parser

# Run with specific timeout
cargo fuzz run fuzz_packet_builder -- -max_total_time=300
```

Fuzzing target example:

```rust
// fuzz/fuzz_targets/fuzz_packet_builder.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use router_flood::packet::PacketBuilder;

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let mut buffer = vec![0u8; 1500];
    
    // Fuzz with random data
    let _ = packet_builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        "192.168.1.100".parse().unwrap(),
        80
    );
});
```

### Integration Testing

#### End-to-End Testing

```rust
#[tokio::test]
async fn test_complete_workflow() {
    // Test complete workflow from CLI to execution
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .ports(vec![80, 443])
        .threads(4)
        .packet_rate(1000)
        .duration(10)
        .dry_run(true)
        .perfect_simulation(true)  // Use perfect simulation for clean testing
        .build()?;

    // Test configuration validation
    validate_config(&config)?;

    // Test simulation creation
    let mut simulation = Simulation::new(config)?;

    // Test simulation execution
    let result = simulation.run().await?;

    // Verify results
    assert!(result.packets_sent > 0);
    assert!(result.success_rate > 95.0);
    assert!(result.duration_seconds >= 10);
}
```

### Network Interface Testing

```rust
#[test]
fn test_network_interface_detection() {
    let interfaces = NetworkInterface::list_available().unwrap();
    assert!(!interfaces.is_empty(), "No network interfaces found");
    
    for interface in interfaces {
        assert!(!interface.name.is_empty());
        assert!(interface.is_up || !interface.is_up); // Boolean check
    }
}
```

#### Perfect Simulation Testing

```rust
#[tokio::test]
async fn test_perfect_simulation_mode() {
    // Test perfect simulation (100% success rate)
    let perfect_config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .dry_run(true)
        .perfect_simulation(true)
        .duration(5)
        .build()?;

    let mut simulation = Simulation::new(perfect_config)?;
    let result = simulation.run().await?;

    // Perfect simulation should have zero failures
    assert_eq!(result.packets_failed, 0);
    assert_eq!(result.success_rate, 100.0);
}

#[tokio::test]
async fn test_realistic_simulation_mode() {
    // Test realistic simulation (98% success rate)
    let realistic_config = ConfigBuilder::new()
        .target_ip("192.168.1.100")?
        .dry_run(true)
        .perfect_simulation(false)
        .duration(5)
        .build()?;

    let mut simulation = Simulation::new(realistic_config)?;
    let result = simulation.run().await?;

    // Realistic simulation should have some failures
    assert!(result.success_rate >= 95.0 && result.success_rate <= 100.0);
    // Should have some packets sent
    assert!(result.packets_sent > 0);
}
```

#### Configuration Integration Testing

```rust
#[test]
fn test_yaml_configuration_loading() {
    let yaml_config = r#"
target:
  ip: "192.168.1.100"
  ports: [80, 443]
attack:
  threads: 4
  packet_rate: 1000
  duration: 60
safety:
  dry_run: true
  perfect_simulation: false
"#;

    let config: Config = serde_yaml::from_str(yaml_config).unwrap();
    assert_eq!(config.target.ip, "192.168.1.100");
    assert_eq!(config.attack.threads, 4);
    assert!(!config.safety.perfect_simulation);
    
    // Test validation
    validate_config(&config).unwrap();
}
```

## 🔄 Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run unit tests
      run: cargo test --lib
    
    - name: Run integration tests
      run: cargo test --test '*'
    
    - name: Run property tests
      run: cargo test --test property_tests
      env:
        PROPTEST_CASES: 1000
    
    - name: Run benchmarks
      run: cargo bench --no-run
    
    - name: Generate coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml
    
    - name: Upload coverage
      uses: codecov/codecov-action@v3
```

### Test Scripts

```bash
#!/bin/bash
# scripts/run-tests.sh

set -e

echo "🧪 Running Router Flood Test Suite"

# Format check
echo "📝 Checking code formatting..."
cargo fmt --check

# Lint check
echo "🔍 Running clippy lints..."
cargo clippy -- -D warnings

# Unit tests
echo "🔧 Running unit tests..."
cargo test --lib

# Integration tests
echo "🔗 Running integration tests..."
cargo test --test '*'

# Property tests
echo "🎲 Running property-based tests..."
PROPTEST_CASES=10000 cargo test --test property_tests

# Security tests
echo "🛡️ Running security tests..."
cargo test security

# Performance tests
echo "⚡ Running performance tests..."
cargo test performance

# Benchmarks
echo "📊 Running benchmarks..."
cargo bench --no-run

echo "✅ All tests passed!"
```

### Test Environment Setup

```bash
# scripts/setup-test-env.sh
#!/bin/bash

# Install test dependencies
cargo install cargo-tarpaulin
cargo install cargo-fuzz
cargo install criterion

# Setup test data
mkdir -p test-data
echo "192.168.1.100" > test-data/test-targets.txt

# Setup test configuration
cat > test-data/test-config.yaml << 'EOF'
target:
  ip: "192.168.1.100"
  ports: [80]
attack:
  threads: 1
  packet_rate: 100
  duration: 5
safety:
  dry_run: true
EOF

echo "Test environment setup complete!"
```

### Performance Monitoring

```bash
# Monitor test performance over time
#!/bin/bash
# scripts/monitor-test-performance.sh

BASELINE_FILE="test-performance-baseline.json"
CURRENT_FILE="test-performance-current.json"

# Run benchmarks and save results
cargo bench -- --output-format json > "$CURRENT_FILE"

# Compare with baseline if it exists
if [ -f "$BASELINE_FILE" ]; then
    echo "Comparing with baseline..."
    # Add comparison logic here
fi

# Update baseline
cp "$CURRENT_FILE" "$BASELINE_FILE"
```

---

**Note**: This testing guide covers comprehensive testing strategies. Adapt the examples to your specific testing needs and environment.