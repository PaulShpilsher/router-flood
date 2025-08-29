# Development Guide

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Linux system (required for raw socket support)
- Development tools: `cargo`, `git`

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/PaulShpilsher/router-flood.git
cd router-flood

# Build the project
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

## Project Structure

```
router-flood/
├── src/
│   ├── abstractions/    # Trait-based abstractions
│   ├── cli/             # Command-line interface
│   ├── config/          # Configuration management
│   ├── core/            # Core functionality
│   │   ├── network.rs   # Network interface management
│   │   ├── simulation/  # Simulation modes
│   │   ├── target.rs    # Target management
│   │   └── worker.rs    # Worker threads
│   ├── error/           # Error handling
│   ├── monitoring/      # Metrics and monitoring
│   ├── packet/          # Packet construction
│   ├── performance/     # Performance optimizations
│   ├── security/        # Security features
│   ├── stats/           # Statistics collection
│   │   ├── lockfree.rs  # Lock-free statistics
│   │   └── adapter.rs   # Backward compatibility
│   ├── transport/       # Network transport
│   ├── ui/              # User interface
│   ├── utils/           # Utility modules
│   │   ├── buffer_pool.rs
│   │   ├── raii.rs      # RAII guards
│   │   ├── rng.rs       # Random generation
│   │   └── terminal.rs  # Terminal utilities
│   └── validation/      # Input validation
├── tests/               # Test suite
├── benches/             # Performance benchmarks
├── fuzz/                # Fuzzing targets
└── examples/            # Usage examples
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test lockfree_stats

# Run tests in release mode
cargo test --release

# Run with specific features
cargo test --all-features
```

### Test Organization

Tests are organized into several categories:

#### Unit Tests (315+ tests)
Located in `tests/` directory:
- `lockfree_stats_tests.rs` - Lock-free statistics tests
- `raii_tests.rs` - RAII guard tests
- `abstractions_tests.rs` - Abstraction layer tests
- `core_tests.rs` - Core functionality tests
- `stats_adapter_tests.rs` - Statistics adapter tests

#### Common Test Utilities
`tests/common/mod.rs` provides shared test configuration:

```rust
use router_flood::tests::common::*;

let config = create_test_config();
let safe_config = create_safe_test_config();
```

### Writing Tests

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // Arrange
        let config = create_test_config();
        
        // Act
        let result = some_function(&config);
        
        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Async test implementation
    }
}
```

### Property-Based Testing

Using `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_packet_size_property(size in 64..=1500usize) {
        let packet = create_packet(size);
        prop_assert!(packet.len() == size);
    }
}
```

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench lockfree_stats

# Run specific test within benchmark
cargo bench --bench lockfree_stats "lockfree_increment"

# Quick benchmark validation
./test_bench.sh

# Full benchmark suite
./run_benchmarks.sh
```

### Benchmark Suites

1. **packet_building** - Packet construction performance
   - Zero-copy vs allocation
   - Different packet types
   - Buffer pool efficiency

2. **config_validation** - Configuration validation speed
   - Valid config building
   - Invalid config detection
   - Protocol mix validation

3. **lockfree_stats** - Statistics performance
   - Lock-free vs traditional (2x improvement)
   - Batched updates (11x improvement)
   - Per-CPU aggregation

4. **raii_guards** - RAII overhead measurement
   - Guard lifecycle
   - Manual vs RAII cleanup
   - Nested guards

5. **abstractions** - Abstraction layer overhead
   - Network provider
   - System provider
   - Zero overhead verification

### Writing Benchmarks

