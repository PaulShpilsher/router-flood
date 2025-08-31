# Performance Guide

## Overview

Router Flood achieves high-performance packet generation through carefully selected optimizations. This guide details the performance features, their implementation, and tuning recommendations.

## Performance Metrics

### Baseline Performance

| Metric | Value | Conditions |
|--------|-------|------------|
| Max Packet Rate | 10M pps | 8-core system, 64-byte packets |
| Throughput | 10 Gbps | 1400-byte packets |
| Latency | < 1µs | Per-packet processing |
| Memory Usage | ~100MB | Steady state |
| CPU Efficiency | 85-90% | Worker thread utilization |

## Core Optimizations

### 1. SIMD Payload Generation

**Location**: `src/performance/simd.rs`

SIMD (Single Instruction, Multiple Data) accelerates payload generation by processing multiple bytes in parallel.

#### Implementation Details

```rust
// AVX2: Process 32 bytes at once
#[target_feature(enable = "avx2")]
unsafe fn fill_payload_avx2(buffer: &mut [u8], pattern: u8) {
    let pattern_vec = _mm256_set1_epi8(pattern as i8);
    // Process 32-byte chunks
    for chunk in buffer.chunks_exact_mut(32) {
        _mm256_storeu_si256(chunk.as_mut_ptr() as *mut __m256i, pattern_vec);
    }
}
```

#### Performance Impact
- **3-5x faster** payload generation
- **256-bit** operations with AVX2
- **128-bit** operations with SSE4.2
- **Automatic fallback** for older CPUs

#### CPU Feature Detection
```rust
if is_x86_feature_detected!("avx2") {
    // Use AVX2 path
} else if is_x86_feature_detected!("sse4.2") {
    // Use SSE4.2 path
} else {
    // Fallback to standard
}
```

### 2. Lock-Free Memory Pool

**Location**: `src/performance/memory_pool.rs`

The Treiber stack algorithm provides lock-free buffer management.

#### Algorithm Overview
```rust
pub struct MemoryPool {
    head: AtomicPtr<BufferNode>,
    buffer_size: usize,
    capacity: usize,
}

struct BufferNode {
    buffer: Box<[u8]>,
    next: *mut BufferNode,
}
```

#### Benefits
- **Zero allocations** after initialization
- **No mutex contention**
- **Wait-free** for single producer/consumer
- **Lock-free** for multiple threads

#### Usage Pattern
```rust
// Get buffer from pool
let buffer = pool.get().unwrap_or_else(|| {
    vec![0u8; BUFFER_SIZE].into_boxed_slice()
});

// Automatic return via RAII guard
let _guard = BufferGuard::new(buffer, pool.clone());
```

### 3. CPU Affinity

**Location**: `src/performance/cpu_affinity.rs`

Pin worker threads to specific CPU cores for optimal cache utilization.

#### NUMA Awareness
```rust
pub fn set_thread_affinity(cpu_id: usize) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let mut cpu_set: libc::cpu_set_t = unsafe { mem::zeroed() };
        unsafe {
            libc::CPU_SET(cpu_id, &mut cpu_set);
            libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), &cpu_set);
        }
    }
}
```

#### Performance Gains
- **15-25% throughput increase**
- **Reduced context switching**
- **Better L1/L2 cache hit rates**
- **NUMA-local memory access**

### 4. Batched Statistics

**Location**: `src/stats/stats_aggregator.rs`

Reduce atomic contention through local accumulation.

#### Batching Strategy
```rust
pub struct BatchStats {
    stats: Arc<Stats>,     // Global atomics
    packets_sent: u64,     // Local accumulation
    bytes_sent: u64,
    packets_failed: u64,
    batch_size: u64,       // Flush threshold
}

impl BatchStats {
    pub fn record_success(&mut self, bytes: u64) {
        self.packets_sent += 1;
        self.bytes_sent += bytes;
        self.count += 1;
        
        if self.count >= self.batch_size {
            self.flush();  // Atomic update
        }
    }
}
```

#### Impact
- **50x reduction** in atomic operations
- **Configurable batch size** (default: 50)
- **Automatic flush** on threshold

### 5. Batched RNG

**Location**: `src/utils/rng.rs`

Pre-generate random values in batches to reduce overhead.

#### Implementation
```rust
pub struct BatchedRng {
    rng: StdRng,
    port_batch: VecDeque<u16>,      // 1000 pre-generated ports
    sequence_batch: VecDeque<u32>,   // 1000 sequence numbers
    ttl_batch: VecDeque<u8>,        // 1000 TTL values
    // ... other batches
}
```

#### Performance
- **40% reduction** in RNG overhead
- **1000 values** per batch
- **Automatic replenishment**
- **Type-specific batches**

### 6. Zero-Copy Packet Construction

**Location**: `src/packet/builder.rs`

Build packets directly in pre-allocated buffers.

#### Technique
```rust
pub fn build_packet_into_buffer(
    &mut self,
    buffer: &mut [u8],
    packet_type: PacketType,
    target_ip: IpAddr,
    port: u16,
) -> Result<(usize, &'static str)> {
    // Write directly to buffer
    match packet_type {
        PacketType::Udp => {
            // Construct UDP header in-place
            let header = unsafe {
                &mut *(buffer.as_mut_ptr() as *mut UdpHeader)
            };
            header.source_port = self.rng.port().to_be();
            header.dest_port = port.to_be();
            // ...
        }
    }
}
```

