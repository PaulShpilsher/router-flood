# Router Flood - Production Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying the router-flood tool in production environments for educational and authorized network testing purposes.

## 🚨 **CRITICAL SAFETY NOTICE**

**⚠️ EDUCATIONAL USE ONLY ⚠️**

- This tool is designed **exclusively** for educational purposes and authorized network testing
- **NEVER** use this tool against networks you don't own or lack explicit written permission to test
- Unauthorized use may be **illegal** and could result in serious legal consequences
- Always comply with local, national, and international laws and regulations
- Obtain proper authorization before conducting any network testing

## Prerequisites

### System Requirements

#### Minimum Requirements
- **OS**: Linux (Ubuntu 20.04+, CentOS 8+, RHEL 8+)
- **CPU**: 2 cores, 2.4 GHz
- **RAM**: 4 GB
- **Storage**: 1 GB free space
- **Network**: Ethernet interface with raw socket support

#### Recommended Requirements
- **OS**: Linux (Ubuntu 22.04 LTS)
- **CPU**: 4+ cores, 3.0+ GHz
- **RAM**: 8+ GB
- **Storage**: 10+ GB free space (for logs and exports)
- **Network**: Gigabit Ethernet interface

#### Software Dependencies
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall -y "Development Tools"
sudo yum install -y openssl-devel pkg-config

# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Permissions and Security

#### Root Privileges
Router-flood requires root privileges for raw socket access:
```bash
# Run with sudo
sudo ./router-flood --config production.yaml

# Or use capabilities (recommended)
sudo setcap cap_net_raw+ep ./router-flood
```

#### Network Interface Access
Ensure the target network interface is available:
```bash
# List available interfaces
ip link show

# Verify interface is up
sudo ip link set eth0 up
```

## Installation

### From Source (Recommended)

```bash
# Clone repository
git clone https://github.com/your-org/router-flood.git
cd router-flood

# Build optimized release
cargo build --release

# Install to system path (optional)
sudo cp target/release/router-flood /usr/local/bin/
sudo chmod +x /usr/local/bin/router-flood
```

### Binary Installation

```bash
# Download latest release
wget https://github.com/your-org/router-flood/releases/latest/download/router-flood-linux-x86_64.tar.gz

# Extract and install
tar -xzf router-flood-linux-x86_64.tar.gz
sudo cp router-flood /usr/local/bin/
sudo chmod +x /usr/local/bin/router-flood
```

## Configuration

### Production Configuration Template

Create a production configuration file:

```yaml
# production.yaml
target:
  ip: "192.168.100.1"  # MUST be private IP range
  ports: [80, 443, 8080, 8443]
  protocol_mix:
    udp_ratio: 0.6
    tcp_syn_ratio: 0.25
    tcp_ack_ratio: 0.05
    icmp_ratio: 0.05
    ipv6_ratio: 0.03
    arp_ratio: 0.02

attack:
  threads: 4
  packet_rate: 1000  # packets per second per thread
  packet_size_range: [64, 1400]
  duration: 300  # 5 minutes
  burst_pattern:
    type: "sustained"
    rate: 1000
  randomize_timing: true

safety:
  dry_run: false
  max_duration: 3600  # 1 hour maximum
  require_confirmation: true
  audit_logging: true

monitoring:
  stats_interval: 5
  export_interval: 60
  dashboard_enabled: true
  real_time_alerts: true

export:
  enabled: true
  format: "both"  # json, csv, both
  filename_pattern: "production_test"
  include_system_stats: true
  compression: true

network:
  interface: "eth0"  # Specify interface
  buffer_size: 4096
  send_timeout: 1000  # milliseconds
```

### Environment Variables

```bash
# Optional environment variables
export RUST_LOG=info                    # Logging level
export ROUTER_FLOOD_CONFIG=production.yaml
export ROUTER_FLOOD_EXPORT_DIR=/var/log/router-flood
export ROUTER_FLOOD_MAX_THREADS=8
```

## Deployment Scenarios

### Scenario 1: Educational Lab Testing

**Use Case**: University network lab for teaching network security concepts

```yaml
# lab-config.yaml
target:
  ip: "192.168.1.1"  # Lab router
  ports: [80, 443]

attack:
  threads: 2
  packet_rate: 100
  duration: 60  # 1 minute tests

safety:
  dry_run: false
  require_confirmation: true
  audit_logging: true

monitoring:
  dashboard_enabled: true
  real_time_alerts: true
```

**Deployment**:
```bash
# Create dedicated user
sudo useradd -r -s /bin/false router-flood

# Set up directories
sudo mkdir -p /var/log/router-flood
sudo chown router-flood:router-flood /var/log/router-flood

# Run test
sudo -u router-flood router-flood --config lab-config.yaml
```

