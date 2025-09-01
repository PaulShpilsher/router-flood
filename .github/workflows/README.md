# CI/CD Workflows

This directory contains GitHub Actions workflows for continuous integration and deployment.

## Workflows

### test.yml - Test Suite
Runs on every push and pull request to ensure code quality.

**Features:**
- Multi-OS testing (Ubuntu, macOS)
- Multiple Rust versions (stable, nightly, MSRV)
- Format checking with rustfmt
- Linting with clippy
- Comprehensive test coverage
- Security audit
- Code coverage reporting with tarpaulin

**Triggers:**
- Push to main or develop branches
- Pull requests to main

### benchmark.yml - Performance Benchmarks
Tracks performance metrics and detects regressions.

**Features:**
- Runs all benchmarks
- Stores results for historical tracking
- Compares PR performance against base branch
- Comments on PRs with performance changes
- Alerts on >10% performance regression

**Triggers:**
- Push to main
- Pull requests to main
- Manual workflow dispatch

### release.yml - Release Automation
Builds and publishes release artifacts.

**Features:**
- Multi-platform binary builds
- Cross-compilation support
- Automated GitHub release creation
- Binary archives for all platforms

**Triggers:**
- Push of version tags (v*)

## Status Badges

Add these to your README.md:

```markdown
[![Test Suite](https://github.com/yourusername/router-flood/actions/workflows/test.yml/badge.svg)](https://github.com/yourusername/router-flood/actions/workflows/test.yml)
[![Benchmarks](https://github.com/yourusername/router-flood/actions/workflows/benchmark.yml/badge.svg)](https://github.com/yourusername/router-flood/actions/workflows/benchmark.yml)
[![codecov](https://codecov.io/gh/yourusername/router-flood/branch/main/graph/badge.svg)](https://codecov.io/gh/yourusername/router-flood)
```

## Local Testing

To test workflows locally before pushing:

```bash
# Install act (GitHub Actions locally)
brew install act  # or your package manager

# Run test workflow
act -j test

# Run specific job
act -j security-audit
```

## Secrets Required

The following secrets should be configured in repository settings:

- `GITHUB_TOKEN` - Automatically provided by GitHub
- `CODECOV_TOKEN` - (Optional) For enhanced Codecov features

## Caching Strategy

All workflows use aggressive caching to speed up builds:
- Cargo registry cache
- Cargo index cache  
- Build target cache

Caches are invalidated when Cargo.lock changes.

## Performance Monitoring

Benchmark results are:
- Stored in GitHub Pages for historical tracking
- Compared on every PR to detect regressions
- Available as artifacts for download

## Security

- Security audit runs on every push
- Dependency vulnerabilities are automatically reported
- Clippy enforces security best practices

## Maintenance

### Updating Rust Version
Update MSRV in test.yml when bumping minimum supported Rust version.

### Adding New Benchmarks
New benchmarks are automatically included if added to `benches/` directory.

### Workflow Debugging
Enable debug logging:
```yaml
env:
  ACTIONS_STEP_DEBUG: true
  ACTIONS_RUNNER_DEBUG: true
```