# Repository Tour

## 🎯 What This Repository Does

Router Flood is an advanced network stress testing tool designed for educational purposes and authorized network testing scenarios. It combines cutting-edge performance optimizations with enterprise-grade security features while maintaining a safety-first approach.

**Key responsibilities:**
- Generate high-performance network traffic for stress testing private networks
- Provide comprehensive monitoring and metrics collection with Prometheus integration
- Ensure safety through private IP validation, capability-based security, and audit logging

---

## 🏗️ Architecture Overview

### System Context
```
[User/CLI] → [Router Flood] → [Private Network Targets]
                    ↓
            [Prometheus Metrics]
                    ↓
            [Export Files (JSON/CSV)]
```

### Key Components
- **CLI System** - Enhanced command-line interface with interactive mode and subcommands
- **Configuration Engine** - Trait-based system with builder pattern and YAML templates
- **Core Simulation** - Worker thread management with CPU affinity and NUMA awareness
- **Packet Strategies** - Protocol-specific builders using Strategy pattern (UDP, TCP, ICMP, IPv6, ARP)
- **Transport Layer** - Raw socket abstraction with mock support for testing
- **Statistics System** - Lock-free atomic counters with real-time monitoring
- **Security Framework** - Capability-based validation and tamper-proof audit logging
- **Performance Engine** - SIMD optimizations, buffer pools, and zero-copy operations

### Data Flow
1. User configures via CLI arguments or YAML configuration files
2. Configuration validated for safety (private IP ranges, rate limits, capabilities)
3. Network interface setup with Linux capability checks (CAP_NET_RAW)
4. Worker threads spawn with optimal CPU affinity assignments
5. Packet strategies generate protocol-specific packets using zero-copy techniques
6. Transport layer sends packets to targets (or simulates in dry-run mode)
7. Lock-free statistics collection with real-time display and export

---

## 📁 Project Structure [Partial Directory Tree]

```
router-flood/
├── src/                           # Main application source code
│   ├── abstractions/              # Trait-based abstractions for testability
│   ├── cli/                       # Enhanced CLI with interactive mode
│   │   ├── parser.rs              # Command structure and argument definitions
│   │   ├── commands.rs            # Command execution logic
│   │   ├── interactive.rs         # Interactive configuration mode
│   │   └── prompts.rs             # User input utilities
│   ├── config/                    # Configuration management system
│   │   ├── traits.rs              # Interface-segregated configuration traits
│   │   ├── builder.rs             # Fluent builder API
│   │   ├── schema.rs              # Configuration templates and validation
│   │   └── validation.rs          # Centralized validation logic
│   ├── core/                      # Core engine components
│   │   ├── simulation/            # Simulation orchestration and RAII guards
│   │   ├── worker.rs              # Worker thread management
│   │   ├── target.rs              # Multi-port target handling
│   │   └── network.rs             # Network interface management
│   ├── packet/                    # Multi-protocol packet construction
│   │   ├── strategies/            # Protocol-specific implementations
│   │   ├── builder.rs             # Zero-copy packet building
│   │   ├── chain.rs               # Chain of Responsibility pattern
│   │   ├── decorator.rs           # Decorator pattern for packet modification
│   │   └── plugin.rs              # Plugin system for extensibility
│   ├── performance/               # Performance optimizations
│   │   ├── simd_packet.rs         # SIMD-optimized packet building
│   │   ├── cpu_affinity.rs        # NUMA-aware CPU assignment
│   │   └── buffer_pool.rs         # Lock-free buffer management
│   ├── security/                  # Security and safety features
│   │   ├── capabilities.rs        # Linux capability management
│   │   └── audit.rs               # Tamper-proof audit logging
│   ├── stats/                     # Statistics collection and export
│   │   ├── lockfree.rs            # Lock-free atomic statistics
│   │   ├── observer.rs            # Observer pattern for events
│   │   └── export.rs              # JSON/CSV export functionality
│   ├── monitoring/                # Advanced monitoring system
│   │   ├── prometheus.rs          # Prometheus metrics export
│   │   ├── dashboard.rs           # Real-time performance dashboard
│   │   └── alerts.rs              # Alert management system
│   └── transport/                 # Network transport abstraction
│       ├── raw_socket.rs          # Raw socket implementation
│       └── mock.rs                # Mock transport for testing
├── tests/                         # Comprehensive test suite (320+ tests)
│   ├── integration_tests.rs       # End-to-end integration tests
│   ├── property_tests.rs          # Property-based testing
│   └── security_tests.rs          # Security validation tests
├── benches/                       # Performance benchmarks (15 suites)
│   ├── packet_building.rs         # Packet construction benchmarks
│   ├── lockfree_stats.rs          # Statistics performance tests
│   └── simd.rs                    # SIMD optimization benchmarks
├── fuzz/                          # Fuzzing targets for security testing
│   └── fuzz_targets/              # 3 fuzz targets for robustness
├── examples/                      # Usage examples and demos
│   ├── basic_usage.rs             # Basic functionality demonstration
│   └── config_usage.rs            # Configuration system examples
└── docs/                          # Comprehensive documentation
    └── architecture/              # Architecture documentation
```

### Key Files to Know

