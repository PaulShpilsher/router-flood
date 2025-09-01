# router-flood

A high-performance network stress testing tool for authorized testing of network infrastructure resilience.

## ⚠️ Important

**Educational and authorized testing only.** Only use on networks you own or have explicit permission to test. Unauthorized use is illegal.

## Features

* **Safety-first design** - Enforces private IP ranges (RFC 1918), includes rate limiting and dry-run mode
* **High performance** - SIMD operations, lock-free memory pools, CPU affinity for optimal throughput
* **Multi-protocol support** - TCP (SYN/ACK), UDP, ICMP with configurable protocol mix
* **Real-time monitoring** - Live statistics with JSON/CSV export capabilities
* **Flexible configuration** - Command-line arguments or YAML configuration files

## Installation

### From source

```bash
# Clone and build
git clone https://github.com/paulspilsher/router-flood.git
cd router-flood
cargo build --release

# Grant network capabilities (recommended over running as root)
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

### Prerequisites

* Linux system with kernel 3.10+
* Rust 1.70+ (install via [rustup](https://rustup.rs/))
* Network interface with raw socket support

## Quick start

```bash
# Test configuration without sending packets
router-flood --target 192.168.1.1 --ports 80,443 --dry-run

# Basic stress test with 4 threads
router-flood --target 192.168.1.1 --ports 80 --threads 4 --rate 100

# Time-limited test
router-flood --target 192.168.1.1 --ports 80,443 --duration 60

# Using configuration file
router-flood --config stress-test.yaml
```

## Usage

See [USAGE.md](USAGE.md) for comprehensive usage documentation with examples.

### Command-line options

```
router-flood [OPTIONS]

OPTIONS:
    -t, --target <IP>           Target IP address (must be private range)
    -p, --ports <PORTS>         Target ports (comma-separated)
    --threads <NUM>             Number of worker threads [default: 4]
    --rate <PPS>                Packets per second per thread [default: 100]
    -d, --duration <SECONDS>    Test duration in seconds
    -c, --config <FILE>         Load configuration from YAML file
    -i, --interface <NAME>      Network interface to use
    --export <FORMAT>           Export statistics (json, csv, both)
    --dry-run                   Test configuration without sending packets
    --perfect-simulation        Use 100% success rate in dry-run mode
    --list-interfaces           List available network interfaces
```

### Common examples

```bash
# Test web server
router-flood --target 192.168.1.100 --ports 80,443 --threads 4 --rate 500

# DNS server stress test
router-flood --target 10.0.0.53 --ports 53 --threads 4 --rate 2000 --duration 120

# Safe configuration testing
router-flood --target 192.168.1.1 --ports 80 --dry-run --perfect-simulation
```

For configuration files, advanced usage, and more examples, see [USAGE.md](USAGE.md).

## Building from source

```bash
# Standard build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Build with specific features
cargo build --release --features "json-export,prometheus"
```

## Performance tuning

For optimal performance:

1. **CPU affinity**: Workers are automatically pinned to CPU cores
2. **Memory pools**: Pre-allocated buffers minimize allocation overhead
3. **SIMD operations**: Automatic detection and use of AVX2/SSE4.2 for payload generation
4. **Batch size**: Adjust statistics batch size for your workload (default: 50)

## Safety features

* **IP validation**: Only accepts RFC 1918 private addresses
* **Rate limiting**: Built-in limits prevent accidental network saturation
* **Resource limits**: Enforces reasonable thread and memory constraints
* **Dry-run mode**: Test configurations without network impact
* **Capability-based security**: Runs with minimal privileges (CAP_NET_RAW)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## Security

For security concerns, see [SECURITY.md](SECURITY.md) or report issues privately.

## License

MIT License - see [LICENSE](LICENSE) file for details.