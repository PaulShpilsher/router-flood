# Architecture

## Overview

Router Flood is a high-performance network stress testing CLI tool with a simplified, modular architecture.

## Project Structure

```
src/
├── config/          # Configuration and validation
├── network/         # Core networking (workers, flood controller)
├── packet/          # Packet generation and building
├── performance/     # CPU affinity, memory pools, SIMD
├── protocols/       # Protocol implementations (TCP, UDP, ICMP)
├── stats/           # Statistics collection and export
└── utils/           # Utilities (RNG, validation, RAII)
```

## Key Components

### Worker Threads
- Handle packet generation and sending
- Rate limiting per thread
- Local statistics batching

### Memory Pool
- Lock-free Treiber stack
- Pre-allocated buffers
- Zero allocation after init

### SIMD Operations
- AVX2/SSE4.2 payload generation
- Runtime CPU feature detection
- ~3-5x speedup

### Statistics
- Atomic counters for lock-free updates
- Batched updates to reduce contention
- JSON/CSV export support

## Performance Features

- **CPU Affinity**: Pin workers to specific cores
- **Zero-Copy**: In-place buffer operations
- **Batched RNG**: Pre-computed random values
- **Lock-Free**: Wait-free data structures where possible

## Safety Features

- Private IP validation (RFC 1918 only)
- Rate limiting
- Dry-run mode
- CAP_NET_RAW capability (no root required)