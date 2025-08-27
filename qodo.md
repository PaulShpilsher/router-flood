# Repository Tour

## 🎯 What This Repository Does

Router Flood is a high-performance educational network stress testing tool designed for controlled local network environments. It provides comprehensive multi-protocol simulation capabilities with zero-copy packet construction and advanced buffer pool optimization, delivering up to 80% better performance than traditional approaches.

**Key responsibilities:**
- Generate realistic multi-protocol network traffic (UDP, TCP, ICMP, IPv6, ARP) for router stress testing
- Provide safe educational environment for understanding network behavior under load
- Enable security researchers and network administrators to evaluate mitigation strategies

---

## 🏗️ Architecture Overview

### System Context
```
[Network Admin/Student] → [Router Flood CLI] → [Target Router (Private IP)]
                               ↓
                         [Statistics Export] → [JSON/CSV Files]
                               ↓
                         [Audit Logging] → [Session Tracking]
```

### Key Components
- **Simulation Controller** - Orchestrates the entire testing lifecycle with graceful shutdown handling
- **Worker Manager** - Manages concurrent packet generation threads with per-worker transport channels
- **Packet Builder** - Constructs realistic multi-protocol packets using zero-copy optimization and buffer pools
- **Stats Engine** - Provides real-time monitoring with JSON/CSV export capabilities
- **Validation Layer** - Ensures safe and ethical usage by restricting to private IP ranges only
- **Audit System** - Maintains comprehensive session logs with UUID-based tracking

### Data Flow
1. User configures target (private IP only) and attack parameters via CLI or YAML
2. System validates safety requirements and detects network interface
3. Worker threads spawn with dedicated transport channels to eliminate contention
4. Packets generated using configurable protocol mix ratios (UDP/TCP/ICMP/IPv6/ARP)
5. Zero-copy packet construction with buffer pool reuse for 60-80% performance improvement
6. Real-time statistics collection with batched atomic updates
7. Periodic export to JSON/CSV with system resource monitoring

---

## 📁 Project Structure [Partial Directory Tree]

```
router-flood/
├── src/                    # Main application code (14 modules)
│   ├── main.rs            # Application entry point and orchestration
│   ├── lib.rs             # Library interface and module exports
│   ├── simulation.rs      # High-level simulation orchestration
│   ├── worker.rs          # Worker thread management and packet generation
│   ├── packet.rs          # Multi-protocol packet construction with zero-copy
│   ├── config.rs          # YAML configuration management and validation
│   ├── cli.rs             # Command-line argument parsing with clap
│   ├── stats.rs           # Statistics collection and export engine
│   ├── validation.rs      # Security and safety validation layer
│   ├── audit.rs           # Audit logging and session tracking
│   ├── network.rs         # Network interface detection and management
│   ├── target.rs          # Multi-port target management and rotation
│   ├── monitor.rs         # System resource monitoring (CPU, memory)
│   ├── buffer_pool.rs     # Zero-copy buffer management for performance
│   ├── transport.rs       # Per-worker transport channel management
│   ├── rng.rs             # Batched random number generation optimization
│   ├── error.rs           # Comprehensive error handling and types
│   └── constants.rs       # Application constants and defaults
├── tests/                 # Comprehensive test suite (162 tests across 17 modules)
├── .github/workflows/     # CI/CD pipeline with automated testing
├── exports/               # Statistics export directory
├── target/                # Rust build artifacts
├── Cargo.toml            # Dependencies and project metadata
├── router_flood_config.yaml # Default YAML configuration
└── README.md             # Comprehensive documentation
```

### Key Files to Know

| File | Purpose | When You'd Touch It |
|------|---------|---------------------|
| `src/main.rs` | Application entry point | Adding new CLI commands or startup logic |
| `src/simulation.rs` | Main simulation orchestration | Modifying simulation lifecycle or monitoring |
| `src/packet.rs` | Multi-protocol packet generation | Adding new protocols or packet types |
| `src/worker.rs` | Worker thread management | Changing concurrency or rate limiting |
| `src/config.rs` | Configuration management | Adding new config options or validation |
| `router_flood_config.yaml` | Default configuration | Changing default attack parameters |
| `Cargo.toml` | Dependencies and metadata | Adding new libraries or updating versions |
| `src/validation.rs` | Safety validation | Modifying security checks or IP validation |
| `src/stats.rs` | Statistics and export | Adding new metrics or export formats |
| `tests/integration_tests.rs` | End-to-end testing | Adding new integration scenarios |

---

## 🔧 Technology Stack

