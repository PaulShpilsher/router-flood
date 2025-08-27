# Router Flood - Performance Tuning Guide

## Overview

This guide provides comprehensive performance optimization strategies for the router-flood tool, covering system-level optimizations, application tuning, and monitoring best practices.

## Performance Architecture

### High-Level Performance Features

Router-flood incorporates several performance optimizations:

1. **Lock-Free Buffer Pools** - Zero-contention memory management
2. **Zero-Copy Packet Building** - Direct buffer writing eliminates allocations
3. **Batched RNG Operations** - Pre-computed random values reduce overhead
4. **Inline Function Optimization** - Strategic compiler hints for hot paths
5. **Per-Worker Channels** - Eliminates mutex contention between threads
6. **Const Function Optimizations** - Compile-time computations

### Performance Metrics

Key performance indicators to monitor:

| Metric | Target | Critical Threshold |
|--------|--------|--------------------|
| Packets/Second | >10,000 | <1,000 |
| CPU Usage | <70% | >90% |
| Memory Usage | <500MB | >1GB |
| Success Rate | >99% | <95% |
| Packet Build Time | <1ms | >10ms |
| Buffer Pool Utilization | 60-80% | >95% |

## System-Level Optimizations

### Operating System Tuning

#### Network Stack Optimization

```bash
# Increase network buffer sizes
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.rmem_default = 65536' >> /etc/sysctl.conf
echo 'net.core.wmem_default = 65536' >> /etc/sysctl.conf

# Increase network device queue length
echo 'net.core.netdev_max_backlog = 5000' >> /etc/sysctl.conf
echo 'net.core.netdev_budget = 600' >> /etc/sysctl.conf

# Optimize TCP settings
echo 'net.ipv4.tcp_rmem = 4096 65536 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_wmem = 4096 65536 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_congestion_control = bbr' >> /etc/sysctl.conf

# Apply changes
sysctl -p
```

#### Memory Management

```bash
# Increase file descriptor limits
echo 'router-flood soft nofile 65536' >> /etc/security/limits.conf
echo 'router-flood hard nofile 65536' >> /etc/security/limits.conf

# Optimize memory allocation
echo 'vm.swappiness = 10' >> /etc/sysctl.conf
echo 'vm.dirty_ratio = 15' >> /etc/sysctl.conf
echo 'vm.dirty_background_ratio = 5' >> /etc/sysctl.conf

# Huge pages for large memory allocations
echo 'vm.nr_hugepages = 128' >> /etc/sysctl.conf
```

#### CPU Optimization

```bash
# Set CPU governor to performance
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable CPU frequency scaling
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Set CPU affinity for router-flood
taskset -c 0-3 router-flood --config high-performance.yaml
```

#### Interrupt Handling

```bash
# Distribute network interrupts across CPUs
echo 2 > /proc/irq/24/smp_affinity  # CPU 1
echo 4 > /proc/irq/25/smp_affinity  # CPU 2
echo 8 > /proc/irq/26/smp_affinity  # CPU 3

# Use NAPI for network interrupt handling
ethtool -C eth0 rx-usecs 50 rx-frames 32
```

### Hardware Considerations

#### CPU Selection
- **Recommended**: Intel Xeon or AMD EPYC with high single-thread performance
- **Minimum**: 4 cores, 2.4 GHz
- **Optimal**: 8+ cores, 3.0+ GHz with large L3 cache

#### Memory Configuration
- **Minimum**: 4 GB RAM
- **Recommended**: 16+ GB RAM with ECC
- **Optimal**: DDR4-3200 or faster with low latency

#### Network Interface
- **Minimum**: Gigabit Ethernet
- **Recommended**: 10 Gigabit Ethernet with SR-IOV support
- **Optimal**: 25+ Gigabit with DPDK compatibility

## Application-Level Tuning

### Configuration Optimization

#### High-Performance Configuration

```yaml
# high-performance.yaml
attack:
  threads: 16  # Match CPU core count
  packet_rate: 10000  # High packet rate
  packet_size_range: [64, 1400]
  burst_pattern:
    type: "sustained"
    rate: 10000
  randomize_timing: false  # Disable for max performance

performance:
  buffer_pool_size: 2000  # Large buffer pool
  buffer_pool_workers: 4  # Multiple buffer pools
  batch_size: 1000  # Large batches
  zero_copy_enabled: true
  inline_optimizations: true

network:
  interface: "eth0"
  buffer_size: 8192  # Large network buffers
  send_timeout: 100  # Short timeout
  batch_sends: true

monitoring:
  stats_interval: 10  # Less frequent stats
  export_interval: 300
  dashboard_enabled: false  # Disable for max performance
```

#### Memory-Optimized Configuration