### Scenario 2: Network Infrastructure Testing

**Use Case**: IT department testing router resilience

```yaml
# infrastructure-test.yaml
target:
  ip: "10.0.1.1"  # Internal router
  ports: [22, 80, 443, 8080]

attack:
  threads: 8
  packet_rate: 2000
  duration: 1800  # 30 minutes
  burst_pattern:
    type: "ramp"
    start_rate: 500
    end_rate: 2000
    ramp_duration: 300

monitoring:
  stats_interval: 1  # More frequent monitoring
  export_interval: 30
  dashboard_enabled: true

export:
  enabled: true
  format: "both"
  include_system_stats: true
```

### Scenario 3: Security Research

**Use Case**: Authorized penetration testing

```yaml
# pentest-config.yaml
target:
  ip: "172.16.1.1"  # Target system
  ports: [21, 22, 23, 53, 80, 110, 143, 443, 993, 995]

attack:
  threads: 16
  packet_rate: 5000
  duration: 3600  # 1 hour
  burst_pattern:
    type: "bursts"
    burst_size: 1000
    burst_interval_ms: 100

safety:
  audit_logging: true
  require_confirmation: true

monitoring:
  real_time_alerts: true
  alert_thresholds:
    cpu_usage: 85.0
    memory_usage: 90.0
    success_rate: 95.0
```

## Monitoring and Observability

### Real-Time Dashboard

Enable the built-in dashboard:
```bash
# Start with dashboard
router-flood --config production.yaml --dashboard

# Dashboard will display:
# - Packets per second
# - Success rate
# - System resource usage
# - Active alerts
# - Performance metrics
```

### Metrics Export

Configure automatic metrics export:
```yaml
export:
  enabled: true
  format: "prometheus"  # For Grafana integration
  output_path: "/var/log/router-flood/metrics"
  export_interval: 30
```

### Integration with Monitoring Systems

#### Prometheus + Grafana
```bash
# Export Prometheus metrics
router-flood --config production.yaml --export-prometheus

# Configure Prometheus scraping
# prometheus.yml:
scrape_configs:
  - job_name: 'router-flood'
    static_configs:
      - targets: ['localhost:9090']
    file_sd_configs:
      - files: ['/var/log/router-flood/metrics/*.txt']
```

#### ELK Stack Integration
```bash
# Export JSON logs for Elasticsearch
router-flood --config production.yaml --export-json

# Configure Filebeat
# filebeat.yml:
filebeat.inputs:
- type: log
  paths:
    - /var/log/router-flood/*.json
  json.keys_under_root: true
```

## Performance Tuning

### System Optimization

#### Network Buffer Tuning
```bash
# Increase network buffers
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.netdev_max_backlog = 5000' >> /etc/sysctl.conf
sysctl -p
```

#### CPU Affinity
```bash
# Pin to specific CPU cores
taskset -c 0,1 router-flood --config production.yaml
```

#### Memory Optimization
```bash
# Increase file descriptor limits
echo 'router-flood soft nofile 65536' >> /etc/security/limits.conf
echo 'router-flood hard nofile 65536' >> /etc/security/limits.conf
```

### Application Tuning

#### High-Performance Configuration
```yaml
attack:
  threads: 16  # Match CPU cores
  packet_rate: 10000  # High rate
  
performance:
  buffer_pool_size: 1000
  batch_size: 100
  zero_copy: true
  
network:
  buffer_size: 8192
  send_timeout: 500
```

#### Memory-Optimized Configuration
```yaml
performance:
  buffer_pool_size: 100  # Smaller pool
  max_packet_history: 500
  
monitoring:
  stats_interval: 10  # Less frequent
  export_interval: 300
```

## Security Considerations

### Network Isolation

#### VLAN Isolation
```bash
# Create isolated VLAN for testing
sudo vconfig add eth0 100
sudo ip addr add 192.168.100.10/24 dev eth0.100
sudo ip link set eth0.100 up
```

#### Firewall Rules
```bash
# Restrict outbound traffic
sudo iptables -A OUTPUT -d 192.168.0.0/16 -j ACCEPT
sudo iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT
sudo iptables -A OUTPUT -d 172.16.0.0/12 -j ACCEPT
sudo iptables -A OUTPUT -j DROP
```

### Access Control

#### User Permissions
```bash
# Create dedicated group
sudo groupadd router-flood-users

# Add authorized users
sudo usermod -a -G router-flood-users username

# Set file permissions
sudo chown root:router-flood-users /usr/local/bin/router-flood
sudo chmod 750 /usr/local/bin/router-flood
```

