# Benchmark Guide

## Overview
This guide provides instructions for running, interpreting, and maintaining performance benchmarks for the router-flood project.

## Running Benchmarks

### Prerequisites
```bash
# Ensure release mode optimizations
cargo build --release

# Install benchmark visualization tools (optional)
cargo install cargo-criterion
```

### Basic Commands

#### Run All Benchmarks
```bash
cargo bench
```

#### Run Specific Benchmark Suite
```bash
# Run packet generation benchmarks
cargo bench packet_generation

# Run statistics collection benchmarks
cargo bench stats_collection

# Run memory pool benchmarks
cargo bench memory_pool

# Run throughput benchmarks
cargo bench throughput
```

#### Run Specific Benchmark Function
```bash
# Run only UDP packet generation benchmark
cargo bench udp_packet_generation
```

### Advanced Options

#### Save Baseline for Comparison
```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Run and compare against baseline
cargo bench -- --baseline main
```

#### Configure Measurement Time
```bash
# Increase measurement time for more accurate results
cargo bench -- --measurement-time 10
```

#### Generate HTML Reports
```bash
# Using cargo-criterion for detailed HTML reports
cargo criterion
```

## Interpreting Results

### Understanding Output

Example output:
```
udp_packet_generation   time:   [784.23 ns 789.45 ns 795.12 ns]
                        change: [-2.1234% -1.0012% +0.1234%] (p = 0.08 > 0.05)
                        No change in performance detected.
```

- **time**: [lower_bound, estimate, upper_bound]
  - Lower bound: Best case measurement
  - Estimate: Most likely value
  - Upper bound: Worst case measurement

- **change**: Performance change from baseline
  - Negative: Performance improvement
  - Positive: Performance regression
  - p-value: Statistical significance (< 0.05 is significant)

### Performance Metrics

#### Throughput Metrics
- **Elements/second**: Operations completed per second
- **Bytes/second**: Data processed per second

#### Latency Metrics
- **ns (nanoseconds)**: For very fast operations
- **μs (microseconds)**: For moderate operations  
- **ms (milliseconds)**: For slower operations

### Benchmark Categories

#### 1. Packet Generation (packet_generation.rs)
- **Target**: < 1μs per packet
- **Measures**: Time to construct various packet types
- **Metrics**: UDP, TCP SYN/ACK, ICMP packet creation times

#### 2. Statistics Collection (stats_collection.rs)
- **Target**: > 10M updates/sec
- **Measures**: Atomic counter performance
- **Metrics**: Single updates, concurrent updates, snapshot generation

#### 3. Memory Pool (memory_pool.rs)
- **Target**: > 1M allocations/sec
- **Measures**: Memory allocation efficiency
- **Metrics**: Pool vs heap allocation, concurrent access

#### 4. Throughput (throughput.rs)
- **Target**: > 1 Gbps simulated
- **Measures**: Overall system throughput
- **Metrics**: Packet throughput, mixed protocols, statistics overhead

## Regression Detection

### Setting Thresholds
Performance regression is flagged when:
- Performance degrades by > 10% from baseline
- Statistical significance (p < 0.05)

### Automatic Detection
```bash
# Run with regression detection
cargo bench -- --baseline main --regress 0.1
```

### Manual Analysis
```python
# Python script for trend analysis
import json

with open('target/criterion/baseline.json') as f:
    baseline = json.load(f)
    
with open('target/criterion/current.json') as f:
    current = json.load(f)
    
for bench in baseline:
    if bench in current:
        change = (current[bench] - baseline[bench]) / baseline[bench]
        if change > 0.1:
            print(f"REGRESSION: {bench} degraded by {change:.2%}")
```

## Performance Targets

### Baseline Targets

| Benchmark | Target | Description |
|-----------|--------|-------------|
| UDP packet generation | < 1μs | Single packet creation time |
| TCP packet generation | < 1μs | Single packet with flags |
| Stats single update | < 100ns | Atomic counter increment |
| Stats concurrent | < 500ns/thread | Concurrent counter updates |
| Memory pool allocation | < 200ns | Pool allocation time |
| Packet throughput | > 100k/s | Packets per second |
| Mixed protocol throughput | > 80k/s | Mixed packet types |

### Performance Goals

#### Phase 1 (Current)
- Establish baselines
- Ensure no obvious bottlenecks
- Document current performance

#### Phase 2 (Optimization)
- Improve packet generation by 20%
- Reduce memory allocation overhead
- Optimize hot paths

#### Phase 3 (Scaling)
- Linear scaling to 8 threads
- Maintain performance under load
- Zero-copy optimizations

## Optimization Guide

### Identifying Bottlenecks

1. **Run Flamegraph**
```bash
cargo install flamegraph
cargo flamegraph --bench packet_generation
```

2. **Profile with perf**
```bash
perf record cargo bench packet_generation
perf report
```

3. **Check Assembly**
```bash
cargo bench -- --emit-asm
```

### Common Optimizations

#### Memory Optimizations
- Use memory pools for frequent allocations
- Implement zero-copy where possible
- Reduce allocations in hot paths

#### Concurrency Optimizations
- Use lock-free data structures
- Minimize contention points
- Batch operations when possible

#### Algorithm Optimizations
- Replace algorithms with O(n²) or worse
- Use lookup tables for computations
- Leverage SIMD when applicable

## Continuous Monitoring

### CI Integration
```yaml
# .github/workflows/benchmark.yml
name: Benchmarks

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Run benchmarks
        run: |
          cargo bench -- --output-format bencher | tee output.txt
          
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### Performance Dashboard
Create a dashboard to track performance over time:

1. Export benchmark results to JSON
2. Store in time-series database
3. Visualize trends with Grafana

Example query:
```sql
SELECT 
    timestamp,
    benchmark_name,
    median_time_ns,
    throughput_ops_sec
FROM benchmarks
WHERE timestamp > NOW() - INTERVAL '30 days'
ORDER BY timestamp DESC;
```

## Troubleshooting

### Inconsistent Results
**Problem**: Benchmark results vary significantly between runs

**Solutions**:
- Increase measurement time: `--measurement-time 20`
- Reduce system load during benchmarks
- Disable CPU frequency scaling
- Pin benchmark to specific CPU cores

### Benchmarks Too Slow
**Problem**: Benchmarks take too long to complete

**Solutions**:
- Reduce sample size: `--sample-size 10`
- Run specific benchmarks only
- Use `--profile-time 2` for faster profiling

### Memory Issues
**Problem**: Out of memory during benchmarks

**Solutions**:
- Reduce concurrent operations
- Clear caches between iterations
- Monitor memory usage with `valgrind`

## Best Practices

### Writing Benchmarks
1. **Isolate the operation being measured**
2. **Use `black_box` to prevent optimization**
3. **Warm up caches before measurement**
4. **Test with realistic data sizes**
5. **Include both best and worst cases**

### Maintaining Benchmarks
1. **Update benchmarks when code changes**
2. **Document what each benchmark measures**
3. **Set realistic performance targets**
4. **Review anomalous results**
5. **Keep historical data for trends**

### Benchmark Code Example
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("operation_group");
    
    // Test different input sizes
    for size in &[100, 1000, 10000] {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                // Setup
                let input = prepare_input(size);
                
                // Benchmark
                b.iter(|| {
                    operation(black_box(&input))
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_operation);
criterion_main!(benches);
```

## References

- [Criterion.rs Documentation](https://docs.rs/criterion)
- [Cargo Bench Guide](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)