```yaml
# memory-optimized.yaml
attack:
  threads: 4  # Fewer threads
  packet_rate: 1000
  
performance:
  buffer_pool_size: 100  # Smaller pool
  max_packet_history: 100
  compression_enabled: true
  
monitoring:
  stats_interval: 30
  export_interval: 600
  memory_limit: 256MB
```

#### Latency-Optimized Configuration

```yaml
# latency-optimized.yaml
attack:
  threads: 1  # Single thread for consistency
  packet_rate: 5000
  randomize_timing: false
  
performance:
  zero_copy_enabled: true
  batch_size: 1  # No batching
  immediate_send: true
  
network:
  send_timeout: 1  # Immediate timeout
  buffer_size: 1500  # Minimal buffering
```

### Thread and Concurrency Tuning

#### Optimal Thread Count

```rust
// Calculate optimal thread count
fn calculate_optimal_threads() -> usize {
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    
    // For CPU-bound workloads: threads = CPU cores
    // For I/O-bound workloads: threads = CPU cores * 2
    
    match workload_type() {
        WorkloadType::CpuBound => cpu_count,
        WorkloadType::IoBound => cpu_count * 2,
        WorkloadType::Mixed => (cpu_count as f32 * 1.5) as usize,
    }
}
```

#### Thread Affinity

```bash
# Pin threads to specific CPU cores
# Core 0-3: Worker threads
# Core 4: Statistics thread
# Core 5: Monitoring thread
# Core 6-7: System processes

taskset -c 0-3 router-flood --config production.yaml
```

### Memory Optimization

#### Buffer Pool Tuning

```yaml
performance:
  # Buffer pool configuration
  buffer_pool_size: 1000  # Number of buffers
  buffer_size: 1500       # Size per buffer
  pool_growth_factor: 1.5 # Growth when exhausted
  max_pools: 10          # Maximum pools per worker
  
  # Memory allocation strategy
  allocation_strategy: "pool_first"  # pool_first, direct, hybrid
  large_packet_threshold: 1400
  
  # Garbage collection tuning
  gc_interval: 60        # Seconds between cleanup
  gc_threshold: 0.8      # Trigger when 80% full
```

#### Memory Monitoring

```rust
// Monitor memory usage
pub struct MemoryMonitor {
    peak_usage: AtomicU64,
    current_usage: AtomicU64,
    allocation_count: AtomicU64,
}

impl MemoryMonitor {
    pub fn track_allocation(&self, size: usize) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        let current = self.current_usage.fetch_add(size as u64, Ordering::Relaxed);
        
        // Update peak if necessary
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak, current, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
    }
}
```

### Network Performance

#### Interface Optimization

```bash
# Optimize network interface settings
ethtool -G eth0 rx 4096 tx 4096  # Increase ring buffer size
ethtool -C eth0 rx-usecs 50      # Interrupt coalescing
ethtool -K eth0 gso on           # Generic segmentation offload
ethtool -K eth0 tso on           # TCP segmentation offload
ethtool -K eth0 gro on           # Generic receive offload

# Check current settings
ethtool -g eth0  # Ring buffer settings
ethtool -c eth0  # Coalescing settings
ethtool -k eth0  # Offload settings
```

#### Packet Sending Optimization

```rust
// Optimized packet sending with batching
pub struct BatchedSender {
    batch_size: usize,
    batch_buffer: Vec<Vec<u8>>,
    send_count: usize,
}

impl BatchedSender {
    pub fn send_packet(&mut self, packet: Vec<u8>) -> Result<()> {
        self.batch_buffer.push(packet);
        
        if self.batch_buffer.len() >= self.batch_size {
            self.flush_batch()?;
        }
        
        Ok(())
    }
    
    fn flush_batch(&mut self) -> Result<()> {
        // Send all packets in batch
        for packet in self.batch_buffer.drain(..) {
            self.send_single_packet(&packet)?;
        }
        Ok(())
    }
}
```

## Performance Monitoring

### Real-Time Metrics

#### Key Performance Indicators

```rust
// Performance metrics to track
pub struct PerformanceMetrics {
    // Throughput metrics
    pub packets_per_second: f64,
    pub bytes_per_second: f64,
    pub megabits_per_second: f64,
    
    // Latency metrics
    pub avg_packet_build_time: Duration,
    pub avg_send_time: Duration,
    pub p95_total_time: Duration,
    pub p99_total_time: Duration,
    
    // Resource utilization
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub buffer_pool_utilization: f64,
    
    // Error rates
    pub success_rate_percent: f64,
    pub error_rate_per_second: f64,
}
```

#### Performance Dashboard

