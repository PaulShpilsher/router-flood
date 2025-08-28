# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Router Flood is a Rust-based educational network stress testing tool designed for authorized network testing. It's a safety-first tool with enterprise-grade features for network performance testing and monitoring.

## Build and Development Commands

### Building
```bash
# Standard build
cargo build

# Release build (optimized)
cargo build --release

# After building, set capabilities (required for raw socket access)
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run specific test categories
cargo test security
cargo test performance
cargo test integration

# Run property-based tests
cargo test --test property_tests

# Run benchmarks
cargo bench
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Clean build artifacts
cargo clean
```

### Fuzzing (if needed)
```bash
# Install cargo-fuzz first
cargo install cargo-fuzz

# Run fuzz tests
cargo fuzz run fuzz_packet_builder
cargo fuzz run fuzz_config_parser
cargo fuzz run fuzz_cli_parser
```

## Architecture

The codebase follows a modular architecture with clear separation of concerns:

### Core Components

- **CLI System** (`src/cli/`): Enhanced CLI with interactive mode and subcommand structure. The `enhanced.rs` provides advanced functionality while `basic.rs` handles core argument parsing.

- **Configuration** (`src/config/`): YAML-based configuration with validation, templates, and builder pattern. Configuration can be loaded from files or generated programmatically.

- **Packet Building** (`src/packet/`): Multi-protocol packet construction with strategy pattern. Supports TCP, UDP, ICMP for both IPv4 and IPv6, plus ARP packets.

- **Performance Optimizations** (`src/performance/`): 
  - SIMD acceleration for packet generation (AVX2, SSE4.2, NEON)
  - CPU affinity management for NUMA-aware worker placement
  - Advanced buffer pooling to minimize allocations
  - Zero-copy packet construction

- **Security** (`src/security/`): Capability-based security using Linux capabilities instead of requiring root. Includes tamper-proof audit logging with cryptographic hash chains.

- **Monitoring** (`src/monitoring/`): Prometheus metrics integration, real-time statistics, system resource tracking, and export functionality.

- **Transport Layer** (`src/transport/`): Abstracted network transport with raw socket implementation and mock transport for testing.

- **Validation** (`src/validation.rs`): Multi-layer input validation ensuring only private IP ranges (RFC 1918) are targeted.

### Key Design Patterns

1. **Strategy Pattern**: Used in packet building for different protocol types
2. **Builder Pattern**: Configuration construction and validation
3. **Adapter Pattern**: Stats collection and channel adapters for flexible monitoring
4. **Mock Pattern**: Transport layer abstraction for testability

### Safety Features

- Hard-coded private IP validation (only RFC 1918 ranges allowed)
- Dry-run mode with optional perfect simulation (100% success rate)
- Rate limiting and safety checks
- Capability-based security (CAP_NET_RAW)
- Comprehensive audit logging

## Important Notes

1. **Security First**: This tool is for educational and authorized testing only. It validates that targets are in private IP ranges and includes multiple safety mechanisms.

2. **Performance**: The codebase uses advanced optimizations including SIMD instructions and zero-copy operations. When modifying performance-critical code, consider the impact on these optimizations.

3. **Testing**: Extensive test coverage including unit tests, integration tests, property-based tests, and fuzzing. Always run tests before committing changes.

4. **Error Handling**: User-friendly error messages are provided through the error module. Follow the existing pattern for error handling.

5. **Monitoring**: The tool integrates with Prometheus for production monitoring. Metrics are exposed on a configurable port.

## Common Development Tasks

### Adding a New Protocol

1. Create a new strategy in `src/packet/strategies/`
2. Implement the packet building logic
3. Update the packet builder to include the new strategy
4. Add corresponding tests

### Modifying Configuration

1. Update schema in `src/config/schema.rs`
2. Update validation in `src/config/validation.rs`
3. Update builder if needed in `src/config/builder.rs`
4. Update CLI arguments if needed

### Performance Optimization

1. Profile first using the built-in benchmarks
2. Consider SIMD optimizations in `src/performance/simd_packet.rs`
3. Check buffer pool usage in `src/performance/advanced_buffer_pool.rs`
4. Verify CPU affinity settings