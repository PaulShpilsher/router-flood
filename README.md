# Router Flood - Network Stress Testing Tool

A high-performance network stress testing CLI tool for authorized testing purposes only.

## ⚠️ Important

**Educational and authorized testing only.** Only use on networks you own or have explicit permission to test. Unauthorized use is illegal.

## Features

- **Safety-First**: Private IP validation, rate limiting, dry-run mode
- **High Performance**: SIMD operations, lock-free memory pools, CPU affinity
- **Monitoring**: Real-time statistics, protocol breakdown, JSON/CSV export

## Installation

```bash
git clone https://github.com/paulspilsher/router-flood.git
cd router-flood
cargo build --release
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

## Usage

```bash
# Dry run (no packets sent)
./target/release/router-flood --target 192.168.1.1 --ports 80,443 --dry-run

# Basic test
./target/release/router-flood --target 192.168.1.1 --ports 80 --threads 4 --rate 500

# With config file
./target/release/router-flood --config config.yaml
```

## Configuration

```yaml
target:
  ip: "192.168.1.1"
  ports: [80, 443]
  
load:
  threads: 4
  packet_rate: 1000
  duration: 60
```

## Testing

```bash
cargo test --all
cargo bench
```

## License

MIT License - See [LICENSE](LICENSE) file for details.