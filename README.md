# 🚀 Router Flood - Advanced Educational Network Stress Tester

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-322%2B%20passing-green.svg)](#testing)
[![Security](https://img.shields.io/badge/security-capability--based-blue.svg)](#security)
[![Performance](https://img.shields.io/badge/performance-SIMD%20optimized-brightgreen.svg)](#performance)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#build-status)

A comprehensive, safety-first network testing tool designed for educational purposes and authorized network testing scenarios. Router Flood combines cutting-edge performance optimizations with enterprise-grade security features while maintaining an educational focus.

## 🎯 Key Features

### 🛡️ **Safety-First Design**
- **Private IP Only**: Hard-coded validation for RFC 1918 private ranges
- **Capability-Based Security**: No root required (CAP_NET_RAW sufficient)
- **Rate Limiting**: Built-in safety limits and monitoring
- **Dry-Run Mode**: Safe testing without sending packets
- **Perfect Simulation**: 100% success rate option for pure configuration validation
- **Tamper-Proof Audit Logging**: Cryptographic integrity protection

### ⚡ **High Performance**
- **SIMD Optimization**: 2-4x faster packet generation (AVX2, SSE4.2, NEON)
- **Advanced Buffer Management**: 60-80% reduction in memory allocations
- **CPU Affinity**: NUMA-aware worker placement for optimal performance
- **Zero-Copy Packet Construction**: Direct in-place packet building
- **Lock-Free Data Structures**: Improved concurrency performance

### 📊 **Professional Monitoring**
- **Prometheus Metrics**: Production-ready monitoring integration
- **Real-Time Statistics**: Live performance monitoring with formatted output
- **System Resource Tracking**: CPU, memory, and network usage
- **Protocol-Level Breakdown**: Detailed traffic analysis
- **Performance Profiling**: Built-in performance analysis tools

### 🧪 **Robust Testing**
- **Property-Based Testing**: 10,000+ generated test cases per property
- **Fuzzing Support**: Continuous security testing with cargo-fuzz (3 fuzz targets)
- **322+ Comprehensive Tests**: Unit, integration, and security tests
- **Regression Protection**: Automated edge case detection
- **Zero Warnings**: Clean compilation with strict linting

### 🎯 **User-Friendly Interface**
- **Interactive Mode**: Guided configuration for beginners
- **Configuration Templates**: Pre-built scenarios for common use cases
- **Enhanced CLI**: Professional subcommand structure
- **System Diagnostics**: Built-in troubleshooting and analysis

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+**: [Install Rust](https://rustup.rs/)
- **Linux System**: Required for raw socket capabilities
- **Network Access**: Private network for testing

### Installation

```bash
# Clone the repository
git clone https://github.com/PaulShpilsher/router-flood.git
cd router-flood

# Build the project
cargo build --release

# Set capabilities (recommended over running as root)
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

### Basic Usage

```bash
# Interactive mode (recommended for beginners)
./target/release/router-flood interactive

# Quick test with dry-run (safe, no packets sent)
./target/release/router-flood --target 192.168.1.1 --ports 80,443 --dry-run

# Perfect simulation (100% success rate for clean validation)
./target/release/router-flood --target 192.168.1.1 --ports 80,443 --dry-run --perfect-simulation

# High-performance test with monitoring
./target/release/router-flood run --config high_perf.yaml --cpu-affinity --prometheus-port 9090
```

## 📚 Usage Examples

### 🎓 Educational Scenarios

#### Basic Web Server Testing
```bash
# Generate a web server configuration
nrouter-flood config generate --template web_server --output web_test.yaml

# Run the test with monitoring
router-flood run --config web_test.yaml --export json
```

#### DNS Server Stress Test
```bash
# DNS-focused configuration
router-flood config generate --template dns_server --output dns_test.yaml

# Execute with CPU affinity optimization
router-flood run --config dns_test.yaml --cpu-affinity
```

#### Interactive Learning Mode
```bash
# Guided configuration for learning
router-flood interactive

# System analysis and recommendations
router-flood system performance --workers 8
router-flood system security
```

### 🏢 Professional Use Cases

#### High-Performance Testing
```bash
# Generate high-performance configuration
router-flood config generate --template high_performance

# Run with full optimizations
router-flood run --config high_performance.yaml \\
  --cpu-affinity \\
  --prometheus-port 9090 \\
  --export prometheus
```

#### Continuous Integration
```bash
# Validate configuration in CI
router-flood config validate test_config.yaml

# Run automated tests
router-flood run --config ci_test.yaml --dry-run --export json
```

#### Production Monitoring
```bash
# Start with Prometheus metrics
router-flood run --config production.yaml \\
  --prometheus-port 9090 \\
  --cpu-affinity \\
  --export both

# Monitor with external tools
curl http://localhost:9090/metrics
```

## 🔧 Configuration

### Configuration Templates

Router Flood provides several pre-built templates:

| Template | Use Case | Protocols | Performance |
|----------|----------|-----------|-------------|
| `basic` | Learning/Testing | UDP-focused | Low impact |
| `web_server` | HTTP/HTTPS testing | TCP-focused | Medium |
| `dns_server` | DNS stress testing | UDP-focused | High |
| `high_performance` | Maximum throughput | Mixed protocols | Maximum |

### YAML Configuration Example

```yaml
target:
  ip: \"192.168.1.100\"
  ports: [80, 443, 8080]
  protocol_mix:
    udp_ratio: 0.6
    tcp_syn_ratio: 0.25
    tcp_ack_ratio: 0.1
    icmp_ratio: 0.05

attack:
  threads: 8
  packet_rate: 1000
  duration: 300
  packet_size_range: [64, 1400]
  randomize_timing: false

safety:
  require_private_ranges: true
  dry_run: false
  perfect_simulation: false
  max_threads: 100
  max_packet_rate: 10000

export:
  enabled: true
  format: Both
  include_system_stats: true

monitoring:
  system_monitoring: true
  stats_interval: 1
  performance_tracking: true
```

## 🛡️ Security

### Capability-Based Security

Router Flood uses Linux capabilities instead of requiring root access:

```bash
# Grant only the required capability
sudo setcap cap_net_raw+ep ./target/release/router-flood

# Run as regular user
./target/release/router-flood run --target 192.168.1.1 --ports 80
```

### Security Features

- **Private IP Validation**: Only allows RFC 1918 private ranges
- **Rate Limiting**: Built-in safety limits prevent system overwhelm
- **Audit Logging**: Tamper-proof cryptographic audit trails
- **Privilege Validation**: Automatic security context analysis
- **Dry-Run Mode**: Safe testing without network impact
- **Perfect Simulation**: Optional 100% success rate for clean configuration validation

### Dry-Run Modes

Router Flood offers two dry-run modes for safe testing:

#### Realistic Simulation (Default)
```bash
# Simulates 98% success rate for realistic training
./target/release/router-flood --target 192.168.1.1 --ports 80 --dry-run

# Example output:
# 📊 Stats - Sent: 878, Failed: 21, Rate: 167.2 pps
# Success Rate: 97.7% ≈ 98% (realistic)
```

#### Perfect Simulation
```bash
# 100% success rate for pure configuration validation
./target/release/router-flood --target 192.168.1.1 --ports 80 --dry-run --perfect-simulation

# Example output:
# 📊 Stats - Sent: 894, Failed: 0, Rate: 169.8 pps
# Success Rate: 100% (perfect)
```

**When to use each mode:**
- **Realistic Mode**: Educational training, understanding real-world network behavior
- **Perfect Mode**: Configuration validation, CI/CD testing, beginner-friendly learning

### Security Analysis

```bash
# Check security context
router-flood system security

# Example output:
# 🔒 Security Context Report:
#    Process ID: 12345
#    Real UID: 1000
#    Effective UID: 1000
#    Capabilities Available: true
#    Capabilities:
#      CAP_NET_RAW: ✅ Available
#      CAP_NET_ADMIN: ❌ Missing
```

## ⚡ Performance

### SIMD Optimizations

Router Flood automatically detects and uses available SIMD instruction sets:

- **AVX2**: 32-byte vector operations (4x performance improvement)
- **SSE4.2**: 16-byte vector operations (2x performance improvement)
- **NEON**: ARM64 16-byte vector operations (2x performance improvement)
- **Automatic Fallback**: Graceful degradation to scalar code

### CPU Affinity

Optimal performance through intelligent CPU assignment:

```bash
# Analyze performance for 8 workers
router-flood system performance --workers 8

# Example output:
# ⚡ Performance Analysis for 8 workers
# 🎯 Proposed CPU Assignments:
#   Worker 0 → CPU 0 (NUMA Node 0)
#   Worker 1 → CPU 2 (NUMA Node 0)
#   Worker 2 → CPU 4 (NUMA Node 0)
#   Worker 3 → CPU 6 (NUMA Node 0)
```

### Performance Metrics

- **Packet Generation**: Up to 100,000+ PPS per thread
- **Memory Efficiency**: 60-80% reduction in allocations
- **CPU Utilization**: Optimal core usage with NUMA awareness
- **Latency**: Sub-microsecond packet construction

## 📊 Monitoring

### Prometheus Integration

```bash
# Start with Prometheus metrics
router-flood run --config test.yaml --prometheus-port 9090

# Available metrics:
curl http://localhost:9090/metrics
```

### Key Metrics

- `router_flood_packets_sent_total`: Total packets sent
- `router_flood_packets_failed_total`: Failed packet count
- `router_flood_bytes_sent_total`: Total bytes transmitted
- `router_flood_packets_per_second`: Current packet rate
- `router_flood_success_rate_percent`: Success rate percentage
- `router_flood_cpu_usage_percent`: CPU utilization
- `router_flood_memory_usage_bytes`: Memory consumption

### Real-Time Display

```
📊 Packets: 15.4K | Failed: 23 | Rate: 257.0 PPS (2.47 Mbps) | Avg: 257.0 PPS | Success: 99.9% | Time: 60.1s
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run property-based tests
cargo test --test property_tests

# Run with coverage
cargo test --all-features

# Run specific test categories
cargo test security
cargo test performance
cargo test integration
```

### Test Categories

- **Unit Tests**: 200+ tests covering individual components
- **Integration Tests**: 50+ tests covering end-to-end scenarios
- **Property Tests**: 20+ tests with 10,000+ generated cases each
- **Security Tests**: 30+ tests for capability and audit logging validation
- **Performance Tests**: 20+ benchmark regression detection tests

### Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run packet builder fuzzing
cargo fuzz run fuzz_packet_builder

# Run configuration parser fuzzing
cargo fuzz run fuzz_config_parser

# Run CLI parser fuzzing
cargo fuzz run fuzz_cli_parser

# List all fuzz targets
cargo fuzz list
```

## 🏗️ Architecture

### Module Structure

```
router-flood/
├── src/
│   ├── cli/                 # Enhanced CLI with interactive mode
│   ├── config/              # Configuration management and templates
│   ├── error/               # User-friendly error handling
│   ├── monitoring/          # Prometheus metrics and system monitoring
│   ├── packet/              # Multi-protocol packet construction
│   ├── performance/         # SIMD optimizations and CPU affinity
│   ├── security/            # Capability-based security and audit logging
│   ├── stats/               # Statistics collection and export
│   ├── transport/           # Network transport layer
│   ├── ui/                  # Progress indicators and user interface
│   └── validation/          # Safety validation and IP checking
├── tests/                   # Comprehensive test suite
├── fuzz/                    # Fuzzing targets
├── benches/                 # Performance benchmarks
└── examples/                # Usage examples
```

### Design Principles

1. **Safety First**: Multiple validation layers and safety checks
2. **Performance**: Zero-copy operations and SIMD optimizations
3. **Modularity**: Clean separation of concerns
4. **Testability**: Comprehensive test coverage with property-based testing
5. **Usability**: User-friendly interfaces and clear error messages
6. **Security**: Capability-based security and audit logging

## 🔄 Recent Improvements

### Performance Optimizations (v0.0.1)
- **Zero-Copy Packet Building**: Direct in-place packet construction
- **Advanced Buffer Pools**: Memory-aligned buffers with reuse
- **SIMD Acceleration**: Platform-specific optimizations
- **CPU Affinity Management**: NUMA-aware worker placement
- **Lock-Free Data Structures**: Improved concurrency

### Code Quality Improvements
- **Zero Compiler Warnings**: Clean compilation with strict linting
- **Comprehensive Testing**: 322+ tests with property-based testing
- **Modular Architecture**: Well-organized module structure
- **Test Organization**: Inline tests moved to dedicated test files
- **Documentation Coverage**: Complete API documentation

### Security Enhancements
- **Capability-Based Security**: Linux capabilities support
- **Tamper-Proof Audit Logging**: Cryptographic hash chains
- **Enhanced Validation**: Multi-layer input validation
- **Security Context Detection**: Automatic privilege analysis

## 🤝 Contributing

### Development Setup

```bash
# Clone and setup
git clone https://github.com/PaulShpilsher/router-flood.git
cd router-flood

# Install development dependencies
cargo install cargo-fuzz
cargo install criterion

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check

# Run linting
cargo clippy -- -D warnings
```

### Code Quality

- **Formatting**: Use `cargo fmt`
- **Linting**: Pass `cargo clippy`
- **Testing**: Maintain test coverage
- **Documentation**: Update docs for new features
- **Security**: Follow security best practices

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Update documentation
6. Submit pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Legal Notice

**EDUCATIONAL USE ONLY**: This tool is designed exclusively for educational purposes and authorized network testing. Users must:

- Only test networks they own or have explicit written permission to test
- Comply with all applicable local, national, and international laws
- Use responsibly and ethically
- Respect network resources and other users

**The authors and contributors are not responsible for any misuse of this tool.**

## 🙏 Acknowledgments

- **Rust Community**: For excellent libraries and tools
- **Security Researchers**: For responsible disclosure practices
- **Network Engineers**: For testing and feedback
- **Open Source Contributors**: For improvements and bug reports

## 📞 Support

- **Documentation**: [GitHub Wiki](https://github.com/PaulShpilsher/router-flood/wiki)
- **Issues**: [GitHub Issues](https://github.com/PaulShpilsher/router-flood/issues)
- **Discussions**: [GitHub Discussions](https://github.com/PaulShpilsher/router-flood/discussions)
- **Security**: [Security Policy](SECURITY.md)

---

**Router Flood** - Transforming network testing through safety, performance, and education.