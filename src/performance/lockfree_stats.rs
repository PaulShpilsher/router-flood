//! Lock-free statistics collection with SIMD optimizations
//!
//! This module provides high-performance statistics collection using
//! lock-free data structures and SIMD operations where possible.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::mem;

/// Lock-free per-CPU statistics to minimize contention
#[repr(align(64))] // Cache line alignment to prevent false sharing
pub struct PerCpuStats {
    packets_sent: AtomicU64,
    packets_failed: AtomicU64,
    bytes_sent: AtomicU64,
    // Protocol counters
    udp_packets: AtomicU64,
    tcp_packets: AtomicU64,
    icmp_packets: AtomicU64,
    ipv6_packets: AtomicU64,
    arp_packets: AtomicU64,
    // Padding to ensure cache line alignment
    _padding: [u8; 64 - (8 * mem::size_of::<AtomicU64>()) % 64],
}

impl PerCpuStats {
    /// Create new per-CPU stats
    pub fn new() -> Self {
        Self {
            packets_sent: AtomicU64::new(0),
            packets_failed: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            udp_packets: AtomicU64::new(0),
            tcp_packets: AtomicU64::new(0),
            icmp_packets: AtomicU64::new(0),
            ipv6_packets: AtomicU64::new(0),
            arp_packets: AtomicU64::new(0),
            _padding: [0; 64 - (8 * mem::size_of::<AtomicU64>()) % 64],
        }
    }
    
    /// Record a sent packet
    #[inline(always)]
    pub fn record_sent(&self, protocol: &str, size: usize) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(size as u64, Ordering::Relaxed);
        
        // Update protocol-specific counter
        match protocol {
            "UDP" => self.udp_packets.fetch_add(1, Ordering::Relaxed),
            "TCP" | "TCP-SYN" | "TCP-ACK" => self.tcp_packets.fetch_add(1, Ordering::Relaxed),
            "ICMP" | "IPv6-ICMP" => self.icmp_packets.fetch_add(1, Ordering::Relaxed),
            "IPv6" | "IPv6-UDP" | "IPv6-TCP" => self.ipv6_packets.fetch_add(1, Ordering::Relaxed),
            "ARP" => self.arp_packets.fetch_add(1, Ordering::Relaxed),
            _ => 0, // Unknown protocol
        };
    }
    
    /// Record a failed packet
    #[inline(always)]
    pub fn record_failed(&self) {
        self.packets_failed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get snapshot of current stats
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            packets_sent: self.packets_sent.load(Ordering::Relaxed),
            packets_failed: self.packets_failed.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            udp_packets: self.udp_packets.load(Ordering::Relaxed),
            tcp_packets: self.tcp_packets.load(Ordering::Relaxed),
            icmp_packets: self.icmp_packets.load(Ordering::Relaxed),
            ipv6_packets: self.ipv6_packets.load(Ordering::Relaxed),
            arp_packets: self.arp_packets.load(Ordering::Relaxed),
        }
    }
    
    /// Reset all counters
    pub fn reset(&self) {
        self.packets_sent.store(0, Ordering::Relaxed);
        self.packets_failed.store(0, Ordering::Relaxed);
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.udp_packets.store(0, Ordering::Relaxed);
        self.tcp_packets.store(0, Ordering::Relaxed);
        self.icmp_packets.store(0, Ordering::Relaxed);
        self.ipv6_packets.store(0, Ordering::Relaxed);
        self.arp_packets.store(0, Ordering::Relaxed);
    }
}

impl Default for PerCpuStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of statistics at a point in time
#[derive(Debug, Clone, Copy)]
pub struct StatsSnapshot {
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub bytes_sent: u64,
    pub udp_packets: u64,
    pub tcp_packets: u64,
    pub icmp_packets: u64,
    pub ipv6_packets: u64,
    pub arp_packets: u64,
}

impl StatsSnapshot {
    /// Create an empty snapshot
    pub fn zero() -> Self {
        Self {
            packets_sent: 0,
            packets_failed: 0,
            bytes_sent: 0,
            udp_packets: 0,
            tcp_packets: 0,
            icmp_packets: 0,
            ipv6_packets: 0,
            arp_packets: 0,
        }
    }
    
    /// Add another snapshot to this one
    pub fn add(&mut self, other: &StatsSnapshot) {
        self.packets_sent += other.packets_sent;
        self.packets_failed += other.packets_failed;
        self.bytes_sent += other.bytes_sent;
        self.udp_packets += other.udp_packets;
        self.tcp_packets += other.tcp_packets;
        self.icmp_packets += other.icmp_packets;
        self.ipv6_packets += other.ipv6_packets;
        self.arp_packets += other.arp_packets;
    }
    
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.packets_sent + self.packets_failed;
        if total == 0 {
            100.0
        } else {
            (self.packets_sent as f64 / total as f64) * 100.0
        }
    }
    
    /// Calculate total packets
    pub fn total_packets(&self) -> u64 {
        self.packets_sent + self.packets_failed
    }
}

