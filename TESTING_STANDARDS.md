# Testing Standards

## Test Naming Conventions

### Unit Tests
- Test function names should follow: `test_<what>_<condition>_<expected_result>`
- Examples:
  - `test_ip_validation_with_private_ipv4_succeeds()`
  - `test_packet_builder_with_invalid_size_returns_error()`
  - `test_stats_counter_under_concurrent_access_remains_consistent()`

### Integration Tests
- Test function names should follow: `test_<workflow>_<scenario>`
- Examples:
  - `test_security_workflow_with_valid_config()`
  - `test_graceful_shutdown_with_active_workers()`

### Benchmarks
- Benchmark function names should follow: `bench_<operation>_<variant>`
- Examples:
  - `bench_packet_generation_udp()`
  - `bench_stats_update_concurrent()`

## Test Organization

### Directory Structure
```
tests/
├── unit/                 # Unit tests for individual modules
│   ├── security/        # Security validation tests
│   ├── config/          # Configuration tests
│   ├── stats/           # Statistics tests
│   ├── packet/          # Packet generation tests
│   └── error/           # Error handling tests
├── integration/         # End-to-end integration tests
├── common/             # Shared test utilities
│   ├── mod.rs         # Module exports
│   ├── fixtures.rs    # Test data generators
│   └── assertions.rs  # Custom assertions
└── *.rs               # Individual test files

benches/               # Performance benchmarks
├── packet_generation.rs
├── stats_collection.rs
├── memory_pool.rs
└── throughput.rs
```

### Module Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::*;
    
    // Group related tests in nested modules
    mod validation {
        use super::*;
        
        #[test]
        fn test_valid_input() { /* ... */ }
        
        #[test]
        fn test_invalid_input() { /* ... */ }
    }
    
    mod error_cases {
        use super::*;
        
        #[test]
        fn test_error_condition() { /* ... */ }
    }
}
```

## Assertion Patterns

### Basic Assertions
```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, not_expected);

// Boolean conditions
assert!(condition);
assert!(!condition);

// Custom messages
assert!(condition, "Failed because: {}", reason);
```

### Result Assertions
```rust
// Success cases
let result = function_under_test();
assert!(result.is_ok());
let value = result.unwrap();

// Error cases
let result = function_under_test();
assert!(result.is_err());
let error = result.unwrap_err();
assert_eq!(error.kind(), ErrorKind::Validation);
```

### Panic Testing
```rust
#[test]
#[should_panic(expected = "specific panic message")]
fn test_panics_on_invalid_state() {
    // Code that should panic
}
```

### Custom Assertions
```rust
// In tests/common/assertions.rs
pub fn assert_in_range<T: PartialOrd + Display>(value: T, min: T, max: T) {
    assert!(
        value >= min && value <= max,
        "Value {} is not in range [{}, {}]",
        value, min, max
    );
}
```

## Coverage Requirements

### Mandatory Coverage
- **Security validation**: 100% coverage required
- **Configuration validation**: 100% coverage required
- **Error handling paths**: >90% coverage required
- **Public API**: >95% coverage required

### Coverage Targets by Module
| Module | Line Coverage | Branch Coverage |
|--------|--------------|-----------------|
| security | 100% | 100% |
| config | 100% | 95% |
| error | 95% | 90% |
| packet | 90% | 85% |
| stats | 90% | 85% |
| utils | 85% | 80% |

### Measuring Coverage
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Generate coverage with specific features
cargo tarpaulin --features "feature1 feature2" --out Xml
```

## Test Patterns

### Arrange-Act-Assert (AAA)
```rust
#[test]
fn test_example() {
    // Arrange: Set up test data and environment
    let config = test_config_builder()
        .with_threads(4)
        .build();
    
    // Act: Execute the operation
    let result = validate_config(&config);
    
    // Assert: Verify the outcome
    assert!(result.is_ok());
    assert_eq!(result.unwrap().threads, 4);
}
```

### Table-Driven Tests
```rust
#[test]
fn test_ip_validation() {
    let test_cases = vec![
        ("192.168.1.1", true, "private IPv4"),
        ("8.8.8.8", false, "public IPv4"),
        ("127.0.0.1", false, "loopback"),
        ("fe80::1", true, "private IPv6"),
    ];
    
    for (ip, expected, description) in test_cases {
        let result = validate_ip(ip);
        assert_eq!(
            result.is_ok(), 
            expected, 
            "Failed for {}: {}", 
            description, 
            ip
        );
    }
}
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_packet_size_bounds(
        size in 20..=9000usize
    ) {
        let packet = create_packet(size);
        prop_assert!(packet.len() >= 20);
        prop_assert!(packet.len() <= 9000);
    }
}
```

## Test Execution

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test integration_tests

# Run tests matching pattern
cargo test test_security

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4

# Run ignored tests
cargo test -- --ignored
```

### Benchmark Execution
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench packet_generation

# Save baseline
cargo bench -- --save-baseline my_baseline

# Compare with baseline
cargo bench -- --baseline my_baseline
```

## Best Practices

### Do's
- ✅ Keep tests focused and independent
- ✅ Use descriptive test names
- ✅ Test both success and failure paths
- ✅ Use test fixtures for complex data
- ✅ Clean up resources in tests
- ✅ Use `#[ignore]` for slow tests
- ✅ Document why a test exists if not obvious

### Don'ts
- ❌ Don't test implementation details
- ❌ Don't use production data in tests
- ❌ Don't rely on test execution order
- ❌ Don't use hard-coded delays
- ❌ Don't ignore flaky tests
- ❌ Don't test external dependencies directly
- ❌ Don't write tests that could bypass security

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run benchmarks (no-run)
        run: cargo bench --no-run
```

## Test Maintenance

### Review Checklist
- [ ] Test name clearly describes what is being tested
- [ ] Test has clear arrange-act-assert structure
- [ ] Test is independent of other tests
- [ ] Test cleanup is performed if needed
- [ ] Test failure provides helpful error message
- [ ] Test covers edge cases
- [ ] Test is in the appropriate location

### Updating Tests
When code changes require test updates:
1. Update test to match new behavior
2. Verify test still provides value
3. Add tests for new functionality
4. Remove tests for deleted functionality
5. Update documentation if needed

## Security Considerations

### Testing Security Features
- Always test that security restrictions work
- Never test ways to bypass security
- Document security implications in tests
- Use private IP ranges in all network tests
- Validate all input boundaries

### Sensitive Data
- Never use real credentials in tests
- Use mock data for sensitive operations
- Clear sensitive test data after use
- Don't log sensitive information