#### Audit Logging
```yaml
safety:
  audit_logging: true
  audit_log_path: "/var/log/router-flood/audit.log"
  log_level: "info"
  
# Audit log format:
# 2024-12-19T10:30:00Z [INFO] Session started: user=testuser, target=192.168.1.1, duration=300s
# 2024-12-19T10:35:00Z [INFO] Session completed: packets_sent=150000, success_rate=99.2%
```

## Troubleshooting

### Common Issues

#### Permission Denied
```bash
# Error: Permission denied (raw socket)
# Solution: Run with sudo or set capabilities
sudo setcap cap_net_raw+ep ./router-flood
```

#### Interface Not Found
```bash
# Error: Network interface 'eth0' not found
# Solution: List and specify correct interface
ip link show
router-flood --interface eth1 --config production.yaml
```

#### High CPU Usage
```bash
# Monitor CPU usage
top -p $(pgrep router-flood)

# Reduce load
# - Decrease thread count
# - Lower packet rate
# - Enable rate limiting
```

#### Memory Issues
```bash
# Monitor memory usage
ps aux | grep router-flood

# Solutions:
# - Reduce buffer pool size
# - Decrease packet history
# - Enable compression
```

### Debugging

#### Enable Debug Logging
```bash
RUST_LOG=debug router-flood --config production.yaml
```

#### Performance Profiling
```bash
# Install perf tools
sudo apt install linux-tools-generic

# Profile CPU usage
sudo perf record -g ./router-flood --config production.yaml
sudo perf report
```

#### Network Analysis
```bash
# Monitor network traffic
sudo tcpdump -i eth0 -c 100

# Check interface statistics
cat /proc/net/dev
```

## Compliance and Legal

### Documentation Requirements

#### Test Authorization
- Written authorization from network owner
- Scope and duration of testing
- Emergency contact information
- Incident response procedures

#### Audit Trail
- Complete session logs
- Performance metrics
- System resource usage
- Any incidents or anomalies

### Reporting Template

```markdown
# Network Testing Report

## Test Details
- **Date**: 2024-12-19
- **Duration**: 30 minutes
- **Target**: 192.168.1.1 (Internal lab router)
- **Authorization**: Lab Manager John Doe

## Test Configuration
- **Threads**: 4
- **Packet Rate**: 1000 pps
- **Protocols**: UDP (60%), TCP (30%), ICMP (10%)

## Results
- **Packets Sent**: 1,800,000
- **Success Rate**: 99.5%
- **Peak CPU**: 45%
- **Peak Memory**: 128 MB

## Observations
- Router handled load well
- No packet loss detected
- Response times remained stable

## Recommendations
- Router can handle current load
- Consider upgrading for 2x capacity
- Monitor during peak hours
```

## Maintenance

### Regular Tasks

#### Log Rotation
```bash
# Configure logrotate
cat > /etc/logrotate.d/router-flood << EOF
/var/log/router-flood/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 router-flood router-flood
}
EOF
```

#### Cleanup Scripts
```bash
#!/bin/bash
# cleanup.sh - Remove old export files

EXPORT_DIR="/var/log/router-flood/exports"
DAYS_TO_KEEP=30

find "$EXPORT_DIR" -name "*.json" -mtime +$DAYS_TO_KEEP -delete
find "$EXPORT_DIR" -name "*.csv" -mtime +$DAYS_TO_KEEP -delete

echo "Cleanup completed: $(date)"
```

#### Health Checks
```bash
#!/bin/bash
# health-check.sh

# Check if router-flood is running
if pgrep router-flood > /dev/null; then
    echo "✅ Router-flood is running"
else
    echo "❌ Router-flood is not running"
    exit 1
fi

# Check log file size
LOG_SIZE=$(du -m /var/log/router-flood/router-flood.log | cut -f1)
if [ "$LOG_SIZE" -gt 100 ]; then
    echo "⚠️  Log file is large: ${LOG_SIZE}MB"
fi

# Check disk space
DISK_USAGE=$(df /var/log/router-flood | tail -1 | awk '{print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 80 ]; then
    echo "⚠️  Disk usage high: ${DISK_USAGE}%"
fi

echo "✅ Health check completed"
```

## Support and Resources

### Documentation
- [User Guide](USER_GUIDE.md)
- [API Reference](API_REFERENCE.md)
- [Performance Tuning](PERFORMANCE_TUNING.md)
- [Security Best Practices](SECURITY.md)

### Community
- GitHub Issues: Report bugs and feature requests
- Discussions: Ask questions and share experiences
- Wiki: Community-contributed guides and examples

### Professional Support
For enterprise deployments and professional support:
- Email: support@router-flood.org
- Documentation: https://docs.router-flood.org
- Training: Available for educational institutions

---

**Remember**: Always use router-flood responsibly and ethically. Obtain proper authorization before testing any network infrastructure.