/// High-performance lock-free statistics collector
pub struct LockFreeStatsCollector {
    per_cpu_stats: Vec<PerCpuStats>,
    cpu_count: usize,
}

impl LockFreeStatsCollector {
    /// Create a new lock-free stats collector
    pub fn new() -> Self {
        let cpu_count = num_cpus::get();
        let mut per_cpu_stats = Vec::with_capacity(cpu_count);
        
        for _ in 0..cpu_count {
            per_cpu_stats.push(PerCpuStats::new());
        }
        
        Self {
            per_cpu_stats,
            cpu_count,
        }
    }
    
    /// Record a sent packet on the current CPU
    #[inline(always)]
    pub fn record_sent(&self, protocol: &str, size: usize) {
        let cpu_id = self.get_cpu_id();
        self.per_cpu_stats[cpu_id].record_sent(protocol, size);
    }
    
    /// Record a failed packet on the current CPU
    #[inline(always)]
    pub fn record_failed(&self) {
        let cpu_id = self.get_cpu_id();
        self.per_cpu_stats[cpu_id].record_failed();
    }
    
    /// Get current CPU ID (approximation)
    #[inline(always)]
    fn get_cpu_id(&self) -> usize {
        // Use a hash of thread ID as a proxy for CPU affinity
        // This is not perfect but avoids expensive syscalls
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        hasher.finish() as usize % self.cpu_count
    }
    
    /// Get aggregated statistics across all CPUs
    pub fn aggregate(&self) -> StatsSnapshot {
        let mut total = StatsSnapshot::zero();
        
        for cpu_stats in &self.per_cpu_stats {
            let snapshot = cpu_stats.snapshot();
            total.add(&snapshot);
        }
        
        total
    }
    
    /// Reset all statistics
    pub fn reset(&self) {
        for cpu_stats in &self.per_cpu_stats {
            cpu_stats.reset();
        }
    }
    
    /// Get per-CPU statistics
    pub fn per_cpu_stats(&self) -> Vec<StatsSnapshot> {
        self.per_cpu_stats.iter().map(|stats| stats.snapshot()).collect()
    }
    
    /// Get CPU count
    pub fn cpu_count(&self) -> usize {
        self.cpu_count
    }
}

impl Default for LockFreeStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Batched statistics collector for reducing atomic operations
pub struct BatchedStatsCollector {
    local_packets_sent: u64,
    local_packets_failed: u64,
    local_bytes_sent: u64,
    local_udp_packets: u64,
    local_tcp_packets: u64,
    local_icmp_packets: u64,
    local_ipv6_packets: u64,
    local_arp_packets: u64,
    batch_size: usize,
    current_batch: usize,
    global_stats: Arc<LockFreeStatsCollector>,
}

impl BatchedStatsCollector {
    /// Create a new batched collector
    pub fn new(global_stats: Arc<LockFreeStatsCollector>, batch_size: usize) -> Self {
        Self {
            local_packets_sent: 0,
            local_packets_failed: 0,
            local_bytes_sent: 0,
            local_udp_packets: 0,
            local_tcp_packets: 0,
            local_icmp_packets: 0,
            local_ipv6_packets: 0,
            local_arp_packets: 0,
            batch_size,
            current_batch: 0,
            global_stats,
        }
    }
    
    /// Record a sent packet locally
    #[inline(always)]
    pub fn record_sent(&mut self, protocol: &str, size: usize) {
        self.local_packets_sent += 1;
        self.local_bytes_sent += size as u64;
        
        match protocol {
            "UDP" => self.local_udp_packets += 1,
            "TCP" | "TCP-SYN" | "TCP-ACK" => self.local_tcp_packets += 1,
            "ICMP" | "IPv6-ICMP" => self.local_icmp_packets += 1,
            "IPv6" | "IPv6-UDP" | "IPv6-TCP" => self.local_ipv6_packets += 1,
            "ARP" => self.local_arp_packets += 1,
            _ => {}
        }
        
        self.current_batch += 1;
        if self.current_batch >= self.batch_size {
            self.flush();
        }
    }
    
    /// Record a failed packet locally
    #[inline(always)]
    pub fn record_failed(&mut self) {
        self.local_packets_failed += 1;
        self.current_batch += 1;
        
        if self.current_batch >= self.batch_size {
            self.flush();
        }
    }
    
