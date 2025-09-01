# Test Utilities Documentation

## Overview
This directory contains shared utilities for testing the router-flood project. These utilities provide common functionality for fixtures, assertions, and test configuration.

## Available Utilities

### Module Structure
```
tests/common/
├── mod.rs           # Module exports and initialization
├── assertions.rs    # Custom assertion functions
├── fixtures.rs      # Test data generators
├── test_config.rs   # Configuration builders for tests
└── README.md        # This file
```

## fixtures.rs - Test Data Generators

### IP Address Generators
```rust
use crate::common::fixtures::*;

// Generate a valid private IPv4 address
let ip = generate_private_ipv4();
assert!(is_private_ip(&ip));

// Generate a valid private IPv6 address  
let ipv6 = generate_private_ipv6();
assert!(is_private_ipv6(&ipv6));

// Generate a public IP (for testing rejection)
let public_ip = generate_public_ip();
assert!(!is_private_ip(&public_ip));
```

### Port Generators
```rust
// Generate a random valid port
let port = generate_port();
assert!(port > 0 && port < 65536);

// Generate a well-known port (< 1024)
let well_known = generate_well_known_port();
assert!(well_known < 1024);

// Generate a random port range
let ports = generate_port_range(5);
assert_eq!(ports.len(), 5);
```

### Payload Generators
```rust
// Generate random payload of specific size
let payload = generate_payload(1024);
assert_eq!(payload.len(), 1024);

// Generate pattern payload
let pattern = generate_pattern_payload(100, 0xAA);
assert!(pattern.iter().all(|&b| b == 0xAA));

// Generate mixed content payload
let mixed = generate_mixed_payload(500);
assert_eq!(mixed.len(), 500);
```

## assertions.rs - Custom Assertions

### Validation Assertions
```rust
use crate::common::assertions::*;

// Assert that an error is a validation error
assert_validation_error(result, "expected_field");

// Assert that a value is in range
assert_in_range(value, min, max);

// Assert configuration is valid
assert_config_valid(&config);

// Assert security check passed
assert_security_check_passed(&result);
```

### Result Assertions
```rust
// Assert success with specific value
assert_ok_eq(result, expected_value);

// Assert error with specific type
assert_err_type::<ValidationError>(result);

// Assert error message contains
assert_err_contains(result, "substring");
```

### Collection Assertions
```rust
// Assert all elements match predicate
assert_all(vec, |item| item > 0);

// Assert any element matches predicate
assert_any(vec, |item| item == target);

// Assert collection is sorted
assert_sorted(&vec);

// Assert no duplicates
assert_unique(&vec);
```

## test_config.rs - Configuration Builders

### Basic Configuration
```rust
use crate::common::test_config::*;

// Create default test configuration
let config = create_test_config();

// Create minimal valid configuration
let minimal = create_minimal_config();

// Create maximal configuration (all limits)
let maximal = create_maximal_config();
```

### Configuration Builder
```rust
// Use fluent builder API
let config = TestConfigBuilder::new()
    .with_target_ip("192.168.1.100")
    .with_ports(vec![8080, 8081])
    .with_threads(4)
    .with_packet_rate(1000.0)
    .with_dry_run(true)
    .build();

// Builder with validation
let result = TestConfigBuilder::new()
    .with_invalid_values()
    .try_build();
assert!(result.is_err());
```

### Specialized Configurations
```rust
// Security testing configuration
let security_config = create_security_test_config();

// Performance testing configuration
let perf_config = create_performance_test_config();

// Stress testing configuration
let stress_config = create_stress_test_config();

// Edge case configuration
let edge_config = create_edge_case_config();
```

## Usage Examples