| File | Purpose | When You'd Touch It |
|------|---------|---------------------|
| `src/main.rs` | Application entry point with async runtime | Adding new CLI commands |
| `src/lib.rs` | Library exports and module organization | Adding new public APIs |
| `Cargo.toml` | Dependencies and build configuration | Adding new libraries |
| `src/config/mod.rs` | Configuration system with YAML support | Modifying configuration schema |
| `src/core/simulation/mod.rs` | Main simulation orchestration | Changing core execution logic |
| `src/packet/strategies/` | Protocol-specific packet builders | Adding new network protocols |
| `src/performance/simd_packet.rs` | SIMD-optimized packet construction | Performance optimizations |
| `router_flood_config.yaml` | Default configuration template | Changing default settings |
| `ARCHITECTURE.md` | Detailed system architecture | Understanding design patterns |
| `README.md` | Comprehensive usage guide | Learning tool capabilities |

---

## 🔧 Technology Stack

### Core Technologies
- **Language:** Rust 2021 Edition (1.70+) - Memory safety and zero-cost abstractions
- **Framework:** Tokio Async Runtime - High-performance concurrent execution
- **Networking:** pnet Library - Raw packet manipulation and network interfaces
- **CLI:** clap v4 with derive features - Professional command-line interface

### Key Libraries
- **pnet** - Raw socket operations and packet construction
- **tokio** - Async runtime with signal handling and futures
- **clap** - Command-line argument parsing with subcommands
- **serde + serde_yaml** - Configuration serialization and YAML support
- **tracing + tracing-subscriber** - Structured logging and observability
- **chrono** - Timestamp handling and duration management
- **csv** - CSV export functionality for statistics
- **uuid** - Session ID generation for audit trails
- **sha2** - Cryptographic hashing for audit logging
- **sysinfo** - System resource monitoring

### Development Tools
- **criterion** - Performance benchmarking with HTML reports (15 benchmark suites)
- **proptest** - Property-based testing with 10,000+ generated test cases
- **cargo-fuzz** - Fuzzing support with 3 security-focused fuzz targets
- **tempfile** - Temporary file handling for tests

---

## 🌐 External Dependencies

### Required Services
- **Linux System** - Required for raw socket capabilities and Linux-specific features
- **Network Interface** - Physical or virtual network interface for packet transmission
- **Private Network** - RFC 1918 private IP ranges for safe testing (192.168.x.x, 10.x.x.x, 172.16-31.x.x)

### Optional Integrations
- **Prometheus** - Metrics collection and monitoring integration
- **System Monitoring** - CPU, memory, and network usage tracking with sysinfo

### Environment Variables

```bash
# Optional
RUST_LOG=                  # Logging verbosity (default: info)
CAP_NET_RAW=              # Linux capability for raw socket access
PROMETHEUS_PORT=          # Port for Prometheus metrics export
```

---

## 🔄 Common Workflows

### Network Stress Testing Workflow
1. User configures target (private IP), ports, and test parameters via CLI or YAML
2. System validates configuration for safety (private IP ranges, rate limits)
3. Linux capabilities checked (CAP_NET_RAW) or dry-run mode enabled
4. Worker threads spawn with optimal CPU affinity for NUMA systems
5. Packet strategies generate protocol-specific traffic (UDP, TCP, ICMP, IPv6, ARP)
6. Real-time statistics collection with lock-free atomic counters
7. Export results to JSON/CSV and optionally to Prometheus metrics

**Code path:** `main.rs` → `simulation.rs` → `worker.rs` → `packet/strategies` → `transport/raw_socket.rs`

### Configuration Management Workflow
1. Load configuration from YAML file or use CLI arguments
2. Apply configuration templates (basic, web_server, dns_server, high_performance)
3. Validate configuration using trait-based validation system
4. Build final configuration using fluent builder pattern
5. Export validated configuration for reuse

**Code path:** `config/mod.rs` → `config/builder.rs` → `config/validation.rs` → `config/schema.rs`

---

## 📈 Performance & Scale

### Performance Considerations
- **SIMD Optimizations:** AVX2, SSE4.2, NEON instruction sets for 2-4x packet generation speedup
- **Zero-Copy Operations:** Direct buffer writing without memory allocations
- **Lock-Free Statistics:** Atomic operations with per-CPU counters for minimal contention
- **CPU Affinity:** NUMA-aware worker placement for optimal memory access
- **Buffer Pools:** Pre-allocated, reusable buffers with 60-80% allocation reduction

### Monitoring
- **Metrics:** Real-time packet rates, success rates, protocol breakdown, system resources
- **Prometheus Integration:** Production-ready metrics export on configurable port
- **Alerts:** Configurable thresholds for performance and error rate monitoring
- **Export Formats:** JSON and CSV export with system statistics inclusion

---

## 🚨 Things to Be Careful About

### 🔒 Security Considerations
- **Private IP Only:** Hard-coded validation ensures only RFC 1918 private ranges are targeted
- **Capability-Based Security:** Uses Linux CAP_NET_RAW instead of requiring root access
- **Audit Logging:** Tamper-proof cryptographic audit trails with SHA2 hashing
- **Rate Limiting:** Built-in safety limits prevent system overwhelm (max 10,000 PPS, 100 threads)
- **Dry-Run Mode:** Safe testing without actual packet transmission (98% simulated success rate)

### ⚠️ Safety Features
- **Target Validation:** Automatic rejection of public IPs, loopback, multicast, and broadcast addresses
- **Resource Limits:** Configurable maximum thread count and packet rate limits
- **Graceful Shutdown:** Signal handling for clean resource cleanup
- **Perfect Simulation:** Optional 100% success rate mode for configuration validation

### 🎯 Educational Focus
- **Authorized Testing Only:** Tool designed exclusively for educational purposes and authorized network testing
- **Documentation:** Comprehensive guides and examples for learning network concepts
- **Safety First:** Multiple validation layers ensure responsible usage

*Updated at: 2025-08-27 UTC*