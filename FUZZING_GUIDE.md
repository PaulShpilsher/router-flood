# 🔍 Router Flood Fuzzing Guide

This guide covers the fuzzing infrastructure for Router Flood, including setup, usage, and maintenance of fuzz targets.

## 📋 Overview

Router Flood includes comprehensive fuzzing support to ensure robustness and security. The fuzzing infrastructure tests critical components with random inputs to discover edge cases, crashes, and potential security vulnerabilities.

## 🎯 Fuzz Targets

Router Flood currently includes 3 fuzz targets:

| Target | Purpose | Component Tested |
|--------|---------|------------------|
| `fuzz_config_parser` | YAML configuration parsing | Configuration validation and parsing |
| `fuzz_cli_parser` | CLI argument parsing | Command-line interface parsing |
| `fuzz_packet_builder` | Packet construction | Multi-protocol packet building |

## 🚀 Quick Start

### Prerequisites

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Ensure you have the nightly toolchain
rustup toolchain install nightly
```

### Running Fuzz Tests

```bash
# List all available fuzz targets
cargo fuzz list

# Run configuration parser fuzzing
cargo fuzz run fuzz_config_parser

# Run CLI parser fuzzing
cargo fuzz run fuzz_cli_parser

# Run packet builder fuzzing
cargo fuzz run fuzz_packet_builder

# Run with specific options
cargo fuzz run fuzz_config_parser -- -max_total_time=60
```

## 🔧 Fuzz Target Details

### 1. Configuration Parser Fuzzer (`fuzz_config_parser`)

**Purpose**: Tests YAML configuration parsing with malformed inputs.

**What it tests**:
- YAML parsing robustness
- Configuration validation
- Error handling for invalid configurations
- Memory safety during parsing

**Input generation**:
- Random byte sequences converted to UTF-8
- Common YAML prefixes and patterns
- Malformed YAML structures

**Example usage**:
```bash
# Run for 5 minutes
cargo fuzz run fuzz_config_parser -- -max_total_time=300

# Run with specific corpus
cargo fuzz run fuzz_config_parser corpus/fuzz_config_parser
```

### 2. CLI Parser Fuzzer (`fuzz_cli_parser`)

**Purpose**: Tests command-line argument parsing with various inputs.

**What it tests**:
- Port parsing robustness
- Numeric argument validation
- Export format parsing
- Error handling for invalid arguments

**Input generation**:
- Random strings for port lists
- Numeric variations and edge cases
- Format string variations
- Special characters and Unicode

**Example usage**:
```bash
# Run with verbose output
cargo fuzz run fuzz_cli_parser -- -verbosity=2

# Run with memory limit
cargo fuzz run fuzz_cli_parser -- -rss_limit_mb=1024
```

### 3. Packet Builder Fuzzer (`fuzz_packet_builder`)

**Purpose**: Tests packet construction with random parameters.

**What it tests**:
- Packet size validation
- Protocol mix normalization
- IP address handling (IPv4/IPv6)
- Buffer overflow protection
- Zero-copy packet building

**Input generation**:
- Random packet sizes and ranges
- Protocol ratio combinations
- IP address variations
- Buffer size edge cases

**Example usage**:
```bash
# Run with specific seed for reproducibility
cargo fuzz run fuzz_packet_builder -- -seed=12345

# Run with artifact minimization
cargo fuzz run fuzz_packet_builder -- -minimize_crash=crash-file
```

## 📊 Monitoring and Analysis

### Viewing Results

```bash
# Check fuzzing statistics
cargo fuzz coverage fuzz_config_parser

# View crash artifacts
ls fuzz/artifacts/fuzz_config_parser/

# Reproduce a crash
cargo fuzz run fuzz_config_parser fuzz/artifacts/fuzz_config_parser/crash-file
```

### Coverage Analysis

```bash
# Generate coverage report
cargo fuzz coverage fuzz_config_parser

# View coverage in browser
cargo fuzz coverage fuzz_config_parser --dev
```

## 🛠️ Maintenance

### Adding New Fuzz Targets

1. Create a new fuzz target file:
```bash
cargo fuzz add fuzz_new_component
```

2. Edit the generated file in `fuzz/fuzz_targets/fuzz_new_component.rs`:
```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use router_flood::your_module::*;

