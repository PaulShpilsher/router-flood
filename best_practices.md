# 📘 Project Best Practices

## 1. Project Purpose

Router Flood is an educational network stress testing tool designed for authorized network testing scenarios. It combines cutting-edge performance optimizations with enterprise-grade security features while maintaining a safety-first approach. The project focuses on educational use cases, private network testing, and capability-based security rather than requiring root privileges.

## 2. Project Structure

### Core Architecture
```
src/
├── abstractions/        # Trait-based abstractions for testability and modularity
├── cli/                 # Enhanced CLI with interactive mode and subcommands
├── config/              # Configuration management with builder pattern and templates
├── core/                # Core functionality (network, simulation, target, worker)
├── error/               # Centralized error handling with user-friendly messages
├── monitoring/          # Prometheus metrics and system monitoring
├── packet/              # Multi-protocol packet construction with SIMD optimizations
├── performance/         # SIMD optimizations and CPU affinity management
├── security/            # Capability-based security and audit logging
├── stats/               # Statistics collection with lock-free atomic operations
├── transport/           # Network transport layer abstraction
├── ui/                  # Progress indicators and user interface components
├── utils/               # Utility modules (buffer pools, RAII guards, RNG, terminal)
└── validation/          # Safety validation and IP address checking
```

### Key Directories
- **`core/`**: Contains the main business logic including simulation modes, worker management, and network interfaces
- **`abstractions/`**: Provides trait-based interfaces for dependency injection and testing
- **`utils/`**: Houses performance-critical utilities like buffer pools and RAII resource management
- **`tests/`**: Comprehensive test suite with 320+ tests organized by category

## 3. Test Strategy

### Framework and Organization
- **Primary Framework**: Rust's built-in test framework with `tokio-test` for async testing
- **Property Testing**: `proptest` with 10,000+ generated test cases per property
- **Benchmarking**: `criterion` for performance regression detection
- **Fuzzing**: `cargo-fuzz` with 3 dedicated fuzz targets

### Test Structure
- **Unit Tests**: Located in dedicated files in `tests/` directory (200+ tests)
- **Integration Tests**: End-to-end scenario testing (50+ tests)
- **Property Tests**: Edge case detection with random input generation (20+ properties)
- **Security Tests**: Capability validation and audit logging verification (30+ tests)
- **Performance Tests**: Benchmark regression detection (20+ tests)

### Testing Guidelines
- All tests moved from inline `#[cfg(test)]` modules to dedicated test files
- Use `tokio::test` for async functionality testing
- Employ `proptest!` macro for property-based testing with custom generators
- Test both realistic simulation (98% success) and perfect simulation (100% success) modes
- Include security context validation in all privilege-related tests

## 4. Code Style

### Language-Specific Rules
- **Async Usage**: Prefer `tokio` runtime with `#[tokio::main]` and `#[tokio::test]`
- **Error Handling**: Use centralized `RouterFloodError` enum, avoid `unwrap()` and `expect()`
- **Memory Management**: Leverage RAII patterns with custom `ResourceGuard` types
- **Performance**: Use SIMD optimizations where available (AVX2, SSE4.2, NEON)

### Naming Conventions
- **Functions**: Snake_case with descriptive verbs (`build_packet_into_buffer`, `validate_config`)
- **Types**: PascalCase with domain-specific naming (`PacketBuilder`, `SimulationRAII`)
- **Constants**: SCREAMING_SNAKE_CASE in `constants.rs` module
- **Files**: Snake_case matching module names (`buffer_pool.rs`, `lockfree_stats.rs`)
- **Modules**: Snake_case with clear separation of concerns

### Documentation and Comments
- Use `//!` for module-level documentation with purpose and usage examples
- Document all public APIs with `///` including examples where helpful
- Include safety comments for `unsafe` code blocks (rare, only in SIMD optimizations)
- Maintain inline comments for complex algorithms and performance-critical sections

### Error Handling Patterns
- Use `Result<T>` type alias defined in `error/mod.rs`
- Implement `From` traits for error conversion between types
- Provide user-friendly error messages with actionable guidance
- Use `MapError` trait for context-specific error mapping

## 5. Common Patterns

### Design Patterns
- **Builder Pattern**: `ConfigBuilder` for fluent configuration API
- **Strategy Pattern**: Protocol-specific packet building with `PacketStrategy` trait
- **Observer Pattern**: Event-driven statistics collection with multiple observers
- **RAII Pattern**: Automatic resource cleanup with `ResourceGuard` and `TerminalGuard`
- **Plugin System**: Extensible protocol registration without core modifications

