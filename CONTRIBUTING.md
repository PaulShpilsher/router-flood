# Contributing to router-flood

Thank you for your interest in contributing to router-flood. This document provides guidelines for contributions.

## Getting started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your feature or fix
4. Make your changes
5. Submit a pull request

## Development setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/router-flood.git
cd router-flood

# Add upstream remote
git remote add upstream https://github.com/paulspilsher/router-flood.git

# Create a feature branch
git checkout -b feature/your-feature-name
```

## Code standards

* Format code with `cargo fmt` before committing
* Ensure `cargo clippy` passes without warnings
* Add tests for new functionality
* Update documentation for API changes
* Write clear, descriptive commit messages

## Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test security_tests

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Pull request process

1. Update your branch with the latest upstream changes
2. Ensure all tests pass
3. Update README.md if adding new features
4. Submit PR with clear description of changes
5. Address review feedback promptly

## Reporting issues

When reporting issues, please include:

* Operating system and version
* Rust version (`rustc --version`)
* Steps to reproduce the issue
* Expected vs actual behavior
* Any relevant error messages or logs

## Security vulnerabilities

Please report security vulnerabilities privately via email rather than public issues.

## Code of conduct

* Be respectful and inclusive
* Welcome newcomers and help them get started
* Focus on constructive criticism
* Respect differing opinions and experiences

## License

By contributing, you agree that your contributions will be licensed under the MIT License.