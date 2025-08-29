# Router Flood - Comprehensive Usage Guide

## Table of Contents
- [Command Structure](#command-structure)
- [Global Options](#global-options)
- [Subcommands](#subcommands)
- [Common Use Cases](#common-use-cases)
- [Advanced Scenarios](#advanced-scenarios)
- [Safety Features](#safety-features)
- [Performance Tuning](#performance-tuning)
- [Monitoring & Export](#monitoring--export)
- [Troubleshooting](#troubleshooting)

## Command Structure

Router Flood uses a hierarchical command structure:

```
router-flood [GLOBAL_OPTIONS] [SUBCOMMAND] [SUBCOMMAND_OPTIONS]
```

### Main Commands
- `run` - Execute network stress test
- `config` - Configuration management
- `system` - System information and diagnostics
- `interactive` - Interactive configuration mode

## Global Options

These options apply to all subcommands:

```bash
# Verbose output (can be repeated for more detail)
router-flood -v run ...           # Basic verbose
router-flood -vv run ...          # More verbose
router-flood -vvv run ...         # Maximum verbosity

# Quiet mode (suppress non-essential output)
router-flood -q run ...

# Use configuration file
router-flood -c config.yaml run

# Combine options
router-flood -vv -c my_config.yaml run
```

## Subcommands

### `run` - Execute Network Test

```bash
# Basic usage with target and ports
router-flood run --target 192.168.1.1 --ports 80,443

# With specific thread count
router-flood run --target 192.168.1.1 --ports 80 --threads 8

# Set packet rate per thread
router-flood run --target 192.168.1.1 --ports 80 --rate 1000

# Limited duration test (60 seconds)
router-flood run --target 192.168.1.1 --ports 80 --duration 60

# Dry run mode (no packets sent)
router-flood run --target 192.168.1.1 --ports 80,443,8080 --dry-run

# Enable CPU affinity for better performance
router-flood run --target 192.168.1.1 --ports 80 --cpu-affinity

# Enable Prometheus metrics
router-flood run --target 192.168.1.1 --ports 80 --prometheus-port 9090

# Export statistics
router-flood run --target 192.168.1.1 --ports 80 --export json
router-flood run --target 192.168.1.1 --ports 80 --export csv
router-flood run --target 192.168.1.1 --ports 80 --export both
```

### `config` - Configuration Management

```bash
# Generate configuration templates
router-flood config generate --template basic
router-flood config generate --template web_server --output web.yaml
router-flood config generate --template dns_server --output dns.yaml
router-flood config generate --template high_performance --output perf.yaml

# List available templates
router-flood config list-templates

# Validate configuration file
router-flood config validate my_config.yaml
```

### `system` - System Diagnostics

```bash
# Display system information
router-flood system info

# Check security context and capabilities
router-flood system security

# Get performance recommendations
router-flood system performance
router-flood system performance --workers 16
```

### `interactive` - Interactive Mode

```bash
# Start interactive configuration
router-flood interactive
```

## Common Use Cases

### 1. Web Server Testing

```bash
# Test HTTP and HTTPS ports
router-flood run --target 192.168.1.100 --ports 80,443 --threads 4 --rate 500

# Test with custom web ports
router-flood run --target 192.168.1.100 --ports 8080,8443,3000 --duration 30

# Test multiple web servers
router-flood run --target 192.168.1.100 --ports 80,443,8080,8443 --threads 8
```

### 2. DNS Server Testing

```bash
# Standard DNS testing
router-flood run --target 192.168.1.53 --ports 53 --threads 2 --rate 1000

# DNS over HTTPS/TLS
router-flood run --target 192.168.1.53 --ports 53,853,443 --threads 4
```

### 3. Mail Server Testing

```bash
# SMTP testing
router-flood run --target 192.168.1.25 --ports 25,587,465 --threads 2 --rate 100

# Full mail server test
router-flood run --target 192.168.1.25 --ports 25,110,143,587,993,995
```

### 4. Game Server Testing

```bash
# Minecraft server
router-flood run --target 192.168.1.200 --ports 25565 --threads 4 --rate 200

# Multiple game ports
router-flood run --target 192.168.1.200 --ports 27015,27016,27017 --threads 6
```

### 5. IoT Device Testing

```bash
# MQTT broker
router-flood run --target 192.168.1.150 --ports 1883,8883 --threads 2 --rate 50

# CoAP testing
router-flood run --target 192.168.1.150 --ports 5683,5684 --threads 1 --rate 100
```

## Advanced Scenarios

### High-Performance Testing

```bash
# Maximum performance with CPU affinity
router-flood run \
  --target 192.168.1.1 \
  --ports 80,443 \
  --threads 16 \
  --rate 10000 \
  --cpu-affinity \
  --export prometheus \
  --prometheus-port 9090

# Sustained load test
router-flood run \
  --target 192.168.1.1 \
  --ports 80 \
  --threads 8 \
  --rate 5000 \
  --duration 3600 \
  --export json
```

### Multi-Protocol Testing

```bash
# Using configuration file for complex scenarios
cat > multi_protocol.yaml << EOF
target:
  ip: "192.168.1.1"
  ports: [80, 443, 53, 22]
  protocol_mix:
    udp_ratio: 0.3
    tcp_syn_ratio: 0.3
    tcp_ack_ratio: 0.2
    icmp_ratio: 0.1
    ipv6_ratio: 0.05
    arp_ratio: 0.05
attack:
  threads: 8
  packet_rate: 1000
  duration: 120
EOF

router-flood -c multi_protocol.yaml run
```

### Gradual Load Increase

```bash
# Start with low rate and increase
for rate in 100 500 1000 5000 10000; do
  echo "Testing at ${rate} pps..."
  router-flood run \
    --target 192.168.1.1 \
    --ports 80 \
    --rate $rate \
    --duration 30 \
    --export json
  sleep 10
done
```

### Comprehensive Network Scan

```bash
# Test common ports systematically
COMMON_PORTS="21,22,23,25,53,80,110,143,443,445,3306,3389,5432,8080,8443"
router-flood run \
  --target 192.168.1.1 \
  --ports $COMMON_PORTS \
  --threads 4 \
  --rate 100 \
  --duration 60
```

## Safety Features

### Dry Run Mode

Always test with dry-run first:

```bash
# Validate configuration without sending packets
router-flood run --target 192.168.1.1 --ports 80 --dry-run

# Dry run with full verbosity
router-flood -vvv run --target 192.168.1.1 --ports 80 --dry-run

# Dry run with export to verify output
router-flood run --target 192.168.1.1 --ports 80 --dry-run --export json
```

### Rate Limiting

```bash
# Conservative rate for testing
router-flood run --target 192.168.1.1 --ports 80 --rate 10 --threads 1

# Moderate rate for normal testing
router-flood run --target 192.168.1.1 --ports 80 --rate 100 --threads 2

# Higher rate with monitoring
router-flood run --target 192.168.1.1 --ports 80 --rate 1000 --threads 4 --prometheus-port 9090
```

### Private IP Validation

```bash
# These will work (private ranges)
router-flood run --target 192.168.1.1 --ports 80      # Class C private
router-flood run --target 10.0.0.1 --ports 80         # Class A private
router-flood run --target 172.16.0.1 --ports 80       # Class B private

# These will be rejected (public IPs)
router-flood run --target 8.8.8.8 --ports 53          # ERROR: Public IP
router-flood run --target 1.1.1.1 --ports 53          # ERROR: Public IP
```

## Performance Tuning

### CPU Affinity

```bash
# Enable CPU affinity for all workers
router-flood run --target 192.168.1.1 --ports 80 --cpu-affinity

# Check system performance recommendations first
router-flood system performance --workers 16
```

### Buffer Optimization

```bash
# Create optimized configuration
cat > optimized.yaml << EOF
buffers:
  packet_buffer_size: 65536
  batch_size: 128
  ring_buffer_size: 8192
  numa_aware: true
attack:
  threads: 8
  packet_rate: 5000
monitoring:
  stats_interval: 1
  system_monitoring: true
EOF

router-flood -c optimized.yaml run --target 192.168.1.1 --ports 80
```

### Thread Tuning

```bash
# Test different thread counts
for threads in 1 2 4 8 16; do
  echo "Testing with ${threads} threads..."
  router-flood run \
    --target 192.168.1.1 \
    --ports 80 \
    --threads $threads \
    --rate 1000 \
    --duration 30 \
    --export csv
done
```

## Monitoring & Export

### Prometheus Integration

```bash
# Start with Prometheus metrics
router-flood run \
  --target 192.168.1.1 \
  --ports 80 \
  --prometheus-port 9090 &

# Query metrics
curl http://localhost:9090/metrics
```

### Export Formats

```bash
# JSON export for processing
router-flood run --target 192.168.1.1 --ports 80 --duration 60 --export json
# Output: stats_[timestamp].json

# CSV for spreadsheets
router-flood run --target 192.168.1.1 --ports 80 --duration 60 --export csv
# Output: stats_[timestamp].csv

# Both formats
router-flood run --target 192.168.1.1 --ports 80 --duration 60 --export both
# Output: stats_[timestamp].json and stats_[timestamp].csv
```

### Real-time Monitoring

```bash
# Watch stats in real-time with verbose mode
router-flood -vv run --target 192.168.1.1 --ports 80 --rate 1000

# Monitor system resources during test
router-flood run --target 192.168.1.1 --ports 80 --rate 5000 &
PID=$!
while kill -0 $PID 2>/dev/null; do
  top -p $PID -n 1 -b | head -20
  sleep 5
done
```

## Troubleshooting

### Permission Issues

```bash
# Check current capabilities
router-flood system security

# Set required capabilities
sudo setcap cap_net_raw+ep $(which router-flood)

# Verify capabilities
getcap $(which router-flood)
```

### Performance Issues

```bash
# Check system recommendations
router-flood system performance

# Test with minimal configuration
router-flood run --target 192.168.1.1 --ports 80 --threads 1 --rate 10

# Gradually increase load
for rate in 10 50 100 500 1000; do
  router-flood run --target 192.168.1.1 --ports 80 --threads 2 --rate $rate --duration 10
done
```

### Network Interface Issues

```bash
# List available interfaces
ip link show

# Specify interface explicitly
router-flood run --target 192.168.1.1 --ports 80 --interface eth0

# Check interface statistics
ip -s link show eth0
```

### Configuration Validation

```bash
# Validate before running
router-flood config validate my_config.yaml

# Test with dry-run
router-flood -c my_config.yaml run --dry-run

# Check configuration with verbose output
router-flood -vvv config validate my_config.yaml
```

## Examples by Target Type

### Home Router

```bash
# Basic connectivity test
router-flood run --target 192.168.1.1 --ports 80,443,53 --threads 2 --rate 50 --duration 30

# Stress test with monitoring
router-flood run --target 192.168.1.1 --ports 80,443 --threads 4 --rate 500 --export json
```

### NAS Device

```bash
# File sharing protocols
router-flood run --target 192.168.1.100 --ports 445,139,2049,548 --threads 2 --rate 100

# Media server ports
router-flood run --target 192.168.1.100 --ports 8096,32400,8080 --threads 2 --rate 50
```

### Smart Home Hub

```bash
# IoT protocols
router-flood run --target 192.168.1.50 --ports 1883,8883,5683 --threads 1 --rate 20 --duration 60
```

### Development Server

```bash
# Common development ports
router-flood run --target 192.168.1.200 --ports 3000,3001,4200,5000,8000,8080,9000 --threads 4 --rate 200
```

## Best Practices

1. **Always start with dry-run**: Test configuration before sending packets
2. **Begin with low rates**: Start at 10-100 pps and increase gradually
3. **Monitor target health**: Watch for signs of overload or crashes
4. **Use appropriate thread counts**: 1-2 for IoT, 4-8 for servers, 8-16 for high-performance
5. **Export statistics**: Keep records of all tests for analysis
6. **Set duration limits**: Use --duration to prevent accidental long-running tests
7. **Check system security**: Run `system security` before testing
8. **Validate configurations**: Always validate YAML files before use
9. **Use templates**: Start with templates and modify as needed
10. **Document your tests**: Export results and keep logs for reference

## Tips and Tricks

```bash
# Create an alias for common tests
alias rftest='router-flood run --dry-run --export json'

# Quick validation test
rftest --target 192.168.1.1 --ports 80

# Function for gradual testing
test_gradual() {
  local target=$1
  local port=$2
  for rate in 10 100 1000; do
    router-flood run --target $target --ports $port --rate $rate --duration 10
    sleep 5
  done
}

# Use it
test_gradual 192.168.1.1 80
```

## Environment Variables

```bash
# Set default configuration file
export ROUTER_FLOOD_CONFIG="/path/to/default.yaml"

# Enable debug logging
export RUST_LOG=debug

# Set default export format
export ROUTER_FLOOD_EXPORT=json

# Run with environment settings
router-flood run --target 192.168.1.1 --ports 80
```