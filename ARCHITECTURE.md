# Router Flood Architecture

## Overview

Router Flood is a high-performance network stress testing tool with a simplified architecture following the KISS principle.
## Core Design Principles

1. **KISS Principle**: Keep It Simple, Stupid - avoid over-engineering
2. **Separation of Concerns**: Clear module boundaries
3. **Performance-Critical Optimizations**: SIMD, lock-free pools, CPU affinity
4. **Zero-Copy Operations**: Buffer reuse and in-place construction
5. **Safety First**: Private IP validation, rate limiting

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Main Entry                           │
│                      src/main.rs                             │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                    Configuration Layer                       │
│                     src/config/                              │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐                     │
│  │ mod.rs  │ │builder.rs│ │validator.rs│                    │
│  └─────────┘ └──────────┘ └──────────┘                     │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                      Network Layer                           │
│                     src/network/                             │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │worker.rs│ │target.rs │ │flood.rs  │ │simulation.rs │   │
│  └─────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                    Packet Generation                         │
│                      src/packet/                             │
│  ┌─────────┐ ┌──────────┐                                   │
│  │builder.rs│ │types.rs  │                                   │
│  └─────────┘ └──────────┘                                   │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                   Protocol Layer                             │
│                    src/protocols/                            │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                  │
│  │ipv4 │ │ipv6 │ │tcp  │ │udp  │ │icmp │                  │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘                  │
└─────────────────────────────────────────────────────────────┘
```

## Module Organization (51 files, ~6,700 LOC)

### Core Modules

#### Configuration (`src/config/`) - 3 files
- **mod.rs**: Core configuration structures and types
- **builder.rs**: Configuration builder with validation
- **validator.rs**: Input validation and safety checks

#### Network (`src/network/`) - 4 files
- **worker.rs**: Packet generation worker with rate limiting
- **target.rs**: Multi-port target management
- **flood.rs**: Main flood orchestration logic
- **simulation.rs**: Dry-run simulation mode

#### Packet (`src/packet/`) - 2 files
- **builder.rs**: Packet construction with zero-copy optimizations
- **types.rs**: Packet type definitions and enums

#### Protocols (`src/protocols/`) - 5 files
- **ipv4.rs**: IPv4 packet construction
- **ipv6.rs**: IPv6 packet construction
- **tcp.rs**: TCP segment generation
- **udp.rs**: UDP datagram generation
- **icmp.rs**: ICMP packet generationbd13968c8687322ecd896ab613531c2b2b47f865

#### Statistics (`src/stats/`) - 3 files
- **stats_aggregator.rs**: Atomic statistics with batching
- **collector.rs**: Statistics collection traits
- **export.rs**: JSON/CSV export functionality

#### Performance (`src/performance/`) - 3 files
- **simd.rs**: SIMD-optimized payload generation (AVX2/SSE4.2)
- **memory_pool.rs**: Lock-free buffer pool (Treiber stack)
- **cpu_affinity.rs**: NUMA-aware CPU pinning

#### Utils (`src/utils/`) - 3 files
- **rng.rs**: Batched random number generation
- **raii.rs**: RAII guards for resource management
- **validation.rs**: Common validation utilities

#### Error Handling (`src/error.rs`) - 1 file
- Consolidated error types for the entire application

### Key Components

## Worker Architecture

The Worker is the core packet generation component:

```rust
pub struct Worker {
    local_stats: BatchStats,      // Local statistics batching
    target: Arc<MultiPortTarget>,  // Shared target configuration
    packet_builder: PacketBuilder, // Zero-copy packet construction
    buffer: Vec<u8>,              // Pre-allocated buffer
    packet_types: Vec<PacketType>, // Pre-calculated distribution
    base_delay: Duration,         // Rate limiting delay
}
```

### Processing Flow:
1. **Single Packet Processing**: Each worker processes one packet at a time
2. **Rate Limiting**: Applies configurable delay between packets
3. **Local Batching**: Accumulates stats locally before atomic flush
4. **Zero-Copy**: Reuses pre-allocated buffers

## Statistics System

Simplified atomic-based statistics:

```rust
pub struct Stats {
    packets_sent: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    packets_failed: Arc<AtomicU64>,
}

pub struct BatchStats {
    stats: Arc<Stats>,  // Reference to global stats
    // Local accumulation
    packets_sent: u64,
    bytes_sent: u64,
    packets_failed: u64,
    batch_size: u64,    // Flush threshold (default: 50)
}
```

### Batching Strategy:
- Workers accumulate stats locally
- Flush to global atomics every 50 packets
- Reduces atomic contention by 50x

## Performance Optimizations

### SIMD Payload Generation
- **Detection**: Runtime CPU feature detection
- **Implementation**: AVX2 (256-bit) and SSE4.2 (128-bit)
- **Performance**: 3-5x faster payload generation
- **Fallback**: Standard generation for older CPUs

### Lock-Free Memory Pool
- **Algorithm**: Treiber stack (lock-free LIFO)
- **Buffer Size**: 64KB pre-allocated buffers
- **Capacity**: Configurable pool size
- **Zero Allocations**: After initial pool creation

### CPU Affinity
- **NUMA Awareness**: Pin workers to specific cores
- **Cache Locality**: Improved L1/L2 cache utilization
- **Reduced Context Switching**: 15-25% throughput gain

### Batched RNG
- **Pre-computation**: Generate random values in batches of 1000
- **Types**: Ports, sequences, TTLs, window sizes
- **Performance**: 40% reduction in RNG overhead

## Safety Features

### Validation Layers
1. **Configuration**: Builder pattern validation
2. **Runtime**: Target IP and port validation
3. **Rate Limiting**: Configurable packet rate limits
4. **Private IP Only**: RFC1918 range enforcement

### Simulation Mode
- **Dry Run**: Test configuration without sending packets
- **Success Rate**: Configurable success simulation
- **Performance Testing**: Validate configuration at scale

## Memory Management

### Buffer Strategy
- **Pre-allocation**: All buffers allocated at startup
- **Reuse**: Zero-copy buffer reuse via pools
- **RAII Guards**: Automatic buffer return to pool
- **Alignment**: SIMD-aligned allocations

### Allocation Profile
```
Startup: ~100MB for buffer pools
Runtime: Zero allocations (steady state)
Cleanup: Automatic via RAII
```

## Concurrency Model

### Thread Architecture
- **Main Thread**: CLI and orchestration
- **Worker Threads**: Packet generation (configurable count)
- **Stats Thread**: Optional statistics export

### Synchronization
- **Lock-Free**: Atomic operations for statistics
- **Arc**: Shared ownership of configuration
- **No Mutexes**: In hot paths

## Testing Strategy

### Test Organization
```
tests/
├── integration/
│   ├── basic_flood.rs
│   ├── config_validation.rs
│   └── stats_collection.rs
└── unit/
    ├── packet_builder.rs
    ├── rng.rs
    └── memory_pool.rs
```

### Coverage Areas
- Unit tests for individual components
- Integration tests for workflows
- Smoke tests for basic functionality
- Performance benchmarks

## Build and Deployment

### Build Profiles
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### Capabilities
```bash
# Required for raw socket access
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

## Future Considerations

### Potential Optimizations
- io_uring for packet transmission
- eBPF for kernel-bypass networking
- DPDK integration for 100Gbps+ rates

### Scalability
- Multi-node coordination
- Distributed statistics aggregation
- Cloud-native deployment