```bash
# Enable real-time performance dashboard
router-flood --config production.yaml --dashboard --performance-mode

# Dashboard shows:
# ┌─ Performance Dashboard ─────────────────────────────┐
# │ Throughput: 15,432 pps | 187.2 Mbps                │
# │ Latency: avg=0.8ms p95=1.2ms p99=2.1ms             │
# │ CPU: 67% | Memory: 234MB | Buffers: 78%             │
# │ Success Rate: 99.7% | Errors: 0.3/s                │
# │                                                     │
# │ Hot Spots:                                          │
# │ • packet_build: 45% CPU                            │
# │ • network_send: 32% CPU                            │
# │ • stats_update: 8% CPU                             │
# └─────────────────────────────────────────────────────┘
```

### Profiling and Analysis

#### CPU Profiling

```bash
# Install profiling tools
sudo apt install linux-tools-generic valgrind

# Profile with perf
sudo perf record -g --call-graph dwarf ./router-flood --config test.yaml
sudo perf report --stdio

# Profile with valgrind
valgrind --tool=callgrind ./router-flood --config test.yaml
kcachegrind callgrind.out.*
```

#### Memory Profiling

```bash
# Memory leak detection
valgrind --tool=memcheck --leak-check=full ./router-flood --config test.yaml

# Heap profiling
valgrind --tool=massif ./router-flood --config test.yaml
ms_print massif.out.*
```

#### Network Analysis

```bash
# Monitor network performance
iftop -i eth0                    # Real-time bandwidth usage
nethogs                          # Per-process network usage
ss -tuln                         # Socket statistics
netstat -i                       # Interface statistics

# Packet capture for analysis
tcpdump -i eth0 -w capture.pcap -c 10000
wireshark capture.pcap
```

## Benchmarking

### Performance Benchmarks

#### Throughput Benchmark

```bash
# Run throughput benchmark
cargo bench throughput

# Results:
# packet_building/udp_zero_copy    time: [68.2 ns 69.1 ns 70.3 ns]
# packet_building/udp_allocation   time: [95.4 ns 97.2 ns 99.1 ns]
# buffer_pool/get_return          time: [12.1 ns 12.4 ns 12.8 ns]
# protocol_selection/ipv4         time: [8.7 ns 9.1 ns 9.6 ns]
```

#### Latency Benchmark

```bash
# Run latency benchmark
cargo bench latency

# Results:
# end_to_end_latency              time: [1.2 ms 1.3 ms 1.4 ms]
# packet_build_latency            time: [0.8 ms 0.9 ms 1.0 ms]
# send_latency                    time: [0.3 ms 0.4 ms 0.5 ms]
```

#### Stress Testing

```bash
# High-load stress test
router-flood --config stress-test.yaml --duration 3600

# Configuration for stress test:
# - 32 threads
# - 50,000 pps per thread
# - 1 hour duration
# - All protocol types
# - Maximum packet sizes
```

### Performance Regression Testing

```bash
# Automated performance testing
#!/bin/bash
# performance-test.sh

BASELINE_PPS=10000
CURRENT_PPS=$(router-flood --config benchmark.yaml --duration 60 | grep "Average PPS" | awk '{print $3}')

PERFORMANCE_RATIO=$(echo "scale=2; $CURRENT_PPS / $BASELINE_PPS" | bc)

if (( $(echo "$PERFORMANCE_RATIO < 0.95" | bc -l) )); then
    echo "❌ Performance regression detected: ${PERFORMANCE_RATIO}x baseline"
    exit 1
else
    echo "✅ Performance acceptable: ${PERFORMANCE_RATIO}x baseline"
fi
```

## Optimization Strategies

### Hot Path Optimization

#### Identify Hot Paths

```bash
# Use perf to identify hot functions
sudo perf record -g ./router-flood --config test.yaml
sudo perf report --sort=overhead --stdio | head -20

# Common hot paths:
# 1. packet_builder::build_packet_into_buffer
# 2. rng::BatchedRng::port
# 3. transport::send_packet
# 4. stats::increment_sent
```

#### Optimize Hot Functions

```rust
// Before optimization
pub fn build_packet(&mut self) -> Result<Vec<u8>> {
    let mut packet = Vec::new();
    // ... build packet
    Ok(packet)
}

// After optimization with inline hints
#[inline(always)]
pub fn build_packet_into_buffer(&mut self, buffer: &mut [u8]) -> Result<usize> {
    // Zero-copy implementation
    // ... build directly into buffer
    Ok(packet_size)
}
```

### Memory Access Optimization

#### Cache-Friendly Data Structures

```rust
// Cache-aligned structures
#[repr(align(64))]  // Cache line alignment
pub struct CacheAlignedStats {
    pub packets_sent: AtomicU64,
    pub packets_failed: AtomicU64,
    pub bytes_sent: AtomicU64,
    // Pad to cache line boundary
    _padding: [u8; 40],
}

// Structure of arrays for better cache locality
pub struct PacketBatch {
    pub types: Vec<PacketType>,      // All types together
    pub sizes: Vec<usize>,           // All sizes together
    pub timestamps: Vec<Instant>,    // All timestamps together
}
```