fuzz_target!(|data: &[u8]| {
    // Your fuzzing logic here
    let _ = your_function(data);
});
```

3. Add any required dependencies to `fuzz/Cargo.toml`:
```toml
[dependencies]
libfuzzer-sys = "0.4"
router-flood = { path = ".." }
# Add other dependencies as needed
```

### Updating Existing Targets

When updating fuzz targets:

1. Ensure all imports use correct syntax (`libfuzzer_sys` not `libfuzzer-sys`)
2. Add required dependencies to `fuzz/Cargo.toml`
3. Handle type conversions properly (e.g., `Cow<str>` to `String`)
4. Add `Arbitrary` derive for custom input structures

### Corpus Management

```bash
# Add interesting inputs to corpus
cp interesting-input.yaml fuzz/corpus/fuzz_config_parser/

# Merge corpora from different runs
cargo fuzz cmin fuzz_config_parser

# Minimize corpus size
cargo fuzz tmin fuzz_config_parser crash-file
```

## 🔍 Troubleshooting

### Common Issues

#### Build Errors

**Problem**: `libfuzzer-sys` import errors
```
error: expected one of `::`, `;`, or `as`, found `-`
```

**Solution**: Use underscore instead of hyphen:
```rust
// Wrong
use libfuzzer-sys::fuzz_target;

// Correct
use libfuzzer_sys::fuzz_target;
```

**Problem**: Missing dependencies
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate
```

**Solution**: Add dependencies to `fuzz/Cargo.toml`:
```toml
[dependencies]
libfuzzer-sys = "0.4"
router-flood = { path = ".." }
serde_yaml = "0.9"
arbitrary = { version = "1.0", features = ["derive"] }
```

#### Runtime Issues

**Problem**: Fuzz target crashes immediately

**Solution**: Check for:
- Proper error handling in fuzzed functions
- Buffer overflow protection
- Input validation

**Problem**: Low coverage or no new paths

**Solution**:
- Review input generation strategy
- Add more diverse seed inputs
- Increase fuzzing time
- Check for early returns in code

### Performance Optimization

```bash
# Use multiple cores
cargo fuzz run fuzz_config_parser -- -workers=4

# Increase memory limit
cargo fuzz run fuzz_config_parser -- -rss_limit_mb=2048

# Use faster fuzzing mode
cargo fuzz run fuzz_config_parser -- -use_value_profile=1
```

## 📈 Best Practices

### Input Design

1. **Diverse Inputs**: Generate varied input patterns
2. **Edge Cases**: Include boundary conditions
3. **Real-world Data**: Use realistic input samples
4. **Gradual Complexity**: Start simple, increase complexity

### Error Handling

1. **Graceful Failures**: Ensure functions don't panic
2. **Resource Cleanup**: Properly clean up resources
3. **Memory Safety**: Prevent buffer overflows
4. **Input Validation**: Validate all inputs thoroughly

### Continuous Integration

```yaml
# Example GitHub Actions workflow
name: Fuzzing
on: [push, pull_request]

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run fuzz_config_parser -- -max_total_time=60
      - run: cargo fuzz run fuzz_cli_parser -- -max_total_time=60
      - run: cargo fuzz run fuzz_packet_builder -- -max_total_time=60
```

## 🔒 Security Considerations

### Responsible Disclosure

If fuzzing discovers security vulnerabilities:

1. **Do not** publish details publicly
2. Follow the [Security Policy](SECURITY.md)
3. Report to maintainers privately
4. Allow time for fixes before disclosure

### Safe Fuzzing

1. **Isolated Environment**: Run fuzzing in containers or VMs
2. **Resource Limits**: Set appropriate memory and time limits
3. **Network Isolation**: Prevent network access during fuzzing
4. **Regular Updates**: Keep fuzzing tools updated

## 📚 Additional Resources

- [cargo-fuzz Documentation](https://rust-fuzz.github.io/book/)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing Best Practices](https://github.com/google/fuzzing/blob/master/docs/good-fuzz-target.md)
- [Router Flood Security Policy](SECURITY.md)

## 🤝 Contributing

### Adding Fuzz Targets

1. Identify components that handle external input
2. Create focused fuzz targets for specific functions
3. Ensure good input coverage
4. Add documentation and examples
5. Test thoroughly before submitting

### Improving Existing Targets

1. Analyze coverage reports
2. Add missing input patterns
3. Optimize performance
4. Fix any discovered issues
5. Update documentation

---

**Remember**: Fuzzing is an ongoing process. Regular fuzzing helps maintain code quality and security as the project evolves.