# Contributing to Router Flood

Thank you for your interest in contributing to Router Flood! This document provides guidelines and information for contributors.

## 🎯 Project Mission

Router Flood is an educational network stress testing tool designed to:

- **Educate**: Help users understand network behavior and security
- **Secure**: Provide safe, controlled testing environments
- **Perform**: Deliver high-performance network testing capabilities
- **Comply**: Ensure responsible and legal usage

## 🤝 How to Contribute

### Types of Contributions

We welcome various types of contributions:

- 🐛 **Bug Reports**: Help us identify and fix issues
- 💡 **Feature Requests**: Suggest new functionality
- 📝 **Documentation**: Improve guides, examples, and API docs
- 🧪 **Testing**: Add tests, improve coverage, or report test failures
- ⚡ **Performance**: Optimize algorithms or system integration
- 🔒 **Security**: Enhance security features or report vulnerabilities
- 🎨 **User Experience**: Improve CLI, error messages, or workflows

### Getting Started

1. **Fork the Repository**
   ```bash
   git clone https://github.com/your-username/router-flood.git
   cd router-flood
   ```

2. **Set Up Development Environment**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install development tools
   cargo install cargo-fuzz
   cargo install cargo-audit
   cargo install criterion
   
   # Build the project
   cargo build
   
   # Run tests
   cargo test
   ```

3. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

## 📋 Development Guidelines

### Code Style

We follow standard Rust conventions with some project-specific guidelines:

#### Formatting
```bash
# Format code before committing
cargo fmt

# Check formatting
cargo fmt --check
```

#### Linting
```bash
# Run clippy with strict settings
cargo clippy -- -D warnings

# Security-focused linting
cargo clippy -- -D warnings -D clippy::security
```

#### Code Organization

- **Modules**: Keep modules focused and well-documented
- **Functions**: Prefer small, single-purpose functions
- **Error Handling**: Use `Result<T>` and proper error propagation
- **Documentation**: Document public APIs with examples
- **Testing**: Write tests for new functionality

### Coding Standards

#### Error Handling
```rust
// ✅ Good: Proper error handling
pub fn validate_ip(ip: &str) -> Result<IpAddr> {
    ip.parse()
        .map_err(|e| ValidationError::InvalidIp {
            ip: ip.to_string(),
            reason: e.to_string(),
        }.into())
}

// ❌ Bad: Panic on error
pub fn validate_ip(ip: &str) -> IpAddr {
    ip.parse().unwrap()  // Don't do this!
}
```

#### Documentation
```rust
/// Validates that an IP address is in a private range
///
/// # Arguments
/// * `ip` - The IP address to validate
///
/// # Returns
/// * `Ok(())` if the IP is in a private range
/// * `Err(ValidationError)` if the IP is public or invalid
///
/// # Examples
/// ```
/// use router_flood::validation::validate_private_ip;
/// use std::net::IpAddr;
///
/// let private_ip: IpAddr = "192.168.1.1".parse().unwrap();
/// assert!(validate_private_ip(&private_ip).is_ok());
/// ```
pub fn validate_private_ip(ip: &IpAddr) -> Result<()> {
    // Implementation...
}
```

#### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_private_ip_validation() {
        let private_ip: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(validate_private_ip(&private_ip).is_ok());
        
        let public_ip: IpAddr = "8.8.8.8".parse().unwrap();
        assert!(validate_private_ip(&public_ip).is_err());
    }
    
    #[tokio::test]
    async fn test_async_functionality() {
        // Async test example
    }
}
```

### Performance Guidelines

- **Zero-Copy**: Prefer zero-copy operations where possible
- **SIMD**: Use SIMD optimizations for performance-critical paths
- **Memory**: Minimize allocations in hot paths
- **Profiling**: Profile performance-critical changes
- **Benchmarks**: Add benchmarks for performance improvements

### Security Guidelines

- **Input Validation**: Validate all external inputs
- **Capability Principle**: Use minimal required privileges
- **Audit Logging**: Log security-relevant events
- **Safe Defaults**: Choose secure defaults
- **Error Information**: Avoid leaking sensitive information in errors

## 🧪 Testing

### Test Categories

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **Property Tests**: Test with generated inputs
4. **Security Tests**: Test security features
5. **Performance Tests**: Benchmark critical paths

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test category
cargo test --test integration_tests
cargo test --test property_based_tests

# Run with coverage
cargo test --all-features

# Run benchmarks
cargo bench

# Run fuzzing
cargo fuzz run fuzz_packet_builder
```

### Writing Tests

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_packet_creation() {
        let packet = create_udp_packet("192.168.1.1", 80, &[1, 2, 3, 4]);
        assert!(packet.is_ok());
        assert_eq!(packet.unwrap().len(), 32); // 20 (IP) + 8 (UDP) + 4 (payload)
    }
}
```