#### Memory Pool Optimization

```rust
// Lock-free memory pool with thread-local caches
pub struct OptimizedBufferPool {
    global_pool: Arc<LockFreeQueue<Buffer>>,
    thread_local_cache: ThreadLocal<RefCell<Vec<Buffer>>>,
    cache_size: usize,
}

impl OptimizedBufferPool {
    pub fn get_buffer(&self) -> Option<Buffer> {
        // Try thread-local cache first
        if let Some(buffer) = self.try_local_cache() {
            return Some(buffer);
        }
        
        // Fall back to global pool
        self.global_pool.pop()
    }
}
```

## Troubleshooting Performance Issues

### Common Performance Problems

#### High CPU Usage

**Symptoms**:
- CPU usage > 90%
- Reduced packet throughput
- Increased latency

**Diagnosis**:
```bash
# Check CPU usage per thread
top -H -p $(pgrep router-flood)

# Profile CPU usage
sudo perf top -p $(pgrep router-flood)
```

**Solutions**:
1. Reduce thread count
2. Lower packet rate
3. Disable unnecessary features
4. Optimize hot paths

#### Memory Leaks

**Symptoms**:
- Continuously increasing memory usage
- System becomes unresponsive
- Out of memory errors

**Diagnosis**:
```bash
# Monitor memory growth
watch -n 1 'ps aux | grep router-flood | grep -v grep'

# Check for memory leaks
valgrind --tool=memcheck --leak-check=full ./router-flood
```

**Solutions**:
1. Enable buffer pool recycling
2. Reduce buffer pool size
3. Implement memory limits
4. Fix memory leaks in code

#### Network Bottlenecks

**Symptoms**:
- Low packet throughput despite low CPU
- High network interface utilization
- Packet drops

**Diagnosis**:
```bash
# Check interface statistics
cat /proc/net/dev
ethtool -S eth0

# Monitor packet drops
netstat -i
```

**Solutions**:
1. Increase network buffer sizes
2. Use multiple network interfaces
3. Optimize packet batching
4. Reduce packet size

### Performance Tuning Checklist

#### System Level
- [ ] Network buffers optimized
- [ ] CPU governor set to performance
- [ ] Memory limits configured
- [ ] Interrupt affinity set
- [ ] Huge pages enabled (if needed)

#### Application Level
- [ ] Thread count optimized
- [ ] Buffer pool sized correctly
- [ ] Zero-copy enabled
- [ ] Batching configured
- [ ] Monitoring overhead minimized

#### Network Level
- [ ] Interface settings optimized
- [ ] Ring buffers increased
- [ ] Offloading enabled
- [ ] Interrupt coalescing tuned

#### Monitoring
- [ ] Performance metrics enabled
- [ ] Alerting configured
- [ ] Profiling tools available
- [ ] Baseline measurements taken

## Advanced Optimization Techniques

### SIMD Optimization

```rust
// Use SIMD for packet processing
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline]
pub fn checksum_simd(data: &[u8]) -> u16 {
    unsafe {
        // Use SSE2 for faster checksum calculation
        let mut sum = _mm_setzero_si128();
        
        for chunk in data.chunks_exact(16) {
            let bytes = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            sum = _mm_add_epi16(sum, bytes);
        }
        
        // Horizontal sum and return
        let sum_array: [u16; 8] = std::mem::transmute(sum);
        sum_array.iter().sum()
    }
}
```

### Zero-Copy Networking

```rust
// Zero-copy packet transmission
pub struct ZeroCopyTransport {
    socket: RawSocket,
    send_buffer: MmapBuffer,  // Memory-mapped buffer
}

impl ZeroCopyTransport {
    pub fn send_zero_copy(&mut self, packet: &[u8]) -> Result<()> {
        // Write directly to memory-mapped buffer
        self.send_buffer.write_at(0, packet)?;
        
        // Send without copying
        self.socket.send_from_mmap(&self.send_buffer, packet.len())
    }
}
```

### Lock-Free Data Structures

```rust
// Lock-free statistics collection
pub struct LockFreeStats {
    counters: [AtomicU64; 16],  // Cache-line aligned
}

impl LockFreeStats {
    #[inline(always)]
    pub fn increment(&self, counter_id: usize) {
        self.counters[counter_id].fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_total(&self) -> u64 {
        self.counters.iter()
            .map(|c| c.load(Ordering::Relaxed))
            .sum()
    }
}
```

---

This performance tuning guide provides comprehensive optimization strategies for achieving maximum performance with the router-flood tool while maintaining stability and reliability.