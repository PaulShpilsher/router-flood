# Usage Guide

## Basic Usage

```bash
router-flood --target <IP> --ports <PORTS> [OPTIONS]
```

## Common Examples

```bash
# Dry run (no packets sent)
router-flood --target 192.168.1.1 --ports 80,443 --dry-run

# Basic test with 4 threads
router-flood --target 192.168.1.1 --ports 80 --threads 4

# Rate-limited test
router-flood --target 192.168.1.1 --ports 80 --rate 1000

# Time-limited test (60 seconds)
router-flood --target 192.168.1.1 --ports 80 --duration 60

# Using config file
router-flood --config config.yaml
```

## Options

- `--target <IP>` - Target IP address (private IPs only)
- `--ports <PORTS>` - Comma-separated port list
- `--threads <N>` - Number of worker threads (default: 4)
- `--rate <N>` - Packets per second per thread (default: 1000)
- `--duration <SECS>` - Test duration in seconds
- `--dry-run` - Test configuration without sending packets
- `--config <FILE>` - Load configuration from YAML file
- `--export <FORMAT>` - Export stats (json/csv)

## Configuration File

```yaml
target:
  ip: "192.168.1.1"
  ports: [80, 443, 8080]

load:
  threads: 4
  packet_rate: 1000
  duration: 60
  
safety:
  dry_run: false
  rate_limit: 10000
```

## Safety Features

- Only works with private IP ranges (RFC 1918)
- Built-in rate limiting
- Dry-run mode for testing
- Requires CAP_NET_RAW capability (not root)