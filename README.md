# 🚀 Router Flood - High-Performance Network Stress Tester

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-320%2B%20passing-green.svg)](#testing)
[![Security](https://img.shields.io/badge/security-capability--based-blue.svg)](#security)
[![Performance](https://img.shields.io/badge/performance-optimized-brightgreen.svg)](#performance)

## ⚠️ Important Disclaimer

**This tool is for educational and authorized testing purposes only.** Users are responsible for complying with all applicable laws and regulations. Unauthorized use against systems you don't own or lack permission to test is strictly prohibited and may be illegal. By using this software, you agree to use it responsibly and only on networks and systems you own or have explicit permission to test.

---

## 📑 Table of Contents

- [Key Features](#-key-features)
- [Quick Start](#-quick-start)
- [Architecture Overview](#-architecture-overview)
- [Configuration](#-configuration)
- [Testing](#-testing)
- [Performance Tuning](#-performance-tuning)
- [Security](#️-security)
- [Troubleshooting](#-troubleshooting)
- [Documentation](#-documentation)
- [Contributing](#-contributing)
- [License](#-license)

---

A high-performance network stress testing tool designed for educational purposes and authorized network testing scenarios. Router Flood features a simplified, KISS-principle architecture while maintaining critical performance optimizations.

## 🎯 Key Features

### 🛡️ **Safety-First Design**
- **Private IP Only**: Hard-coded validation for RFC 1918 private ranges
- **Capability-Based Security**: No root required (CAP_NET_RAW sufficient)
- **Rate Limiting**: Built-in safety limits and monitoring
- **Dry-Run Mode**: Safe testing without sending packets
- **Perfect Simulation**: 100% success rate option for configuration validation
- **Audit Logging**: Cryptographic integrity protection

### ⚡ **Performance Optimizations**
- **SIMD Operations**: AVX2/SSE4.2 for high-speed packet payload generation
- **Lock-Free Memory Pools**: Treiber stack algorithm for zero-allocation operations
- **CPU Affinity**: NUMA-aware worker thread pinning
- **Zero-Copy Packet Construction**: In-place buffer operations
- **Batched Statistics**: Local accumulation with periodic atomic flushes
- **Pre-computed RNG**: Batched random value generation for hot paths

### 📊 **Professional Monitoring**
- **Real-Time Statistics**: Live performance monitoring with batched updates
- **System Resource Tracking**: CPU, memory, and network usage
- **Protocol-Level Breakdown**: Detailed traffic analysis
- **Export Support**: JSON, CSV, and Prometheus metrics

### 🏗️ **Simplified Architecture**

```
Project Structure (51 files, ~6,700 LOC):
├── src/
│   ├── config/          # Configuration management
│   ├── network/         # Core networking (workers, target, flood)
│   ├── packet/          # Packet generation and protocols
│   ├── performance/     # CPU affinity, memory pools, SIMD
│   ├── protocols/       # IPv4, IPv6, TCP, UDP, ICMP implementations
│   ├── stats/           # Statistics collection and export
│   └── utils/           # RNG batching, RAII, validation
└── tests/               # Unit and integration tests
```

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+**: [Install Rust](https://rustup.rs/)
- **Linux System**: Required for raw socket capabilities (kernel 3.10+)
- **Network Access**: Private network for testing
- **Memory**: Minimum 512MB RAM (2GB+ recommended for high-throughput testing)
- **CPU**: Multi-core processor recommended for optimal performance

### Installation

```bash
# Clone the repository
git clone https://github.com/paulspilsher/router-flood.git
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

#### **Worker** (src/network/worker.rs)
- Single packet processing with rate limiting
- Pre-allocated buffers for zero-copy operations
- Pre-calculated packet type distribution based on protocol mix
- Local statistics batching (50 packets) before atomic flush

#### **Stats** (src/stats/stats_aggregator.rs)
- Simple atomic counters (AtomicU64) for lock-free operation
- BatchStats for worker-local accumulation
- Automatic flush on batch size threshold
- Export support for JSON/CSV formats

#### **MemoryPool** (src/performance/memory_pool.rs)
- Lock-free Treiber stack implementation
- Pre-allocated 64KB buffers
- Zero allocations after initialization
- Automatic buffer recycling via RAII guards

#### **SIMD** (src/performance/simd.rs)
- AVX2/SSE4.2 payload generation
- Runtime CPU feature detection
- Fallback to standard generation
- ~3-5x speedup for payload creation

### Performance Characteristics

| Component | Implementation | Performance Impact |
|-----------|---------------|-------------------|
| Payload Generation | SIMD (AVX2/SSE4.2) | 3-5x faster |
| Memory Management | Lock-free pool | Zero allocations |
| Statistics | Batched atomics | 50x reduction in atomic ops |
| RNG | Pre-computed batches | 40% less overhead |
| CPU Affinity | NUMA-aware pinning | 15-25% throughput gain |
| Packet Construction | Zero-copy buffers | 30% memory bandwidth saved |

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

load:
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

## 🔧 Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Permission denied | Run `sudo setcap cap_net_raw+ep ./target/release/router-flood` |
| Cannot open raw socket | Ensure you're on Linux with CAP_NET_RAW capability |
| High CPU usage | Reduce thread count or packet rate |
| Memory issues | Adjust buffer pool size in configuration |
| Target not reachable | Verify network connectivity and firewall rules |

## 📚 Documentation

- [Architecture Guide](./ARCHITECTURE.md) - Detailed component design
- [Performance Guide](./PERFORMANCE.md) - Optimization details
- [API Documentation](./docs/api.md) - Module documentation
- [Security Policy](./SECURITY.md) - Safety features
- [Contributing Guide](./CONTRIBUTING.md) - Development guidelines

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Rust community for excellent libraries and tools
- Contributors and testers who helped improve the project
- Security researchers for responsible disclosure

---

*Built with ❤️ in Rust for network professionals and security researchers*