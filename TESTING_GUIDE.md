# Testing Guide

## Quick Start

Run all tests:
```bash
cargo test
```

Run specific test file:
```bash
cargo test --test security_tests
cargo test --test packet_tests
cargo test --test edge_case_tests
```

Run benchmarks:
```bash
cargo bench
```

## Test Organization

### Unit Tests (`tests/`)
- `security_tests.rs` - IP validation, security controls (23 tests)
- `packet_tests.rs` - Packet building and protocols (10 tests)
- `stats_tests.rs` - Statistics collection (4 tests)
- `config_tests.rs` - Configuration validation (5 tests)
- `error_tests.rs` - Error handling (3 tests)
- `error_handling_tests.rs` - Comprehensive error handling (10 tests)
- `edge_case_tests.rs` - Edge cases and boundaries (14 tests)
- `integration_tests.rs` - End-to-end workflows (3 tests)
- `performance_tests.rs` - Performance module basics (1 test)
- `property_tests.rs` - Property-based testing with proptest (8 tests)
- `utils_tests.rs` - Utility functions and RNG (15 tests)

### Benchmarks (`benches/`)
- `packet_generation.rs` - Packet creation performance
- `stats_collection.rs` - Statistics update performance

## Key Test Coverage

### Security Validation
- ✅ Private IP range enforcement (192.168.x.x, 10.x.x.x, 172.16-31.x.x)
- ✅ Public IP rejection
- ✅ Loopback/multicast/broadcast blocking
- ✅ Thread and rate limits
- ✅ IPv6 private range validation

### Edge Cases Tested
- Thread counts: 0, 100 (max), 101 (over limit)
- Packet rates: 0, 10000 (max), 10001 (over limit)
- Port numbers: 0, 65535, empty array
- Special IPs: 0.0.0.0, 255.255.255.255, 127.0.0.1, multicast
- Buffer boundaries and packet sizes
- Concurrent operations and race conditions

## Running Tests with Output

Show test names:
```bash
cargo test -- --nocapture
```

Run single test:
```bash
cargo test test_boundary_thread_counts
```

## Test Statistics

- **Total Tests**: 96
- **Test Files**: 11
- **Test Suites**: 14
- **Benchmarks**: 2
- **Pass Rate**: 100%

## Common Test Patterns

### Testing Validation
```rust
assert!(validate_target_ip(&private_ip).is_ok());
assert!(validate_target_ip(&public_ip).is_err());
```

### Testing Boundaries
```rust
assert!(validate_comprehensive_security(&ip, &ports, 100, 1000).is_ok());  // Max allowed
assert!(validate_comprehensive_security(&ip, &ports, 101, 1000).is_err()); // Over limit
```

### Testing Concurrency
```rust
let stats = Arc::new(Stats::new(None));
// Spawn threads that modify stats
// Verify no data races
```

## Notes

- Tests validate defensive security controls
- Empty ports array uses defaults [80, 443]
- Zero threads currently allowed (not validated)
- All tests run in dry-run mode for safety