#### Property Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_packet_size_bounds(
        payload_size in 1usize..=1400
    ) {
        let packet = create_packet_with_size(payload_size);
        prop_assert!(packet.is_ok());
        prop_assert!(packet.unwrap().len() >= payload_size);
    }
}
```

#### Integration Tests
```rust
#[tokio::test]
async fn test_full_simulation() {
    let config = get_test_config();
    let result = run_simulation(config).await;
    assert!(result.is_ok());
    assert!(result.unwrap().packets_sent > 0);
}
```

## 📝 Documentation

### Documentation Types

1. **API Documentation**: Rust doc comments for public APIs
2. **User Guides**: Markdown documentation for users
3. **Examples**: Working code examples
4. **Architecture**: High-level design documentation

### Writing Documentation

#### API Documentation
- Use `///` for public items
- Include examples for complex functions
- Document error conditions
- Link to related functions

#### User Documentation
- Write for your audience (beginners vs. experts)
- Include working examples
- Explain the "why" not just the "how"
- Keep it up to date

### Building Documentation

```bash
# Build API documentation
cargo doc --open

# Check documentation
cargo doc --no-deps

# Test documentation examples
cargo test --doc
```

## 🔄 Pull Request Process

### Before Submitting

1. **Test Your Changes**
   ```bash
   cargo test --all-features
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

2. **Update Documentation**
   - Update API docs for code changes
   - Update user guides for new features
   - Add examples for new functionality

3. **Add Tests**
   - Unit tests for new functions
   - Integration tests for new features
   - Property tests for complex logic

### Pull Request Template

When creating a pull request, please include:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Security enhancement

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] Performance impact assessed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added for new functionality
- [ ] All tests pass
- [ ] No new clippy warnings
```

### Review Process

1. **Automated Checks**: CI runs tests, linting, and security scans
2. **Code Review**: Maintainers review code quality and design
3. **Testing**: Reviewers test functionality
4. **Approval**: At least one maintainer approval required
5. **Merge**: Squash and merge to main branch

## 🐛 Bug Reports

### Before Reporting

1. **Search Existing Issues**: Check if the bug is already reported
2. **Reproduce**: Ensure the bug is reproducible
3. **Minimal Example**: Create a minimal reproduction case
4. **Environment**: Note your system configuration

### Bug Report Template

```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g., Ubuntu 22.04]
- Rust Version: [e.g., 1.70.0]
- Router Flood Version: [e.g., 1.0.0]

## Additional Context
Any other relevant information
```

## 💡 Feature Requests

### Before Requesting

1. **Check Existing Issues**: See if the feature is already requested
2. **Consider Scope**: Ensure it fits the project mission
3. **Think About Implementation**: Consider how it might work
4. **Provide Use Cases**: Explain why it's needed

### Feature Request Template

```markdown
## Feature Description
Clear description of the proposed feature

## Use Case
Why is this feature needed?

## Proposed Solution
How should this feature work?

## Alternatives Considered
Other approaches you've considered

## Additional Context
Any other relevant information
```

## 🔒 Security Contributions

### Security-Related Changes

Security contributions require special attention:

1. **Follow Security Policy**: Read [SECURITY.md](SECURITY.md)
2. **Private Disclosure**: Report vulnerabilities privately first
3. **Security Review**: Additional security-focused review
4. **Testing**: Comprehensive security testing required

### Security Testing

```bash
# Run security-focused tests
cargo test security

# Audit dependencies
cargo audit

# Run fuzzing
cargo fuzz run fuzz_packet_builder

# Check for common security issues
cargo clippy -- -D clippy::security
```

## 🏆 Recognition

### Contributors

We recognize contributors in several ways:

- **Contributors File**: Listed in CONTRIBUTORS.md
- **Release Notes**: Mentioned in release announcements
- **GitHub**: Contributor statistics and badges
- **Special Recognition**: Outstanding contributions highlighted

### Contribution Types

All contributions are valued:

- **Code**: New features, bug fixes, optimizations
- **Documentation**: Guides, examples, API docs
- **Testing**: Test cases, bug reports, quality assurance
- **Community**: Helping users, answering questions
- **Design**: UX improvements, architecture suggestions

## 📞 Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Discord**: Real-time chat (link in README)
- **Email**: maintainers@router-flood.org

### Mentorship

New contributors can get help with:

- **First Contribution**: Guidance on getting started
- **Code Review**: Learning from feedback
- **Best Practices**: Rust and project-specific patterns
- **Architecture**: Understanding system design

## 📋 Contributor License Agreement

By contributing to Router Flood, you agree that:

1. **License**: Your contributions will be licensed under the MIT License
2. **Originality**: Your contributions are your original work
3. **Rights**: You have the right to submit the contributions
4. **No Warranty**: Contributions are provided "as is"

## 🙏 Thank You

Thank you for contributing to Router Flood! Your contributions help make network testing safer, more educational, and more accessible for everyone.

---

**Questions?** Feel free to ask in GitHub Discussions or reach out to the maintainers.