    /// Flush local statistics to global collector
    pub fn flush(&mut self) {
        if self.current_batch == 0 {
            return;
        }
        
        let cpu_id = self.get_cpu_id();
        let cpu_stats = &self.global_stats.per_cpu_stats[cpu_id];
        
        // Batch update all counters
        if self.local_packets_sent > 0 {
            cpu_stats.packets_sent.fetch_add(self.local_packets_sent, Ordering::Relaxed);
            self.local_packets_sent = 0;
        }
        
        if self.local_packets_failed > 0 {
            cpu_stats.packets_failed.fetch_add(self.local_packets_failed, Ordering::Relaxed);
            self.local_packets_failed = 0;
        }
        
        if self.local_bytes_sent > 0 {
            cpu_stats.bytes_sent.fetch_add(self.local_bytes_sent, Ordering::Relaxed);
            self.local_bytes_sent = 0;
        }
        
        if self.local_udp_packets > 0 {
            cpu_stats.udp_packets.fetch_add(self.local_udp_packets, Ordering::Relaxed);
            self.local_udp_packets = 0;
        }
        
        if self.local_tcp_packets > 0 {
            cpu_stats.tcp_packets.fetch_add(self.local_tcp_packets, Ordering::Relaxed);
            self.local_tcp_packets = 0;
        }
        
        if self.local_icmp_packets > 0 {
            cpu_stats.icmp_packets.fetch_add(self.local_icmp_packets, Ordering::Relaxed);
            self.local_icmp_packets = 0;
        }
        
        if self.local_ipv6_packets > 0 {
            cpu_stats.ipv6_packets.fetch_add(self.local_ipv6_packets, Ordering::Relaxed);
            self.local_ipv6_packets = 0;
        }
        
        if self.local_arp_packets > 0 {
            cpu_stats.arp_packets.fetch_add(self.local_arp_packets, Ordering::Relaxed);
            self.local_arp_packets = 0;
        }
        
        self.current_batch = 0;
    }
    
    /// Get current CPU ID
    #[inline(always)]
    fn get_cpu_id(&self) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        hasher.finish() as usize % self.global_stats.cpu_count
    }
}

impl Drop for BatchedStatsCollector {
    fn drop(&mut self) {
        self.flush();
    }
}

/// SIMD-optimized statistics aggregation (when available)
mod simd_stats {
    use super::*;
    
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    pub fn aggregate_simd(snapshots: &[StatsSnapshot]) -> StatsSnapshot {
        use std::arch::x86_64::*;
        
        if snapshots.len() < 4 {
            // Fall back to scalar aggregation for small arrays
            return aggregate_scalar(snapshots);
        }
        
        unsafe {
            let mut sum_packets_sent = _mm256_setzero_si256();
            let mut sum_packets_failed = _mm256_setzero_si256();
            let mut sum_bytes_sent = _mm256_setzero_si256();
            
            // Process 4 snapshots at a time
            for chunk in snapshots.chunks(4) {
                if chunk.len() == 4 {
                    let packets_sent = _mm256_set_epi64x(
                        chunk[3].packets_sent as i64,
                        chunk[2].packets_sent as i64,
                        chunk[1].packets_sent as i64,
                        chunk[0].packets_sent as i64,
                    );
                    
                    let packets_failed = _mm256_set_epi64x(
                        chunk[3].packets_failed as i64,
                        chunk[2].packets_failed as i64,
                        chunk[1].packets_failed as i64,
                        chunk[0].packets_failed as i64,
                    );
                    
                    let bytes_sent = _mm256_set_epi64x(
                        chunk[3].bytes_sent as i64,
                        chunk[2].bytes_sent as i64,
                        chunk[1].bytes_sent as i64,
                        chunk[0].bytes_sent as i64,
                    );
                    
                    sum_packets_sent = _mm256_add_epi64(sum_packets_sent, packets_sent);
                    sum_packets_failed = _mm256_add_epi64(sum_packets_failed, packets_failed);
                    sum_bytes_sent = _mm256_add_epi64(sum_bytes_sent, bytes_sent);
                }
            }
            
            // Extract and sum the results
            let mut packets_sent_array = [0i64; 4];
            let mut packets_failed_array = [0i64; 4];
            let mut bytes_sent_array = [0i64; 4];
            
            _mm256_storeu_si256(packets_sent_array.as_mut_ptr() as *mut __m256i, sum_packets_sent);
            _mm256_storeu_si256(packets_failed_array.as_mut_ptr() as *mut __m256i, sum_packets_failed);
            _mm256_storeu_si256(bytes_sent_array.as_mut_ptr() as *mut __m256i, sum_bytes_sent);
            
            let total_packets_sent = packets_sent_array.iter().sum::<i64>() as u64;
            let total_packets_failed = packets_failed_array.iter().sum::<i64>() as u64;
            let total_bytes_sent = bytes_sent_array.iter().sum::<i64>() as u64;
            
            // Handle remaining snapshots and protocol counters with scalar code
            let mut result = StatsSnapshot {
                packets_sent: total_packets_sent,
                packets_failed: total_packets_failed,
                bytes_sent: total_bytes_sent,
                udp_packets: 0,
                tcp_packets: 0,
                icmp_packets: 0,
                ipv6_packets: 0,
                arp_packets: 0,
            };
            
            // Aggregate protocol counters (scalar)
            for snapshot in snapshots {
                result.udp_packets += snapshot.udp_packets;
                result.tcp_packets += snapshot.tcp_packets;
                result.icmp_packets += snapshot.icmp_packets;
                result.ipv6_packets += snapshot.ipv6_packets;
                result.arp_packets += snapshot.arp_packets;
            }
            
            result
        }
    }
    