Example benchmark:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("feature_name");
    
    group.bench_function("test_case", |b| {
        b.iter(|| {
            black_box(function_to_benchmark());
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_feature);
criterion_main!(benches);
```

### Performance Results

Current performance metrics:
- **Lock-free stats**: ~18ns per increment (2x faster)
- **Batched updates**: ~1.9ns (11x faster)
- **Packet building**: ~560ns for UDP packet
- **RAII guards**: Zero measurable overhead
- **Abstractions**: Zero overhead

## Code Quality

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Run with all features
cargo clippy --all-features -- -D warnings

# Fix clippy suggestions
cargo clippy --fix
```

### Formatting

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Format specific file
cargo fmt -- src/main.rs
```

### Documentation

```bash
# Generate documentation
cargo doc --no-deps

# Generate and open documentation
cargo doc --open

# Document private items
cargo doc --document-private-items
```

## Debugging

### Debug Builds

```bash
# Build with debug symbols
cargo build

# Run with debug output
RUST_LOG=debug cargo run -- --target 192.168.1.1

# Run with backtrace
RUST_BACKTRACE=1 cargo run -- --target 192.168.1.1
```

### Logging

Configure logging levels:

```bash
# Set log level
export RUST_LOG=router_flood=debug

# Specific module logging
export RUST_LOG=router_flood::stats=trace

# Multiple modules
export RUST_LOG=router_flood::stats=debug,router_flood::packet=trace
```

### Using GDB

```bash
# Build with debug symbols
cargo build

# Run with GDB
gdb target/debug/router-flood

# GDB commands
(gdb) break main
(gdb) run --target 192.168.1.1
(gdb) backtrace
```

## Fuzzing

### Setup

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing (already done)
cargo fuzz init
```

### Running Fuzzers

```bash
# List available fuzz targets
cargo fuzz list

# Run packet builder fuzzer
cargo fuzz run fuzz_packet_builder

# Run with specific timeout
cargo fuzz run fuzz_packet_builder -- -max_total_time=60

# Run configuration fuzzer
cargo fuzz run fuzz_config_parser
```

### Adding Fuzz Targets

Create new fuzz target in `fuzz/fuzz_targets/`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use router_flood::packet::PacketBuilder;

fuzz_target!(|data: &[u8]| {
    // Fuzz implementation
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = PacketBuilder::from_str(s);
    }
});
```

## Security Testing

### Capability Testing

```bash
# Test without capabilities (should fail)
./target/release/router-flood --target 192.168.1.1

# Grant capability
sudo setcap cap_net_raw+ep ./target/release/router-flood

# Test with capability (should work)
./target/release/router-flood --target 192.168.1.1 --dry-run
```

### Audit Log Verification

```bash
# Run with audit logging
./target/release/router-flood --target 192.168.1.1 --audit-log audit.log

# Verify audit log integrity
./target/release/router-flood verify-audit audit.log
```

## Continuous Integration

### GitHub Actions Workflow

The project uses GitHub Actions for CI/CD:

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
```

### Pre-commit Hooks

Install pre-commit hooks:

```bash
# Create hooks directory
mkdir -p .git/hooks

# Create pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
EOF

# Make executable
chmod +x .git/hooks/pre-commit
```

## Release Process

### Version Bumping

```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
# Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "Release v0.0.2"

# Create tag
git tag -a v0.0.2 -m "Release version 0.0.2"

# Push changes
git push origin main --tags
```

### Building Release

```bash
# Build optimized release
cargo build --release

# Strip debug symbols
strip target/release/router-flood

# Create distribution package
tar czf router-flood-v0.0.2-linux-x64.tar.gz \
    target/release/router-flood \
    README.md \
    LICENSE \
    examples/
```

## Contributing

### Code Style Guidelines

1. **Follow Rust conventions**: Use `cargo fmt` and `cargo clippy`
2. **Write tests**: Add tests for new functionality
3. **Document code**: Add rustdoc comments for public APIs
4. **Use RAII patterns**: Prefer RAII guards for resource management
5. **Prefer lock-free**: Use atomic operations where appropriate
6. **Maintain backward compatibility**: Use adapter patterns when needed

### Pull Request Process

1. Fork the repository
2. Create feature branch: `git checkout -b feature/my-feature`
3. Make changes and add tests
4. Run tests: `cargo test`
5. Run benchmarks: `cargo bench`
6. Check formatting: `cargo fmt --check`
7. Run linter: `cargo clippy -- -D warnings`
8. Commit changes: `git commit -m "feat: add new feature"`
9. Push branch: `git push origin feature/my-feature`
10. Create pull request

### Commit Message Format

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Testing
- `chore`: Maintenance

Example:
```
feat(stats): add lock-free statistics implementation

- Implement atomic counters for thread-safe updates
- Add per-CPU aggregation for cache locality
- Provide 2x performance improvement over mutex-based approach

Closes #123
```

## Troubleshooting

### Common Issues

#### Permission Denied
```bash
# Solution: Grant capability
sudo setcap cap_net_raw+ep ./target/release/router-flood
```

#### Benchmark Timeout
```bash
# Solution: Use quick test script
./test_bench.sh
```

#### Test Failures
```bash
# Run with verbose output
cargo test -- --nocapture --test-threads=1
```

#### Memory Issues
```bash
# Run with valgrind
valgrind --leak-check=full ./target/release/router-flood
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Proptest Documentation](https://altsysrq.github.io/proptest-book/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Linux Capabilities](https://man7.org/linux/man-pages/man7/capabilities.7.html)