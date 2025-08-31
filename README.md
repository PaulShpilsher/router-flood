# 🚀 Router Flood - High-Performance Network Stress Tester

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-320%2B%20passing-green.svg)](#testing)
[![Security](https://img.shields.io/badge/security-capability--based-blue.svg)](#security)
[![Performance](https://img.shields.io/badge/performance-optimized-brightgreen.svg)](#performance)

A comprehensive, safety-first network testing tool designed for educational purposes and authorized network testing scenarios. Router Flood features a streamlined architecture with consolidated, high-performance components.

## 🎯 Key Features

### 🛡️ **Safety-First Design**
- **Private IP Only**: Hard-coded validation for RFC 1918 private ranges
- **Capability-Based Security**: No root required (CAP_NET_RAW sufficient)
- **Rate Limiting**: Built-in safety limits and monitoring
- **Dry-Run Mode**: Safe testing without sending packets
- **Perfect Simulation**: 100% success rate option for configuration validation
- **Audit Logging**: Cryptographic integrity protection

### ⚡ **Optimized Architecture**
- **Lock-Free Statistics**: 33-85% faster stats collection with per-CPU counters
- **Batch Processing**: 20-40% throughput improvement via packet batching
- **Zero-Copy Operations**: Buffer reuse and in-place packet construction
- **Simplified Traits**: Removed async overhead for better performance
- **Consolidated Components**: Single implementation per concept

### 📊 **Professional Monitoring**
- **Real-Time Statistics**: Live performance monitoring with batched updates
- **System Resource Tracking**: CPU, memory, and network usage
- **Protocol-Level Breakdown**: Detailed traffic analysis
- **Export Support**: JSON, CSV, and Prometheus metrics

### 🏗️ **Streamlined Architecture**

```
Core Components:
├── BatchWorker         # High-performance packet generation with batching
├── FloodStatsTracker   # Lock-free statistics with per-CPU counters
├── BufferPool          # Zero-allocation buffer management
└── Simple Traits       # Direct dispatch without async overhead
```

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+**: [Install Rust](https://rustup.rs/)
- **Linux System**: Required for raw socket capabilities
- **Network Access**: Private network for testing

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/router-flood.git
cd router-flood

# Build with optimizations
cargo build --release

# Set capabilities (recommended over running as root)
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

### Basic Usage

```bash
# Safe test with dry-run (no packets sent)
./target/release/router-flood --target 192.168.1.1 --ports 80,443 --dry-run

# Test with specific parameters
./target/release/router-flood --target 192.168.1.1 --ports 80 --threads 4 --rate 500 --duration 30

# Using configuration file
./target/release/router-flood --config config.yaml
```

## 📦 Architecture Overview

### Core Components

#### **BatchWorker** (Packet Generation)
- Batch processing with 50-packet batches
- Zero-copy buffer reuse
- Pre-calculated packet type distribution
- Local statistics batching

#### **FloodStatsTracker** (Statistics)
- Lock-free implementation with atomic operations
- Per-CPU cache-aligned counters
- Automatic batched aggregation
- Export capabilities (JSON, CSV, Prometheus)

#### **BufferPool** (Memory Management)
- Lock-free buffer allocation
- Pre-allocated buffer pool
- Zero allocations in steady state
- Automatic buffer recycling

### Performance Characteristics

| Component | Optimization | Performance Gain |
|-----------|-------------|------------------|
| Statistics | Lock-free counters | 33-85% faster |
| Workers | Batch processing | 20-40% throughput |
| Memory | Buffer pooling | 60-80% fewer allocations |
| Traits | No async overhead | 10-15% faster dispatch |

## 🔧 Configuration

### Basic Configuration

```yaml
target:
  ip: "192.168.1.1"
  ports: [80, 443, 8080]
  protocol_mix:
    udp_ratio: 0.6
    tcp_syn_ratio: 0.3
    icmp_ratio: 0.1

attack:
  threads: 4
  packet_rate: 1000
  duration: 60
  packet_size_range: [64, 1400]

safety:
  dry_run: false
  perfect_simulation: false
  rate_limit: 10000
```

### Advanced Features

```bash
# List network interfaces
./target/release/router-flood --list-interfaces

# Monitor system resources
./target/release/router-flood --target 192.168.1.1 --monitor

# Export statistics
./target/release/router-flood --target 192.168.1.1 --export stats.json
```

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Run benchmarks
cargo bench

# Property-based testing
cargo test --test property_tests

# Performance verification
cargo test --test performance_verification
```

## 📊 Performance Tuning

### Optimization Tips

1. **CPU Affinity**: Pin workers to specific cores
2. **Batch Size**: Adjust batch size for workload (default: 50)
3. **Buffer Pool**: Pre-allocate sufficient buffers
4. **Rate Control**: Use appropriate rate limits

### Benchmarking

```bash
# Run performance benchmarks
cargo bench --bench packet_generation
cargo bench --bench stats_collection
cargo bench --bench buffer_pool
```

## 🛡️ Security

### Safety Features

- **Private IP Validation**: Only RFC 1918 addresses allowed
- **Rate Limiting**: Prevents accidental DoS
- **Audit Logging**: Tamper-proof activity logs
- **Capability-Based**: No root required

### Best Practices

1. Always test in controlled environments
2. Use dry-run mode for initial testing
3. Monitor system resources during tests
4. Keep audit logs for compliance

## 📚 Documentation

- [API Documentation](./API_DOCUMENTATION.md)
- [Architecture Guide](./ARCHITECTURE.md)
- [Performance Guide](./PERFORMANCE_GUIDE.md)
- [Security Policy](./SECURITY.md)
- [Contributing Guide](./CONTRIBUTING.md)

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This tool is for **educational and authorized testing purposes only**. Users are responsible for complying with all applicable laws and regulations. Unauthorized use against systems you don't own or lack permission to test is strictly prohibited and may be illegal.

## 🙏 Acknowledgments

- Rust community for excellent libraries and tools
- Contributors and testers who helped improve the project
- Security researchers for responsible disclosure

---

*Built with ❤️ in Rust for network professionals and security researchers*