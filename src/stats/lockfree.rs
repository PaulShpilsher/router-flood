//! Lock-free statistics aggregation
//!
//! Provides high-performance, lock-free statistics collection using atomic operations
//! and per-CPU aggregation for optimal cache locality.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Protocol identifier for lock-free stats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ProtocolId {
    Udp = 0,
    Tcp = 1,
    Icmp = 2,
    Ipv6 = 3,
    Arp = 4,
}

impl ProtocolId {
    pub const COUNT: usize = 5;
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "UDP" => Some(Self::Udp),
            "TCP" => Some(Self::Tcp),
            "ICMP" => Some(Self::Icmp),
            "IPv6" => Some(Self::Ipv6),
            "ARP" => Some(Self::Arp),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Udp => "UDP",
            Self::Tcp => "TCP",
            Self::Icmp => "ICMP",
            Self::Ipv6 => "IPv6",
            Self::Arp => "ARP",
        }
    }
}

/// Lock-free statistics structure using array-based protocol tracking
pub struct LockFreeStats {
    /// Total packets sent
    pub packets_sent: AtomicU64,
    /// Total packets failed
    pub packets_failed: AtomicU64,
    /// Total bytes sent
    pub bytes_sent: AtomicU64,
    /// Protocol-specific counters (indexed by ProtocolId)
    pub protocol_counters: [AtomicU64; ProtocolId::COUNT],
    /// Session start time
    pub start_time: Instant,
}

impl Default for LockFreeStats {
    fn default() -> Self {
        Self::new()
    }
}

impl LockFreeStats {
    pub fn new() -> Self {
        Self {
            packets_sent: AtomicU64::new(0),
            packets_failed: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            protocol_counters: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            start_time: Instant::now(),
        }
    }
    
    /// Increment sent packet with optimal memory ordering
    #[inline(always)]
    pub fn increment_sent(&self, bytes: u64, protocol_id: ProtocolId) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
        self.protocol_counters[protocol_id as usize].fetch_add(1, Ordering::Relaxed);
    }
    
    /// Increment failed packet count
    #[inline(always)]
    pub fn increment_failed(&self) {
        self.packets_failed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current stats snapshot
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            packets_sent: self.packets_sent.load(Ordering::Relaxed),
            packets_failed: self.packets_failed.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            protocol_counts: [
                self.protocol_counters[0].load(Ordering::Relaxed),
                self.protocol_counters[1].load(Ordering::Relaxed),
                self.protocol_counters[2].load(Ordering::Relaxed),
                self.protocol_counters[3].load(Ordering::Relaxed),
                self.protocol_counters[4].load(Ordering::Relaxed),
            ],
            elapsed_secs: self.start_time.elapsed().as_secs_f64(),
        }
    }
}

/// Snapshot of statistics at a point in time
#[derive(Debug, Clone)]
pub struct StatsSnapshot {
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub bytes_sent: u64,
    pub protocol_counts: [u64; ProtocolId::COUNT],
    pub elapsed_secs: f64,
}

impl StatsSnapshot {
    pub fn packets_per_second(&self) -> f64 {
        if self.elapsed_secs > 0.0 {
            self.packets_sent as f64 / self.elapsed_secs
        } else {
            0.0
        }
    }
    
    pub fn megabits_per_second(&self) -> f64 {
        if self.elapsed_secs > 0.0 {
            (self.bytes_sent as f64 * 8.0) / (self.elapsed_secs * 1_000_000.0)
        } else {
            0.0
        }
    }
}

/// Per-CPU stats aggregator for better cache locality
pub struct PerCpuStats {
    /// Array of stats, one per CPU
    stats: Vec<Arc<LockFreeStats>>,
    /// Current CPU count
    cpu_count: usize,
    /// Round-robin counter for CPU selection
    next_cpu: AtomicUsize,
}

impl Default for PerCpuStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PerCpuStats {
    pub fn new() -> Self {
        let cpu_count = num_cpus::get().max(1);
        let stats = (0..cpu_count)
            .map(|_| Arc::new(LockFreeStats::new()))
            .collect();
        
        Self {
            stats,
            cpu_count,
            next_cpu: AtomicUsize::new(0),
        }
    }
    
    /// Get stats for current thread (uses round-robin for simplicity)
    pub fn get_local(&self) -> Arc<LockFreeStats> {
        let cpu_id = self.next_cpu.fetch_add(1, Ordering::Relaxed) % self.cpu_count;
        self.stats[cpu_id].clone()
    }
    
