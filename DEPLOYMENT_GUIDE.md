# 🚀 Router Flood Deployment Guide

This guide provides comprehensive instructions for deploying Router Flood in various environments, from development to production.

## 📋 Table of Contents

- [System Requirements](#system-requirements)
- [Installation Methods](#installation-methods)
- [Environment Setup](#environment-setup)
- [Configuration Management](#configuration-management)
- [Security Hardening](#security-hardening)
- [Monitoring Setup](#monitoring-setup)
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)

## 💻 System Requirements

### Minimum Requirements

| Component | Requirement | Notes |
|-----------|-------------|-------|
| **OS** | Linux (kernel 3.10+) | Ubuntu 20.04+, CentOS 8+, RHEL 8+ |
| **CPU** | 2 cores, 2.0 GHz | x86_64 or ARM64 |
| **Memory** | 4 GB RAM | 8 GB recommended |
| **Storage** | 1 GB free space | For binaries and logs |
| **Network** | 1 Gbps interface | For high-performance testing |

### Recommended Requirements

| Component | Requirement | Notes |
|-----------|-------------|-------|
| **OS** | Linux (kernel 5.0+) | Latest Ubuntu LTS or RHEL |
| **CPU** | 8+ cores, 3.0+ GHz | With SIMD support (AVX2/NEON) |
| **Memory** | 16+ GB RAM | For high-throughput scenarios |
| **Storage** | 10+ GB SSD | For logs and metrics |
| **Network** | 10+ Gbps interface | For maximum performance |

### Software Dependencies

```bash
# Required packages (Ubuntu/Debian)
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git

# Required packages (RHEL/CentOS)
sudo yum groupinstall -y "Development Tools"
sudo yum install -y \
    openssl-devel \
    curl \
    git
```

## 📦 Installation Methods

### Method 1: From Source (Recommended)

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone repository
git clone https://github.com/PaulShpilsher/router-flood.git
cd router-flood

# Build release version
cargo build --release

# Set capabilities
sudo setcap cap_net_raw+ep ./target/release/router-flood

# Verify installation
./target/release/router-flood --version
```

### Method 2: Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/router-flood /usr/local/bin/
COPY --from=builder /app/router_flood_config.yaml /etc/router-flood/

# Set capabilities in container
RUN setcap cap_net_raw+ep /usr/local/bin/router-flood

EXPOSE 9090
ENTRYPOINT ["router-flood"]
```

```bash
# Build and run Docker container
docker build -t router-flood:latest .
docker run --cap-add=NET_RAW -p 9090:9090 router-flood:latest
```

### Method 3: Package Installation

```bash
# Create DEB package
cargo install cargo-deb
cargo deb

# Install package
sudo dpkg -i target/debian/router-flood_*.deb

# Create RPM package
cargo install cargo-rpm
cargo rpm build

# Install package
sudo rpm -i target/rpm/router-flood-*.rpm
```

## 🔧 Environment Setup

### Development Environment

```bash
# Setup development environment
git clone https://github.com/PaulShpilsher/router-flood.git
cd router-flood

# Install development dependencies
cargo install cargo-fuzz
cargo install criterion
cargo install cargo-audit

# Setup pre-commit hooks
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --check

# Lint check
cargo clippy -- -D warnings

# Test check
cargo test --all-features

echo "All checks passed!"
EOF

chmod +x .git/hooks/pre-commit
```

### Testing Environment

```bash
# Create testing configuration
mkdir -p /etc/router-flood/testing
cat > /etc/router-flood/testing/test.yaml << 'EOF'
target:
  ip: "192.168.1.100"
  ports: [80, 443]

attack:
  threads: 4
  packet_rate: 1000
  duration: 60
  packet_size_range: [64, 1400]

safety:
  dry_run: true  # Safe for testing
  require_private_ranges: true

monitoring:
  system_monitoring: true
  stats_interval: 5

export:
  enabled: true
  format: "json"
EOF

# Test configuration
router-flood config validate /etc/router-flood/testing/test.yaml
```

### Staging Environment

```bash
# Create staging configuration
mkdir -p /etc/router-flood/staging
cat > /etc/router-flood/staging/staging.yaml << 'EOF'
target:
  ip: "192.168.10.100"
  ports: [80, 443, 8080]

attack:
  threads: 8
  packet_rate: 2000
  duration: 300
  packet_size_range: [64, 1400]

safety:
  dry_run: false
  require_private_ranges: true
  max_threads: 50
  max_packet_rate: 5000

performance:
  cpu_affinity_enabled: true
  simd_enabled: true

monitoring:
  system_monitoring: true
  stats_interval: 1
  prometheus_port: 9090

export:
  enabled: true
  format: "both"
  include_system_stats: true
EOF
```

## ⚙️ Configuration Management

### Configuration Structure

```
/etc/router-flood/
├── config/
│   ├── production.yaml
│   ├── staging.yaml
│   ├── testing.yaml
│   └── templates/
│       ├── web_server.yaml
│       ├── dns_server.yaml
│       └── high_performance.yaml
├── security/
│   ├── capabilities.conf
│   └── audit.conf
└── monitoring/
    ├── prometheus.yaml
    └── grafana/
        └── dashboards/
```

### Environment-Specific Configurations

#### Production Configuration

```yaml
# /etc/router-flood/config/production.yaml
target:
  ip: "${TARGET_IP}"
  ports: [80, 443, 8080, 8443]

attack:
  threads: 16
  packet_rate: 5000
  duration: 3600
  packet_size_range: [64, 1500]

safety:
  dry_run: false
  require_private_ranges: true
  max_threads: 100
  max_packet_rate: 10000

performance:
  cpu_affinity_enabled: true
  numa_aware: true
  simd_enabled: true
  buffer_pool_size: 50000

security:
  audit_logging: true
  audit_file: "/var/log/router-flood/audit.log"
  integrity_checking: true

monitoring:
  system_monitoring: true
  stats_interval: 1
  prometheus_port: 9090
  performance_tracking: true

export:
  enabled: true
  format: "prometheus"
  export_interval: 30
  include_system_stats: true
```

#### Configuration Templates

```bash
# Generate configuration from template
router-flood config generate \
  --template production \
  --target-ip 192.168.1.100 \
  --threads 16 \
  --output /etc/router-flood/config/production.yaml

# Validate configuration
router-flood config validate /etc/router-flood/config/production.yaml
```

### Environment Variables

```bash
# Set environment variables
export ROUTER_FLOOD_CONFIG="/etc/router-flood/config/production.yaml"
export ROUTER_FLOOD_LOG_LEVEL="info"
export ROUTER_FLOOD_AUDIT_DIR="/var/log/router-flood"
export ROUTER_FLOOD_METRICS_PORT="9090"

# Use in systemd service
cat > /etc/systemd/system/router-flood.service << 'EOF'
[Unit]
Description=Router Flood Network Testing Tool
After=network.target

[Service]
Type=simple
User=router-flood
Group=router-flood
Environment=ROUTER_FLOOD_CONFIG=/etc/router-flood/config/production.yaml
Environment=ROUTER_FLOOD_LOG_LEVEL=info
ExecStart=/usr/local/bin/router-flood run --config ${ROUTER_FLOOD_CONFIG}
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
```

## 🛡️ Security Hardening

### User and Group Setup

```bash
# Create dedicated user and group
sudo groupadd router-flood
sudo useradd -r -g router-flood -s /bin/false router-flood

# Set up directories
sudo mkdir -p /var/log/router-flood
sudo mkdir -p /var/lib/router-flood
sudo chown router-flood:router-flood /var/log/router-flood
sudo chown router-flood:router-flood /var/lib/router-flood
```

### Capability Configuration

```bash
# Set minimal required capabilities
sudo setcap cap_net_raw+ep /usr/local/bin/router-flood

# Verify capabilities
getcap /usr/local/bin/router-flood
# Output: /usr/local/bin/router-flood = cap_net_raw+ep

# Create capability configuration
cat > /etc/router-flood/security/capabilities.conf << 'EOF'
# Required capabilities for Router Flood
cap_net_raw = required

# Optional capabilities (not used)
cap_net_admin = disabled
cap_sys_admin = disabled
EOF
```

### Firewall Configuration

```bash
# Configure iptables for monitoring port
sudo iptables -A INPUT -p tcp --dport 9090 -s 192.168.0.0/16 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 9090 -j DROP

# Save iptables rules
sudo iptables-save > /etc/iptables/rules.v4

# Configure UFW (Ubuntu)
sudo ufw allow from 192.168.0.0/16 to any port 9090
sudo ufw deny 9090
```

### SELinux Configuration (RHEL/CentOS)

```bash
# Create SELinux policy for Router Flood
cat > router-flood.te << 'EOF'
module router-flood 1.0;

require {
    type unconfined_t;
    type unreserved_port_t;
    class tcp_socket name_bind;
    class rawip_socket { create bind };
}

# Allow raw socket creation
allow unconfined_t self:rawip_socket { create bind };

# Allow binding to monitoring port
allow unconfined_t unreserved_port_t:tcp_socket name_bind;
EOF

# Compile and install policy
checkmodule -M -m -o router-flood.mod router-flood.te
semodule_package -o router-flood.pp -m router-flood.mod
sudo semodule -i router-flood.pp
```

## 📊 Monitoring Setup

### Prometheus Configuration

```yaml
# /etc/router-flood/monitoring/prometheus.yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'router-flood'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
    metrics_path: /metrics
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Router Flood Performance",
    "panels": [
      {
        "title": "Packet Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "router_flood_packets_per_second",
            "legendFormat": "PPS"
          }
        ]
      },
      {
        "title": "Success Rate",
        "type": "singlestat",
        "targets": [
          {
            "expr": "router_flood_success_rate_percent",
            "legendFormat": "Success %"
          }
        ]
      },
      {
        "title": "System Resources",
        "type": "graph",
        "targets": [
          {
            "expr": "router_flood_cpu_usage_percent",
            "legendFormat": "CPU %"
          },
          {
            "expr": "router_flood_memory_usage_bytes / 1024 / 1024",
            "legendFormat": "Memory MB"
          }
        ]
      }
    ]
  }
}
```

### Log Management

```bash
# Configure log rotation
cat > /etc/logrotate.d/router-flood << 'EOF'
/var/log/router-flood/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 router-flood router-flood
    postrotate
        systemctl reload router-flood
    endscript
}
EOF

# Configure rsyslog
cat > /etc/rsyslog.d/router-flood.conf << 'EOF'
# Router Flood logging
if $programname == 'router-flood' then /var/log/router-flood/router-flood.log
& stop
EOF

sudo systemctl restart rsyslog
```

## 🏭 Production Deployment

### Systemd Service

```bash
# Create systemd service
cat > /etc/systemd/system/router-flood.service << 'EOF'
[Unit]
Description=Router Flood Network Testing Tool
Documentation=https://github.com/PaulShpilsher/router-flood
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=router-flood
Group=router-flood
WorkingDirectory=/var/lib/router-flood

# Environment
Environment=RUST_LOG=info
Environment=ROUTER_FLOOD_CONFIG=/etc/router-flood/config/production.yaml

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/router-flood /var/lib/router-flood

# Capabilities
AmbientCapabilities=CAP_NET_RAW
CapabilityBoundingSet=CAP_NET_RAW

# Execution
ExecStart=/usr/local/bin/router-flood run --config ${ROUTER_FLOOD_CONFIG}
ExecReload=/bin/kill -HUP $MAINPID

# Restart policy
Restart=always
RestartSec=10
StartLimitInterval=60
StartLimitBurst=3

# Resource limits
LimitNOFILE=65536
LimitNPROC=32768

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable router-flood
sudo systemctl start router-flood

# Check status
sudo systemctl status router-flood
```

### Health Checks

```bash
# Create health check script
cat > /usr/local/bin/router-flood-health << 'EOF'
#!/bin/bash

# Check if service is running
if ! systemctl is-active --quiet router-flood; then
    echo "CRITICAL: Router Flood service is not running"
    exit 2
fi

# Check metrics endpoint
if ! curl -s http://localhost:9090/metrics > /dev/null; then
    echo "WARNING: Metrics endpoint not responding"
    exit 1
fi

# Check log for errors
if grep -q "ERROR" /var/log/router-flood/router-flood.log; then
    echo "WARNING: Errors found in log file"
    exit 1
fi

echo "OK: Router Flood is healthy"
exit 0
EOF

chmod +x /usr/local/bin/router-flood-health

# Add to cron for monitoring
echo "*/5 * * * * /usr/local/bin/router-flood-health" | sudo crontab -
```

### Load Balancer Configuration

```nginx
# Nginx configuration for multiple instances
upstream router_flood_metrics {
    server 192.168.1.10:9090;
    server 192.168.1.11:9090;
    server 192.168.1.12:9090;
}

server {
    listen 80;
    server_name router-flood-metrics.example.com;

    location /metrics {
        proxy_pass http://router_flood_metrics;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
```

### Backup and Recovery

```bash
# Create backup script
cat > /usr/local/bin/router-flood-backup << 'EOF'
#!/bin/bash

BACKUP_DIR="/backup/router-flood"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR/$DATE"

# Backup configuration
cp -r /etc/router-flood "$BACKUP_DIR/$DATE/"

# Backup logs
cp -r /var/log/router-flood "$BACKUP_DIR/$DATE/"

# Backup audit logs
cp -r /var/lib/router-flood "$BACKUP_DIR/$DATE/"

# Create archive
tar -czf "$BACKUP_DIR/router-flood-$DATE.tar.gz" -C "$BACKUP_DIR" "$DATE"
rm -rf "$BACKUP_DIR/$DATE"

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete

echo "Backup completed: router-flood-$DATE.tar.gz"
EOF

chmod +x /usr/local/bin/router-flood-backup

# Schedule daily backups
echo "0 2 * * * /usr/local/bin/router-flood-backup" | sudo crontab -
```

## 🔧 Troubleshooting

### Common Issues

#### Permission Denied

```bash
# Check capabilities
getcap /usr/local/bin/router-flood

# Fix capabilities
sudo setcap cap_net_raw+ep /usr/local/bin/router-flood

# Check user permissions
sudo -u router-flood /usr/local/bin/router-flood system security
```

#### Service Won't Start

```bash
# Check service status
sudo systemctl status router-flood

# Check logs
sudo journalctl -u router-flood -f

# Validate configuration
router-flood config validate /etc/router-flood/config/production.yaml

# Test manually
sudo -u router-flood /usr/local/bin/router-flood run --config /etc/router-flood/config/production.yaml --dry-run
```

#### Performance Issues

```bash
# Check system resources
router-flood system performance --diagnosis

# Monitor in real-time
router-flood run --config /etc/router-flood/config/production.yaml --performance-monitor

# Check CPU affinity
router-flood system performance --cpu-analysis
```

### Diagnostic Commands

```bash
# System information
router-flood system info

# Security analysis
router-flood system security

# Performance analysis
router-flood system performance --workers 8

# Configuration validation
router-flood config validate /etc/router-flood/config/production.yaml

# Network interface check
router-flood system interfaces
```

### Log Analysis

```bash
# Check for errors
grep -i error /var/log/router-flood/router-flood.log

# Monitor performance
grep -i "packets sent" /var/log/router-flood/router-flood.log | tail -10

# Audit log verification
router-flood audit verify /var/log/router-flood/audit.log
```

---

**Note**: This deployment guide covers common scenarios. Adjust configurations based on your specific environment and requirements.