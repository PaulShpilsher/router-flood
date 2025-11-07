# Security Policy

## Security features

router-flood includes multiple security mechanisms to ensure safe operation:

* **Private IP validation** - Only accepts RFC 1918 private addresses (192.168.0.0/16, 10.0.0.0/8, 172.16.0.0/12)
* **Capability-based security** - Uses CAP_NET_RAW instead of requiring root
* **Rate limiting** - Built-in limits prevent accidental network saturation
* **Dry-run mode** - Test configurations without sending packets
* **Resource limits** - Maximum thread and packet rate constraints

## Reporting vulnerabilities

Please report security vulnerabilities through GitHub's private vulnerability reporting feature or via email to the maintainers. Do not create public issues for security vulnerabilities.

When reporting, please include:
* Description of the vulnerability
* Steps to reproduce
* Potential impact
* Suggested mitigation (if any)

## Safe usage guidelines

1. **Use capabilities instead of root**:
   ```bash
   sudo setcap cap_net_raw+ep /tmp/cargo-target/release/router-flood
   ```

2. **Test with dry-run first**:
   ```bash
   router-flood --target 192.168.1.1 --ports 80 --dry-run
   ```

3. **Start with low rates**:
   ```bash
   router-flood --target 192.168.1.1 --ports 80 --rate 100
   ```

## Testing

Security-focused tests can be run with:

```bash
# Run security validation tests
cargo test security

# Check for dependency vulnerabilities
cargo audit

# Run with security lints
cargo clippy -- -D warnings
```

## Responsible disclosure

We follow responsible disclosure practices:
* Initial response within 72 hours
* Fix development based on severity
* Public disclosure after fix is available