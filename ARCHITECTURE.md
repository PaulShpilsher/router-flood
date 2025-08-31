# Router Flood Architecture

## Overview

Router Flood is a high-performance network testing tool with a streamlined, consolidated architecture. Following successful consolidation efforts, the system now features single, optimized implementations for each core functionality.

## Core Design Principles

1. **Single Implementation**: One optimized solution per concept
2. **Lock-Free Performance**: Atomic operations minimize contention
3. **Batch Processing**: Amortize costs across multiple operations
4. **Zero-Copy Operations**: Reuse buffers and avoid allocations
5. **Safety First**: Private IP validation, rate limiting, audit logging

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ Parser  │ │ Commands │ │Interactive│ │   Prompts    │   │
│  └─────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                    Configuration Layer                       │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ Traits  │ │ Builder  │ │Validation│ │   Schema     │   │
│  └─────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                      Core Layer                              │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ Worker  │ │ Target   │ │Simulation│ │   Network    │   │
│  └─────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                    Packet Layer                              │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │Strategy │ │  Chain   │ │Decorator │ │   Plugin     │   │
│  └─────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                   Transport Layer                            │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐                     │
│  │   Raw   │ │   Mock   │ │  Layer   │                     │
│  └─────────┘ └──────────┘ └──────────┘                     │
└─────────────────────────────────────────────────────────────┘
```

## Module Organization

### CLI Module (`src/cli/`)

Modular command-line interface following Single Responsibility Principle:

- **parser.rs**: Command structure and argument definitions
- **commands.rs**: Command execution logic
- **interactive.rs**: Interactive configuration mode
- **prompts.rs**: User input utilities
- **enhanced.rs**: Facade for backward compatibility

### Configuration Module (`src/config/`)

Interface-segregated configuration system:

- **traits.rs**: Focused configuration traits (ISP)
- **trait_impls.rs**: Trait implementations for Config structures
- **builder.rs**: Fluent builder API
- **validation.rs**: Centralized validation logic
- **schema.rs**: Configuration templates and schemas

### Packet Module (`src/packet/`)

Extensible packet building with multiple design patterns:

#### Strategy Pattern (`strategies/`)
- Protocol-specific implementations (UDP, TCP, ICMP, IPv6, ARP)
- Zero-copy packet construction
- Protocol compatibility checking

#### Chain of Responsibility (`chain.rs`)
- Composable packet processing pipeline
- Handlers: Validation, Checksum, TTL, Logging, RateLimit
- Early termination support

#### Decorator Pattern (`decorator.rs`)
- Transparent strategy enhancement
- Decorators: Fragmentation, Encryption, Compression, Jitter
- Stackable modifications

#### Plugin System (`plugin.rs`)
- Dynamic strategy registration
- Runtime plugin loading/unloading
- Isolated lifecycle management

#### Factory Pattern (`strategy_factory.rs`)
- Centralized strategy creation
- Global registry with singleton pattern
- Type-safe strategy builders

### Statistics Module (`src/stats/`)

Flexible statistics collection with Observer pattern:

- **observer.rs**: Event-driven statistics with multiple observers
- **collector.rs**: Core statistics collection traits
- **lockfree.rs**: Lock-free per-CPU statistics
- **adapter.rs**: Adapters for different stats implementations
- **export.rs**: Export functionality (JSON, CSV)

### Performance Module (`src/performance/`)

Performance optimizations and enhancements:

- **simd_packet.rs**: SIMD-optimized packet building (AVX2, SSE4.2, NEON)
- **buffer_pool.rs**: Lock-free and shared buffer pools
- **advanced_buffer_pool.rs**: Size-class based pool with alignment
- **cpu_affinity.rs**: NUMA-aware CPU affinity management
- **optimized_constants.rs**: Compile-time optimizations

### Utils Module (`src/utils/`)

Reusable utilities and abstractions:

- **pool_trait.rs**: Unified buffer pool traits
- **pool_adapters.rs**: Adapter implementations
- **buffer_pool.rs**: Basic and worker-specific pools
- **raii.rs**: RAII guards for resource management
- **terminal.rs**: Terminal control utilities
- **rng.rs**: Batched random number generation

### Security Module (`src/security/`)

Security and safety features:

- **capabilities.rs**: Linux capability management
- **validation.rs**: Input validation and sanitization
- **audit.rs**: Tamper-proof audit logging

### Transport Module (`src/transport/`)

Network transport abstraction:

- **raw_socket.rs**: Raw socket implementation
- **mock.rs**: Mock transport for testing
- **layer.rs**: Transport layer abstraction

## Design Patterns

### Creational Patterns
- **Builder**: Configuration building with validation
- **Factory**: Packet strategy creation
- **Singleton**: Global strategy registry
- **Abstract Factory**: Plugin system

### Structural Patterns
- **Adapter**: Buffer pool adapters for legacy code
- **Decorator**: Packet modification layers
- **Facade**: CLI module facade
- **Composite**: Observer composition

### Behavioral Patterns
- **Strategy**: Protocol-specific packet building
- **Observer**: Statistics event notification
- **Chain of Responsibility**: Packet processing pipeline
- **Template Method**: Base packet building flow

## Performance Architecture

### Memory Management
- **Zero-Copy**: Direct buffer writing without allocations
- **Buffer Pools**: Pre-allocated, reusable buffers
- **RAII**: Automatic resource cleanup
- **Aligned Allocations**: SIMD-optimized memory layout

### Concurrency
- **Lock-Free Statistics**: Atomic operations with per-CPU counters
- **Thread Pools**: Worker thread management
- **CPU Affinity**: NUMA-aware thread placement
- **Batch Processing**: Amortized synchronization costs

### Optimization Techniques
- **SIMD Instructions**: Vectorized packet operations
- **Compile-Time Constants**: Pre-computed lookup tables
- **Branch Prediction**: Hot-path optimization
- **Cache Locality**: Data structure alignment

## Safety Architecture

### Validation Layers
1. **Configuration Validation**: Builder pattern with comprehensive checks
2. **Runtime Validation**: Target IP and rate limiting
3. **Protocol Validation**: IPv4/IPv6 compatibility
4. **Capability Validation**: Linux capability checks

### Safety Features
- **Private IP Only**: RFC1918 range enforcement
- **Rate Limiting**: Configurable packet rate limits
- **Dry Run Mode**: Testing without packet transmission
- **Perfect Simulation**: 100% success rate for validation

## Extensibility Points

### Adding New Protocols
1. Implement `PacketStrategy` trait
2. Register with `StrategyRegistry`
3. Add to `PacketType` enum
4. Update protocol mix configuration

### Adding Statistics Observers
1. Implement `StatsObserver` trait
2. Register with `StatsSubject`
3. Handle relevant `StatsEvent` types

### Adding Packet Handlers
1. Implement `PacketHandler` trait
2. Add to `HandlerChain`
3. Define `ProcessResult` behavior

### Adding Packet Decorators
1. Extend existing strategies
2. Implement `PacketStrategy` trait
3. Chain with `DecoratorBuilder`

## Testing Architecture

### Test Organization
- **Unit Tests**: `tests/unit/` - Component isolation
- **Integration Tests**: `tests/integration/` - Component interaction
- **Property Tests**: Using proptest for invariants
- **Benchmarks**: `benches/` - Performance regression detection

### Test Coverage
- 320+ tests across all categories
- Property-based testing with 10,000+ cases
- Fuzzing support with cargo-fuzz
- Mock implementations for isolation

## Future Architecture Considerations

### Planned Enhancements
- Async/await for I/O operations
- Plugin loading from external libraries
- WebAssembly support for sandboxed plugins
- gRPC API for remote control

### Scalability
- Distributed testing coordination
- Horizontal scaling with multiple nodes
- Cloud-native deployment support
- Kubernetes operator for orchestration

## Conclusion

The Router Flood architecture prioritizes safety, performance, and extensibility through careful application of design patterns and best practices. The modular structure ensures that components can evolve independently while maintaining system cohesion through well-defined interfaces.