    /// Aggregate all CPU stats into a single snapshot
    pub fn aggregate(&self) -> StatsSnapshot {
        let mut total = StatsSnapshot {
            packets_sent: 0,
            packets_failed: 0,
            bytes_sent: 0,
            protocol_counts: [0; ProtocolId::COUNT],
            elapsed_secs: 0.0,
        };
        
        for stats in &self.stats {
            let snapshot = stats.snapshot();
            total.packets_sent += snapshot.packets_sent;
            total.packets_failed += snapshot.packets_failed;
            total.bytes_sent += snapshot.bytes_sent;
            for i in 0..ProtocolId::COUNT {
                total.protocol_counts[i] += snapshot.protocol_counts[i];
            }
            total.elapsed_secs = snapshot.elapsed_secs.max(total.elapsed_secs);
        }
        
        total
    }
}

/// Lock-free local accumulator with automatic batching
pub struct LockFreeLocalStats {
    /// Reference to global stats
    global: Arc<LockFreeStats>,
    /// Local accumulation buffers
    packets_sent: u64,
    packets_failed: u64,
    bytes_sent: u64,
    protocol_counts: [u64; ProtocolId::COUNT],
    /// Batch size for flushing
    batch_size: usize,
    /// Counter for batch tracking
    operation_count: usize,
}

impl LockFreeLocalStats {
    pub fn new(global: Arc<LockFreeStats>, batch_size: usize) -> Self {
        Self {
            global,
            packets_sent: 0,
            packets_failed: 0,
            bytes_sent: 0,
            protocol_counts: [0; ProtocolId::COUNT],
            batch_size,
            operation_count: 0,
        }
    }
    
    /// Increment sent packet locally
    #[inline(always)]
    pub fn increment_sent(&mut self, bytes: u64, protocol_id: ProtocolId) {
        self.packets_sent += 1;
        self.bytes_sent += bytes;
        self.protocol_counts[protocol_id as usize] += 1;
        self.operation_count += 1;
        
        if self.operation_count >= self.batch_size {
            self.flush();
        }
    }
    
    /// Increment failed packet locally
    #[inline(always)]
    pub fn increment_failed(&mut self) {
        self.packets_failed += 1;
        self.operation_count += 1;
        
        if self.operation_count >= self.batch_size {
            self.flush();
        }
    }
    
    /// Flush local stats to global atomics
    pub fn flush(&mut self) {
        if self.packets_sent > 0 {
            self.global.packets_sent.fetch_add(self.packets_sent, Ordering::Relaxed);
            self.packets_sent = 0;
        }
        
        if self.packets_failed > 0 {
            self.global.packets_failed.fetch_add(self.packets_failed, Ordering::Relaxed);
            self.packets_failed = 0;
        }
        
        if self.bytes_sent > 0 {
            self.global.bytes_sent.fetch_add(self.bytes_sent, Ordering::Relaxed);
            self.bytes_sent = 0;
        }
        
        for (i, count) in self.protocol_counts.iter_mut().enumerate() {
            if *count > 0 {
                self.global.protocol_counters[i].fetch_add(*count, Ordering::Relaxed);
                *count = 0;
            }
        }
        
        self.operation_count = 0;
    }
}

impl Drop for LockFreeLocalStats {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_id_conversion() {
        assert_eq!(ProtocolId::from_str("UDP"), Some(ProtocolId::Udp));
        assert_eq!(ProtocolId::from_str("TCP"), Some(ProtocolId::Tcp));
        assert_eq!(ProtocolId::Udp.as_str(), "UDP");
    }
    
    #[test]
    fn test_lock_free_stats_increment() {
        let stats = LockFreeStats::new();
        stats.increment_sent(100, ProtocolId::Udp);
        stats.increment_sent(200, ProtocolId::Tcp);
        stats.increment_failed();
        
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.packets_sent, 2);
        assert_eq!(snapshot.packets_failed, 1);
        assert_eq!(snapshot.bytes_sent, 300);
        assert_eq!(snapshot.protocol_counts[ProtocolId::Udp as usize], 1);
        assert_eq!(snapshot.protocol_counts[ProtocolId::Tcp as usize], 1);
    }
    
    #[test]
    fn test_local_stats_batching() {
        let global = Arc::new(LockFreeStats::new());
        let mut local = LockFreeLocalStats::new(global.clone(), 3);
        
        local.increment_sent(100, ProtocolId::Udp);
        assert_eq!(global.packets_sent.load(Ordering::Relaxed), 0);
        
        local.increment_sent(100, ProtocolId::Udp);
        local.increment_sent(100, ProtocolId::Udp);
        
        assert_eq!(global.packets_sent.load(Ordering::Relaxed), 3);
        assert_eq!(global.bytes_sent.load(Ordering::Relaxed), 300);
    }
    
    #[test]
    fn test_per_cpu_aggregation() {
        let per_cpu = PerCpuStats::new();
        
        let stats1 = per_cpu.get_local();
        stats1.increment_sent(100, ProtocolId::Udp);
        
        let stats2 = per_cpu.get_local();
        stats2.increment_sent(200, ProtocolId::Tcp);
        
        let aggregate = per_cpu.aggregate();
        assert_eq!(aggregate.packets_sent, 2);
        assert_eq!(aggregate.bytes_sent, 300);
    }
}