    #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
    pub fn aggregate_simd(snapshots: &[StatsSnapshot]) -> StatsSnapshot {
        aggregate_scalar(snapshots)
    }
    
    fn aggregate_scalar(snapshots: &[StatsSnapshot]) -> StatsSnapshot {
        let mut result = StatsSnapshot::zero();
        for snapshot in snapshots {
            result.add(snapshot);
        }
        result
    }
}

// Re-export SIMD aggregation function (always available, falls back to scalar)
pub use simd_stats::aggregate_simd;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_per_cpu_stats() {
        let stats = PerCpuStats::new();
        
        stats.record_sent("UDP", 64);
        stats.record_sent("TCP", 128);
        stats.record_failed();
        
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.packets_sent, 2);
        assert_eq!(snapshot.packets_failed, 1);
        assert_eq!(snapshot.bytes_sent, 192);
        assert_eq!(snapshot.udp_packets, 1);
        assert_eq!(snapshot.tcp_packets, 1);
    }
    
    #[test]
    fn test_stats_snapshot() {
        let mut s1 = StatsSnapshot {
            packets_sent: 10,
            packets_failed: 2,
            bytes_sent: 640,
            udp_packets: 5,
            tcp_packets: 5,
            icmp_packets: 0,
            ipv6_packets: 0,
            arp_packets: 0,
        };
        
        let s2 = StatsSnapshot {
            packets_sent: 5,
            packets_failed: 1,
            bytes_sent: 320,
            udp_packets: 3,
            tcp_packets: 2,
            icmp_packets: 0,
            ipv6_packets: 0,
            arp_packets: 0,
        };
        
        s1.add(&s2);
        
        assert_eq!(s1.packets_sent, 15);
        assert_eq!(s1.packets_failed, 3);
        assert_eq!(s1.bytes_sent, 960);
        assert_eq!(s1.udp_packets, 8);
        assert_eq!(s1.tcp_packets, 7);
        assert_eq!(s1.success_rate(), 83.33333333333334);
    }
    
    #[test]
    fn test_lockfree_collector() {
        let collector = LockFreeStatsCollector::new();
        
        collector.record_sent("UDP", 64);
        collector.record_sent("TCP", 128);
        collector.record_failed();
        
        let stats = collector.aggregate();
        assert_eq!(stats.packets_sent, 2);
        assert_eq!(stats.packets_failed, 1);
        assert_eq!(stats.bytes_sent, 192);
    }
    
    #[test]
    fn test_batched_collector() {
        let global = Arc::new(LockFreeStatsCollector::new());
        let mut batched = BatchedStatsCollector::new(global.clone(), 3);
        
        // Record packets (should not flush yet)
        batched.record_sent("UDP", 64);
        batched.record_sent("TCP", 128);
        
        // Global stats should still be zero
        let stats = global.aggregate();
        assert_eq!(stats.packets_sent, 0);
        
        // This should trigger a flush
        batched.record_failed();
        
        let stats = global.aggregate();
        assert_eq!(stats.packets_sent, 2);
        assert_eq!(stats.packets_failed, 1);
        assert_eq!(stats.bytes_sent, 192);
    }
    
    #[test]
    fn test_simd_aggregation() {
        let snapshots = vec![
            StatsSnapshot {
                packets_sent: 10,
                packets_failed: 1,
                bytes_sent: 640,
                udp_packets: 5,
                tcp_packets: 5,
                icmp_packets: 0,
                ipv6_packets: 0,
                arp_packets: 0,
            },
            StatsSnapshot {
                packets_sent: 20,
                packets_failed: 2,
                bytes_sent: 1280,
                udp_packets: 10,
                tcp_packets: 10,
                icmp_packets: 0,
                ipv6_packets: 0,
                arp_packets: 0,
            },
        ];
        
        let result = aggregate_simd(&snapshots);
        assert_eq!(result.packets_sent, 30);
        assert_eq!(result.packets_failed, 3);
        assert_eq!(result.bytes_sent, 1920);
        assert_eq!(result.udp_packets, 15);
        assert_eq!(result.tcp_packets, 15);
    }
}