### Example 1: Testing IP Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{fixtures::*, assertions::*};
    
    #[test]
    fn test_private_ip_validation() {
        // Generate test data
        let private_ip = generate_private_ipv4();
        let public_ip = generate_public_ip();
        
        // Test validation
        let result1 = validate_ip(&private_ip);
        let result2 = validate_ip(&public_ip);
        
        // Assert results
        assert_ok(result1);
        assert_validation_error(result2, "ip");
    }
}
```

### Example 2: Testing Configuration
```rust
#[test]
fn test_configuration_bounds() {
    use crate::common::test_config::*;
    
    // Test minimal configuration
    let min_config = create_minimal_config();
    assert_config_valid(&min_config);
    
    // Test maximal configuration
    let max_config = create_maximal_config();
    assert_config_valid(&max_config);
    
    // Test invalid configuration
    let invalid = TestConfigBuilder::new()
        .with_threads(1000) // exceeds limit
        .try_build();
    assert_validation_error(invalid, "threads");
}
```

### Example 3: Testing Packet Generation
```rust
#[test]
fn test_packet_generation_with_payloads() {
    use crate::common::fixtures::*;
    
    let payloads = vec![
        generate_payload(64),
        generate_payload(512),
        generate_payload(1400),
    ];
    
    for payload in payloads {
        let packet = create_packet_with_payload(payload);
        assert_in_range(packet.len(), 20, 1500);
    }
}
```

## Extension Guidelines

### Adding New Fixtures
1. Add generator function to `fixtures.rs`
2. Follow naming convention: `generate_<type>()`
3. Include parameters for customization
4. Add documentation with examples
5. Write tests for the generator

Example:
```rust
/// Generate a random MAC address
/// 
/// # Example
/// ```
/// let mac = generate_mac_address();
/// assert_eq!(mac.len(), 6);
/// ```
pub fn generate_mac_address() -> [u8; 6] {
    let mut rng = thread_rng();
    let mut mac = [0u8; 6];
    rng.fill_bytes(&mut mac);
    mac[0] &= 0xFC; // Clear multicast and local bits
    mac
}
```

### Adding New Assertions
1. Add assertion function to `assertions.rs`
2. Follow naming convention: `assert_<condition>()`
3. Provide helpful error messages
4. Return `Result` for composability
5. Document preconditions

Example:
```rust
/// Assert that a packet has valid checksums
pub fn assert_valid_checksums(packet: &[u8]) -> Result<(), String> {
    let ip_checksum = calculate_ip_checksum(packet);
    if ip_checksum != 0 {
        return Err(format!("Invalid IP checksum: {}", ip_checksum));
    }
    
    let tcp_checksum = calculate_tcp_checksum(packet);
    if tcp_checksum != 0 {
        return Err(format!("Invalid TCP checksum: {}", tcp_checksum));
    }
    
    Ok(())
}
```

### Adding Configuration Builders
1. Add builder methods to `test_config.rs`
2. Implement validation in `try_build()`
3. Provide sensible defaults
4. Support method chaining
5. Document constraints

Example:
```rust
impl TestConfigBuilder {
    /// Set custom protocol mix
    pub fn with_protocol_mix(
        mut self,
        udp: f64,
        tcp_syn: f64,
        tcp_ack: f64,
        icmp: f64
    ) -> Self {
        self.protocol_mix = ProtocolMix {
            udp_ratio: udp,
            tcp_syn_ratio: tcp_syn,
            tcp_ack_ratio: tcp_ack,
            icmp_ratio: icmp,
            custom_ratio: 0.0,
        };
        self
    }
}
```

## Best Practices

### Do's
- ✅ Keep utilities focused and reusable
- ✅ Provide clear documentation with examples
- ✅ Use descriptive names
- ✅ Make utilities composable
- ✅ Include edge cases in generators
- ✅ Validate inputs in assertions

### Don'ts
- ❌ Don't duplicate standard library functionality
- ❌ Don't make utilities overly complex
- ❌ Don't use hardcoded values without constants
- ❌ Don't forget error handling
- ❌ Don't create circular dependencies

## Testing the Utilities

The utilities themselves should be tested:

```rust
#[cfg(test)]
mod utility_tests {
    use super::*;
    
    #[test]
    fn test_fixture_generator() {
        let ip = generate_private_ipv4();
        assert!(ip.is_private());
    }
    
    #[test]
    fn test_custom_assertion() {
        let result = Ok(42);
        assert_ok_eq(result, 42);
    }
    
    #[test]
    fn test_config_builder() {
        let config = TestConfigBuilder::new()
            .with_threads(4)
            .build();
        assert_eq!(config.attack.threads, 4);
    }
}
```

## Performance Considerations

- Generators should be fast (< 1ms)
- Avoid expensive operations in assertions
- Cache generated data when reused
- Use lazy_static for shared fixtures
- Profile utilities if tests are slow

## Thread Safety

All utilities should be thread-safe:
- Use `Arc` for shared state
- Avoid global mutable state
- Use thread-local storage when needed
- Document any threading requirements