### Core Technologies
- **Language:** Rust (2021 edition) - Chosen for memory safety, performance, and zero-cost abstractions
- **Framework:** Tokio (1.38.0) - Async runtime for high-performance concurrent packet generation
- **Network Library:** pnet (0.35.0) - Low-level packet crafting and raw socket access
- **CLI Framework:** clap (4.5.4) - Modern command-line argument parsing with derive macros

### Key Libraries
- **serde + serde_yaml** - YAML configuration parsing and serialization
- **tracing + tracing-subscriber** - Structured logging with configurable levels
- **uuid** - Session tracking with unique identifiers for audit trails
- **chrono** - Timestamp handling for statistics and audit logging
- **csv + serde_json** - Statistics export in multiple formats
- **sysinfo** - System resource monitoring (CPU, memory usage)

### Development Tools
- **tokio-test** - Async testing framework for concurrent code validation
- **tempfile** - Temporary file handling for test isolation
- **futures** - Additional async utilities for complex workflows
- **GitHub Actions** - CI/CD pipeline with automated testing (162 tests)

---

## 🌐 External Dependencies

### Required Services
- **Raw Socket Access** - Requires root privileges for packet injection (bypassed in dry-run mode)
- **Network Interface** - Auto-detects available interfaces or accepts manual specification
- **Private IP Targets** - Validates targets against RFC 1918 ranges (192.168.x.x, 10.x.x.x, 172.16-31.x.x)

### Optional Integrations
- **Statistics Export** - JSON/CSV export for external analysis tools
- **System Monitoring** - CPU and memory usage tracking via sysinfo crate
- **Audit Logging** - Session tracking with JSON format for compliance

### Environment Variables

```bash
# Optional
RUST_LOG=debug              # Logging verbosity (default: info)
```

---

## 🔄 Common Workflows

### Educational Network Testing
1. Configure target router IP (must be private range) and ports in YAML or CLI
2. Set protocol mix ratios (UDP/TCP/ICMP/IPv6/ARP) based on testing scenario
3. Launch simulation with specified thread count and packet rate
4. Monitor real-time statistics and system resource usage
5. Export results to JSON/CSV for analysis and reporting

**Code path:** `main.rs` → `simulation.rs` → `worker.rs` → `packet.rs` → `transport.rs`

### Safe Configuration Testing
1. Use dry-run mode to validate configuration without sending packets
2. Test different burst patterns (sustained, burst, ramp) for various scenarios
3. Verify network interface detection and target validation
4. Review audit logs for session tracking and compliance

**Code path:** `cli.rs` → `config.rs` → `validation.rs` → `simulation.rs` (dry-run mode)

---

## 📈 Performance & Scale

### Performance Optimizations
- **Zero-Copy Packet Construction**: Direct in-place packet building eliminates heap allocations (60-80% improvement)
- **Buffer Pool System**: Thread-local buffer reuse with 1.65x memory allocation improvement
- **Batched RNG**: Pre-computed random values with 4.38x payload generation speedup
- **Per-Worker Channels**: Eliminates mutex contention with 8x transport channel speedup
- **High-Resolution Timing**: Sub-millisecond rate limiting with busy-wait optimization

### Monitoring
- **Real-time Metrics**: Packets sent/failed, throughput (PPS/Mbps), protocol breakdown
- **System Resources**: CPU usage, memory consumption, network interface statistics
- **Export Capabilities**: Periodic JSON/CSV export with configurable intervals

---

## 🚨 Things to Be Careful About

### 🔒 Security Considerations
- **Private IP Validation**: Hard-coded restriction to RFC 1918 private ranges only
- **Rate Limiting**: Built-in safety limits (max 100 threads, 10,000 PPS per thread)
- **Audit Logging**: Comprehensive session tracking for accountability and forensic analysis
- **Root Privileges**: Required for raw socket access (can be bypassed with dry-run mode)

### ⚠️ Ethical Usage
- **Educational Purpose Only**: Tool designed exclusively for authorized testing and learning
- **Explicit Permission Required**: Only use on networks you own or have written permission to test
- **Legal Compliance**: Users must comply with local, national, and international laws
- **Responsible Disclosure**: Built-in safety mechanisms prevent misuse and system overwhelm

### 🛡️ Safety Features
- **Dry-Run Mode**: Safe testing without actual packet transmission
- **Graceful Shutdown**: Clean termination with Ctrl+C handling and final statistics
- **Interface Validation**: Automatic network interface detection with fallback options
- **Configuration Validation**: Comprehensive parameter validation before execution

*Last updated: 2025-08-25*
*Update to last commit: 80302e1b835dbe32045e02433f456a7372edfda0*