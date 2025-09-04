# Usage Guide

## Table of Contents

- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Command-Line Options](#command-line-options)
- [Configuration Files](#configuration-files)
- [Common Scenarios](#common-scenarios)
- [Advanced Usage](#advanced-usage)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)

## Installation

### Building from source

```bash
# Clone the repository
git clone https://github.com/paulspilsher/router-flood.git
cd router-flood

# Build in release mode for optimal performance
cargo build --release

# The binary will be at ./target/release/router-flood
```

### Setting up capabilities

Instead of running as root, grant the necessary network capabilities:

```bash
# Grant CAP_NET_RAW capability
sudo setcap cap_net_raw+ep ./target/release/router-flood

# Verify capabilities are set
getcap ./target/release/router-flood
# Should output: ./target/release/router-flood = cap_net_raw+ep
```

### First run

```bash
# Test the installation with a dry run
./target/release/router-flood --target 192.168.1.1 --ports 80 --dry-run

# List available network interfaces
./target/release/router-flood --list-interfaces
```

## Basic Usage

### Minimal example

```bash
# Basic stress test with default settings
router-flood --target 192.168.1.1 --ports 80
```

This will:
- Target IP 192.168.1.1 on port 80
- Use 4 worker threads (default)
- Send 100 packets per second per thread (default)
- Run indefinitely until stopped with Ctrl+C

### Specifying multiple ports

```bash
# Test multiple services simultaneously
router-flood --target 192.168.1.1 --ports 80,443,8080,3306
```

### Controlled duration test

```bash
# Run for exactly 60 seconds
router-flood --target 192.168.1.1 --ports 80 --duration 60
```

### Adjusting intensity

```bash
# Low intensity - good for initial testing
router-flood --target 192.168.1.1 --ports 80 --threads 2 --rate 50

# Medium intensity
router-flood --target 192.168.1.1 --ports 80 --threads 4 --rate 500

# High intensity - ensure your network can handle this
router-flood --target 192.168.1.1 --ports 80 --threads 8 --rate 2000
```

## Command-Line Options

### Target specification

| Option | Description | Example |
|--------|-------------|---------|
| `--target`, `-t` | Target IP address (must be private range) | `--target 192.168.1.1` |
| `--ports`, `-p` | Comma-separated list of ports | `--ports 80,443,8080` |
| `--interface`, `-i` | Network interface to use | `--interface eth0` |

### Load configuration

| Option | Description | Example | Default |
|--------|-------------|---------|---------|
| `--threads` | Number of worker threads | `--threads 8` | 4 |
| `--rate` | Packets per second per thread | `--rate 1000` | 100 |
| `--duration`, `-d` | Test duration in seconds | `--duration 300` | Unlimited |

### Safety options

| Option | Description | Example |
|--------|-------------|---------|
| `--dry-run` | Test without sending packets (98% success rate) | `--dry-run` |
| `--perfect-simulation` | 100% success rate in dry-run mode | `--dry-run --perfect-simulation` |

### Output and logging options

| Option | Description | Example |
|--------|-------------|---------|
| `--export` | Export format (json, csv, yaml, text) | `--export json` |
| `--config`, `-c` | Load settings from YAML file | `--config test.yaml` |
| `--audit-log` | Custom audit log file path | `--audit-log /var/log/audit.log` |
| `--list-interfaces` | List available network interfaces | `--list-interfaces` |

## Configuration Files

### Basic configuration

Create a `config.yaml` file:

```yaml
# config.yaml
target:
  ip: "192.168.1.1"
  ports: [80, 443]

attack:
  threads: 4
  packet_rate: 500
  duration: 60
```

Run with:
```bash
router-flood --config config.yaml
```

### Advanced configuration

```yaml
# advanced-config.yaml
target:
  ip: "192.168.1.100"
  ports: [80, 443, 8080, 8443]
  protocol_mix:
    udp_ratio: 0.3
    tcp_syn_ratio: 0.3
    tcp_ack_ratio: 0.2
    icmp_ratio: 0.2

attack:
  threads: 8
  packet_rate: 1000
  duration: 300
  packet_size_range: [64, 1400]

safety:
  dry_run: false
  perfect_simulation: false  # Only applies when dry_run is true
  rate_limit: 10000
  max_bandwidth_mbps: 100

monitoring:
  export_format: json
  export_path: "./results/"
  stats_interval: 5

audit:
  enabled: true
  log_file: "/var/log/router-flood/audit.log"
```

### Protocol mix configuration

Control the distribution of packet types:

```yaml
# protocol-mix.yaml
target:
  ip: "192.168.1.1"
  ports: [80, 443]
  protocol_mix:
    udp_ratio: 0.4      # 40% UDP packets
    tcp_syn_ratio: 0.3   # 30% TCP SYN packets
    tcp_ack_ratio: 0.2   # 20% TCP ACK packets
    icmp_ratio: 0.1      # 10% ICMP packets
```

## Common Scenarios

### Testing web server resilience

```bash
# Simulate typical web traffic
router-flood \
  --target 192.168.1.100 \
  --ports 80,443 \
  --threads 6 \
  --rate 1000 \
  --duration 300

# High-load web server test
router-flood \
  --target 192.168.1.100 \
  --ports 80,443,8080 \
  --threads 8 \
  --rate 5000 \
  --duration 60
```

### DNS server stress testing

```bash
# Basic DNS load test
router-flood \
  --target 192.168.1.53 \
  --ports 53 \
  --threads 4 \
  --rate 2000

# Mixed TCP/UDP DNS test (using config file)
cat > dns-test.yaml << EOF
target:
  ip: "192.168.1.53"
  ports: [53]
  protocol_mix:
    udp_ratio: 0.8
    tcp_syn_ratio: 0.2
attack:
  threads: 4
  packet_rate: 3000
EOF

router-flood --config dns-test.yaml
```

### Database server testing

```bash
# MySQL/MariaDB stress test
router-flood \
  --target 192.168.1.200 \
  --ports 3306 \
  --threads 4 \
  --rate 500

# PostgreSQL stress test
router-flood \
  --target 192.168.1.201 \
  --ports 5432 \
  --threads 4 \
  --rate 500

# MongoDB stress test
router-flood \
  --target 192.168.1.202 \
  --ports 27017 \
  --threads 4 \
  --rate 500
```

### Multi-service testing

```bash
# Test multiple services on one host
router-flood \
  --target 192.168.1.1 \
  --ports 22,80,443,3306,5432,6379,8080,9000 \
  --threads 8 \
  --rate 200
```

### Gradual load increase

```bash
# Start with low load
router-flood --target 192.168.1.1 --ports 80 --threads 1 --rate 50 --duration 60

# Increase to medium load
router-flood --target 192.168.1.1 --ports 80 --threads 2 --rate 200 --duration 60

# Increase to high load
router-flood --target 192.168.1.1 --ports 80 --threads 4 --rate 500 --duration 60

# Maximum load test
router-flood --target 192.168.1.1 --ports 80 --threads 8 --rate 1000 --duration 60
```

## Advanced Usage

### Using specific network interfaces

```bash
# List available interfaces
router-flood --list-interfaces

# Use specific interface
router-flood \
  --target 192.168.1.1 \
  --ports 80 \
  --interface eth1
```

### Exporting statistics

```bash
# Export to JSON
router-flood \
  --target 192.168.1.1 \
  --ports 80 \
  --duration 60 \
  --export json

# Export to CSV
router-flood \
  --target 192.168.1.1 \
  --ports 80 \
  --duration 60 \
  --export csv

# Export to YAML format
router-flood \
  --target 192.168.1.1 \
  --ports 80 \
  --duration 60 \
  --export yaml
```

### Validation and testing

#### Dry-run modes

The dry-run feature allows safe testing without sending actual network packets:

- **Standard dry-run**: Simulates realistic network conditions with ~98% success rate
- **Perfect simulation**: 100% success rate for pure configuration validation

```bash
# Standard dry-run - simulates realistic packet loss (98% success rate)
router-flood \
  --target 192.168.1.1 \
  --ports 80 \
  --dry-run

# Perfect simulation - no simulated failures (100% success rate)
router-flood \
  --target 192.168.1.1 \
  --ports 80 \
  --dry-run \
  --perfect-simulation

# Test with specific packet sizes
cat > packet-size-test.yaml << EOF
target:
  ip: "192.168.1.1"
  ports: [80]
attack:
  packet_size_range: [1400, 1400]  # Fixed 1400-byte packets
EOF

router-flood --config packet-size-test.yaml
```

### Bandwidth-limited testing

```yaml
# bandwidth-limit.yaml
target:
  ip: "192.168.1.1"
  ports: [80, 443]

attack:
  threads: 4
  packet_rate: 1000

safety:
  max_bandwidth_mbps: 10  # Limit to 10 Mbps
  rate_limit: 5000        # Max 5000 pps total
```

## Performance Tuning

### CPU affinity

The tool automatically pins worker threads to CPU cores for optimal performance. To verify:

```bash
# Check CPU usage per core during test
htop  # or top, press '1' to see per-core usage
```

### Memory optimization

```bash
# For long-running tests, monitor memory usage
router-flood --target 192.168.1.1 --ports 80 &
PID=$!
watch -n 1 "ps -o pid,vsz,rss,comm -p $PID"
```

### Optimal thread counts

```bash
# Get CPU count
nproc

# Use 50-75% of available cores for testing
# For 8-core system:
router-flood --target 192.168.1.1 --ports 80 --threads 6
```

### Rate limiting considerations

```bash
# Calculate total packets per second
# total_pps = threads * rate
# Example: 4 threads * 1000 rate = 4000 pps total

# Conservative test
router-flood --threads 2 --rate 100  # 200 pps total

# Moderate test
router-flood --threads 4 --rate 500  # 2000 pps total

# Aggressive test
router-flood --threads 8 --rate 2000  # 16000 pps total
```

## Troubleshooting

### Common issues and solutions

#### Permission denied

```bash
# Error: Permission denied (os error 13)
# Solution: Set capabilities
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

#### Cannot create raw socket

```bash
# Error: Cannot create raw socket
# Solution 1: Check capabilities
getcap ./target/release/router-flood

# Solution 2: Run with sudo (not recommended)
sudo ./target/release/router-flood --target 192.168.1.1 --ports 80
```

#### Target not in private range

```bash
# Error: Invalid IP range
# Solution: Only RFC 1918 private IPs are allowed
# Valid ranges:
#   192.168.0.0/16
#   10.0.0.0/8
#   172.16.0.0/12
```

#### High CPU usage

```bash
# Reduce thread count
router-flood --target 192.168.1.1 --ports 80 --threads 2

# Reduce packet rate
router-flood --target 192.168.1.1 --ports 80 --rate 50
```

#### Network interface not found

```bash
# List available interfaces
router-flood --list-interfaces

# Use the correct interface name
router-flood --target 192.168.1.1 --ports 80 --interface enp0s3
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug router-flood --target 192.168.1.1 --ports 80

# Trace-level logging (very verbose)
RUST_LOG=trace router-flood --target 192.168.1.1 --ports 80

# Test configuration only
router-flood --target 192.168.1.1 --ports 80 --dry-run
```

### Monitoring during tests

```bash
# Terminal 1: Run the test
router-flood --target 192.168.1.1 --ports 80

# Terminal 2: Monitor network traffic
sudo tcpdump -i eth0 -n host 192.168.1.1

# Terminal 3: Monitor system resources
htop

# Terminal 4: Monitor network statistics
watch -n 1 'netstat -i'
```

## Best Practices

1. **Always start with dry-run**: Test your configuration before sending packets
2. **Begin with low rates**: Start at 100 pps and gradually increase
3. **Monitor target health**: Watch target system resources during testing
4. **Use time limits**: Set duration to prevent accidental extended tests
5. **Document your tests**: Export statistics for analysis
6. **Test incrementally**: Gradually increase load to find breaking points
7. **Verify authorization**: Always have written permission before testing

## Example Test Progression

```bash
# 1. Verify configuration
router-flood --target 192.168.1.100 --ports 80,443 --dry-run

# 2. Low-intensity baseline (1 minute)
router-flood --target 192.168.1.100 --ports 80,443 --threads 2 --rate 100 --duration 60

# 3. Medium intensity (2 minutes)
router-flood --target 192.168.1.100 --ports 80,443 --threads 4 --rate 500 --duration 120

# 4. High intensity (1 minute)
router-flood --target 192.168.1.100 --ports 80,443 --threads 6 --rate 1000 --duration 60

# 5. Stress test (30 seconds)
router-flood --target 192.168.1.100 --ports 80,443 --threads 8 --rate 2000 --duration 30

# 6. Export and analyze results
router-flood --target 192.168.1.100 --ports 80,443 --threads 4 --rate 500 --duration 60 --export json
```