#### Benefits
- **No intermediate allocations**
- **30% memory bandwidth saved**
- **Direct buffer writing**
- **Reusable buffers**

## Performance Tuning

### System Configuration

#### Linux Kernel Parameters
```bash
# Increase network buffer sizes
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
sudo sysctl -w net.core.netdev_max_backlog=5000

# Disable CPU frequency scaling
sudo cpupower frequency-set -g performance

# Disable irqbalance for dedicated cores
sudo systemctl stop irqbalance
```

#### NUMA Optimization
```bash
# Check NUMA topology
numactl --hardware

# Run with NUMA binding
numactl --cpunodebind=0 --membind=0 ./router-flood
```

### Application Tuning

#### Worker Thread Count
```yaml
# Optimal: Number of physical cores - 1
threads: 7  # For 8-core system
```

#### Batch Sizes
```rust
// Statistics batching
const STATS_BATCH_SIZE: u64 = 50;  // Balance between latency and throughput

// RNG batching
const RNG_BATCH_SIZE: usize = 1000;  // Amortize generation cost

// Buffer pool size
const POOL_CAPACITY: usize = 1024;  // Enough for all workers
```

#### Packet Size Distribution
```yaml
# Optimize for cache line alignment
packet_size_range: [64, 1472]  # Avoid fragmentation
```

## Benchmarking

### Running Benchmarks

```bash
# Full benchmark suite
cargo bench

# Specific benchmarks
cargo bench --bench packet_generation
cargo bench --bench stats_collection
cargo bench --bench memory_pool
cargo bench --bench simd_performance
```

### Benchmark Results

#### Packet Generation
```
test packet_generation_udp       ... bench:        142 ns/iter
test packet_generation_tcp_syn   ... bench:        156 ns/iter
test packet_generation_icmp      ... bench:        134 ns/iter
```

#### SIMD Performance
```
test payload_fill_standard       ... bench:      1,234 ns/iter
test payload_fill_sse42          ... bench:        412 ns/iter
test payload_fill_avx2           ... bench:        245 ns/iter
```

#### Memory Pool
```
test pool_get_return             ... bench:         18 ns/iter
test pool_concurrent_access      ... bench:         42 ns/iter
```

## Profiling

### Using perf

```bash
# Record profile
sudo perf record -g ./router-flood --target 192.168.1.1

# Analyze
sudo perf report

# Top functions
sudo perf top -p $(pidof router-flood)
```

### Using Flamegraph

```bash
# Generate flamegraph
cargo install flamegraph
cargo build --release
flamegraph ./target/release/router-flood --target 192.168.1.1

# View in browser
firefox flamegraph.svg
```

## Optimization Checklist

### Pre-deployment
- [ ] Build with `--release` and LTO
- [ ] Set CPU governor to performance
- [ ] Configure CPU affinity
- [ ] Tune kernel network parameters
- [ ] Disable unnecessary services
- [ ] Enable huge pages if using large buffers

### Runtime
- [ ] Monitor CPU utilization per core
- [ ] Check for atomic contention
- [ ] Verify SIMD feature detection
- [ ] Monitor memory pool efficiency
- [ ] Track cache miss rates

## Common Performance Issues

### Issue: Low Packet Rate

**Symptoms**: Cannot achieve expected pps
**Solutions**:
1. Increase worker thread count
2. Reduce packet size
3. Check CPU frequency scaling
4. Verify SIMD is enabled

### Issue: High CPU Usage

**Symptoms**: 100% CPU with low throughput
**Solutions**:
1. Enable CPU affinity
2. Increase batch sizes
3. Check for lock contention
4. Profile with `perf`

### Issue: Memory Growth

**Symptoms**: RSS increases over time
**Solutions**:
1. Verify pool capacity is sufficient
2. Check for buffer leaks
3. Monitor pool get/return ratio
4. Enable debug assertions

## Advanced Optimizations

### Future Improvements

#### io_uring Integration
```rust
// Potential 2x throughput improvement
async fn send_with_io_uring(packets: &[Packet]) {
    let ring = IoUring::new(256)?;
    // Batch submission
    for packet in packets {
        ring.submit_send(packet)?;
    }
}
```

#### eBPF Offloading
```c
// Kernel-space packet generation
SEC("xdp")
int generate_packets(struct xdp_md *ctx) {
    // Generate directly in kernel
    return XDP_TX;
}
```

#### DPDK Support
```rust
// Bypass kernel for 100Gbps+
fn dpdk_transmit(port: u16, packets: &[Packet]) {
    // Direct NIC access
    dpdk::tx_burst(port, packets);
}
```

## Conclusion

Router Flood's performance comes from careful optimization of critical paths:
- SIMD for compute-intensive operations
- Lock-free algorithms for concurrency
- Batching to amortize costs
- Zero-copy to reduce memory bandwidth
- CPU affinity for cache optimization

These optimizations work together to achieve high packet rates while maintaining low latency and predictable performance.