### Architectural Patterns
- **Trait-Based Abstractions**: Interface segregation with focused traits
- **Lock-Free Programming**: Atomic operations for high-performance statistics
- **Zero-Copy Operations**: Direct in-place packet construction to minimize allocations
- **CPU Affinity Management**: NUMA-aware worker placement for optimal performance

### Utility Patterns
- **Buffer Pools**: Reusable buffer management to reduce allocation overhead
- **SIMD Acceleration**: Platform-specific optimizations with graceful fallback
- **Configuration Templates**: Pre-built scenarios for common use cases
- **Dry-Run Modes**: Safe testing with realistic and perfect simulation options

## 6. Do's and Don'ts

### ✅ Do's
- **Always validate private IP ranges** before any network operations
- **Use capability-based security** instead of requiring root privileges
- **Implement comprehensive error handling** with user-friendly messages
- **Leverage RAII patterns** for automatic resource cleanup
- **Write property-based tests** for edge case detection
- **Use lock-free data structures** for performance-critical paths
- **Provide dry-run modes** for safe configuration testing
- **Include audit logging** for security and compliance
- **Optimize with SIMD** where performance is critical
- **Use builder patterns** for complex configuration objects

### ❌ Don'ts
- **Never use `unwrap()` or `expect()`** in production code paths
- **Don't allow public IP addresses** without explicit override flags
- **Avoid blocking operations** in async contexts
- **Don't ignore error handling** - always propagate or handle appropriately
- **Never skip input validation** especially for network-related parameters
- **Don't use raw pointers** unless absolutely necessary for SIMD operations
- **Avoid string allocations** in hot paths - use constants from `error::messages`
- **Don't implement `Drop` manually** when RAII guards can handle cleanup
- **Never bypass safety checks** even in performance-critical sections

## 7. Tools & Dependencies

### Core Dependencies
- **`pnet`**: Low-level network packet manipulation
- **`tokio`**: Async runtime with full feature set
- **`clap`**: CLI parsing with derive macros
- **`serde`**: Serialization with YAML and JSON support
- **`tracing`**: Structured logging and instrumentation

### Development Dependencies
- **`criterion`**: Performance benchmarking with HTML reports
- **`proptest`**: Property-based testing framework
- **`tokio-test`**: Async testing utilities
- **`tempfile`**: Temporary file management for tests

### Performance Dependencies
- **`num_cpus`**: CPU core detection for worker placement
- **`once_cell`**: Lazy static initialization for global state
- **`futures`**: Async utilities and combinators

### Security Dependencies
- **`sha2`**: Cryptographic hashing for audit log integrity
- **`uuid`**: Unique identifier generation
- **`libc`**: System capability detection

### Setup Instructions
```bash
# Install Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <repository-url>
cd router-flood
cargo build --release

# Set capabilities (recommended over root)
sudo setcap cap_net_raw+ep ./target/release/router-flood

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench
```

## 8. Other Notes

### LLM Code Generation Guidelines
- **Configuration**: Always use `ConfigBuilder` for programmatic config creation
- **Error Handling**: Wrap all fallible operations in `Result<T>` and handle appropriately
- **Testing**: Include both unit tests and property-based tests for new functionality
- **Performance**: Consider SIMD optimizations for data-intensive operations
- **Security**: Validate all network inputs and maintain audit trails
- **Async Code**: Use `tokio::spawn` for concurrent operations, avoid blocking calls

### Special Constraints
- **Private Networks Only**: Hard-coded validation prevents public IP targeting
- **Capability-Based Security**: Designed to work with `CAP_NET_RAW` instead of root
- **Educational Focus**: All features designed with learning and safety in mind
- **Cross-Platform SIMD**: Automatic detection and fallback for different architectures

### Performance Considerations
- **Lock-Free Statistics**: Use atomic operations for high-frequency updates
- **Buffer Reuse**: Leverage buffer pools to minimize allocation overhead
- **Zero-Copy Packet Building**: Direct in-place construction when possible
- **CPU Affinity**: Consider NUMA topology for multi-threaded workloads

### Extension Points
- **Protocol Plugins**: Implement `PacketStrategy` trait for new protocols
- **Export Formats**: Add new formats by extending `ExportFormat` enum
- **Monitoring Backends**: Implement observer pattern for new metrics systems
- **Configuration Sources**: Extend config loading beyond YAML files