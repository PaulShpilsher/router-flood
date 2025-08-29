# Router Flood Architecture

## Overview

Router Flood is a high-performance network testing tool built with a modular, safety-first architecture. The system is designed for educational purposes while maintaining enterprise-grade performance and security features.

## Core Design Principles

1. **Safety First**: Multiple validation layers ensure safe operation
2. **Performance**: Zero-copy operations, SIMD optimizations, lock-free data structures
3. **Modularity**: Clean separation of concerns with trait-based abstractions
4. **Testability**: Comprehensive test coverage with dependency injection
5. **Usability**: User-friendly interfaces with clear error messages

## Module Organization

### Core Modules (`src/core/`)

The core functionality is organized into focused modules:

#### Network Module (`network.rs`)
- Network interface discovery and management
- Interface validation and selection
- IP address handling

#### Target Module (`target.rs`)
- Target IP and port management
- Multi-port target support with round-robin selection
- Target validation and safety checks

#### Worker Module (`worker.rs`)
- Worker thread management
- Task distribution and coordination
- Thread pool implementation

#### Simulation Module (`simulation/`)
- **Basic Mode** (`basic.rs`): Standard simulation with configurable success rates
- **RAII Mode** (`raii.rs`): Resource-managed simulation with automatic cleanup
- Dry-run capabilities for safe testing

### Utility Modules (`src/utils/`)

Common utilities and helpers:

#### Buffer Pool (`buffer_pool.rs`)
- Memory-aligned buffer management
- Buffer reuse for reduced allocations
- Zero-copy packet construction support

#### RAII Guards (`raii.rs`)
- **WorkerGuard**: Automatic worker cleanup
- **ChannelGuard**: Channel resource management
- **SignalGuard**: Signal handler registration/deregistration
- **StatsGuard**: Statistics flushing on drop
- **ResourceGuard**: Composite guard for multiple resources

#### Random Number Generation (`rng.rs`)
- Batched random number generation
- Optimized for packet construction
- Thread-safe random value generation

#### Terminal Utilities (`terminal.rs`)
- Terminal state management
- Raw mode handling for interactive UI
- Terminal restoration on exit

### Abstractions Layer (`src/abstractions/`)

Trait-based abstractions for testability and flexibility:

#### Network Provider
```rust
pub trait NetworkProvider {
    fn interfaces(&self) -> Vec<NetworkInterface>;
    fn find_by_name(&self, name: &str) -> Option<NetworkInterface>;
    fn default_interface(&self) -> Option<NetworkInterface>;
}
```

#### System Provider
```rust
pub trait SystemProvider {
    fn is_root(&self) -> bool;
    fn effective_uid(&self) -> u32;
    fn is_tty(&self) -> bool;
    fn cpu_count(&self) -> usize;
}
```

### Statistics System (`src/stats/`)

High-performance statistics collection:

#### Lock-Free Statistics (`lockfree.rs`)
- Atomic operations for thread-safe updates
- Per-CPU statistics for cache locality
- 2x performance improvement over mutex-based approach
- Protocol-specific counters using array indexing

```rust
pub struct LockFreeStats {
    pub packets_sent: AtomicU64,
    pub packets_failed: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub protocol_counters: [AtomicU64; ProtocolId::COUNT],
    pub start_time: Instant,
}
```

#### Backward Compatibility (`adapter.rs`)
- Adapter pattern for existing FloodStats interface
- Seamless migration from mutex-based to lock-free stats
- Protocol name to ID conversion

### Performance Optimizations (`src/performance/`)

#### SIMD Packet Building (`simd_packet.rs`)
- AVX2/SSE4.2/NEON acceleration
- 2-4x performance improvement
- Automatic fallback to scalar code

#### CPU Affinity (`cpu_affinity.rs`)
- NUMA-aware worker placement
- Optimal cache utilization
- Reduced cross-CPU communication

#### Advanced Buffer Pool (`advanced_buffer_pool.rs`)
- Memory alignment for SIMD operations
- Size classes for different packet sizes
- 60-80% reduction in allocations

### Security Features (`src/security/`)

#### Capability Management (`capability.rs`)
- Linux capabilities detection
- CAP_NET_RAW validation
- Privilege escalation prevention

#### Audit Logging (`audit.rs`)
- Tamper-proof hash chains
- Cryptographic integrity protection
- Comprehensive activity logging

### Packet Construction (`src/packet/`)

Multi-protocol packet building with zero-copy optimization:

- UDP packet construction
- TCP SYN/ACK packets
- ICMP echo requests
- IPv6 support
- ARP packets
- Layer 2 (Ethernet) frames

## Data Flow

```
User Input → CLI Parser → Configuration Validation
                ↓
        Target Validation (Private IP only)
                ↓
        Worker Thread Creation
                ↓
    ┌───────────┴───────────┐
    ↓                       ↓
Packet Builder         Statistics Collection
    ↓                       ↓
Buffer Pool            Lock-Free Counters
    ↓                       ↓
Network Transport      Per-CPU Aggregation
    ↓                       ↓
Raw Socket Send        Real-time Display
```

## Performance Characteristics

### Lock-Free Statistics
- **Increment Operation**: ~18ns (2x faster than mutex)
- **Batched Updates**: ~1.9ns (11x faster)
- **Per-CPU Aggregation**: Eliminates contention

### RAII Guards
- **Zero Overhead**: Same performance as manual cleanup
- **Automatic Resource Management**: No leaks
- **Composable**: Nested guard support

### Packet Building
- **Zero-Copy UDP**: ~560ns per packet
- **TCP SYN**: ~59ns per packet
- **Buffer Pool**: 60-80% allocation reduction

### Abstraction Layer
- **Zero Overhead**: Trait abstractions compile away
- **System Calls**: ~143ns (same as direct calls)

## Testing Strategy

### Unit Tests
- 315+ tests across all modules
- Property-based testing with proptest
- Comprehensive edge case coverage

### Integration Tests
- End-to-end scenarios
- Multi-threaded stress tests
- Resource leak detection

### Benchmarks
- Criterion.rs for statistical analysis
- Regression detection
- Performance tracking

### Test Organization
```
tests/
├── common/              # Shared test utilities
├── lockfree_stats_tests.rs
├── raii_tests.rs
├── abstractions_tests.rs
├── core_tests.rs
└── stats_adapter_tests.rs

benches/
├── packet_building.rs   # Packet construction performance
├── config_validation.rs # Configuration validation speed
├── lockfree_stats.rs    # Statistics performance
├── raii_guards.rs       # RAII overhead measurement
└── abstractions.rs      # Abstraction layer overhead
```

## Build Configuration

### Release Optimizations
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### Benchmark Configuration
```toml
[profile.bench]
inherits = "release"
debug = false
```

## Future Enhancements

### Planned Improvements
1. **WebAssembly Support**: Browser-based network analysis
2. **eBPF Integration**: Kernel-level packet filtering
3. **Distributed Testing**: Multi-node coordination
4. **Machine Learning**: Anomaly detection in results
5. **GUI Frontend**: Native desktop application

### Performance Goals
- 1M PPS per thread
- Sub-microsecond latency
- Zero-allocation steady state
- Linear scaling to 128 cores

## Contributing

When contributing to Router Flood architecture:

1. Maintain the modular structure
2. Add tests for new functionality
3. Update benchmarks for performance changes
4. Follow RAII patterns for resource management
5. Use lock-free structures where appropriate
6. Document architectural decisions