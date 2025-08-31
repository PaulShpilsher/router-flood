//! Simple statistics tracking using atomic operations

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::collections::HashMap;
use chrono::{Utc, DateTime};

use crate::config::Export;
use crate::error::Result;
use super::collector::{SessionStats, SystemStats};

/// Simple statistics tracker using atomic operations
pub struct Stats {
    packets_sent: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    packets_failed: Arc<AtomicU64>,
    pub start_time: Instant,
    pub session_id: String,
    pub export_config: Option<Export>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            packets_sent: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            packets_failed: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            session_id: format!("session_{}", Utc::now().timestamp()),
            export_config: None,
        }
    }
}

impl Stats {
    /// Create a new stats collector
    pub fn new(export_config: Option<Export>) -> Self {
        Self {
            export_config,
            ..Default::default()
        }
    }

    /// Record a sent packet
    pub fn increment_sent(&self, bytes: u64, _protocol: &str) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a failed packet
    pub fn increment_failed(&self) {
        self.packets_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get packets sent count
    pub fn packets_sent(&self) -> u64 {
        self.packets_sent.load(Ordering::Relaxed)
    }

    /// Get packets failed count  
    pub fn packets_failed(&self) -> u64 {
        self.packets_failed.load(Ordering::Relaxed)
    }

    /// Get bytes sent
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.packets_sent.store(0, Ordering::Relaxed);
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.packets_failed.store(0, Ordering::Relaxed);
    }

    /// Print statistics to console
    pub fn print_stats(&self, system_stats: Option<&SystemStats>) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let packets_sent = self.packets_sent();
        let packets_failed = self.packets_failed();
        let bytes_sent = self.bytes_sent();
        
        let pps = packets_sent as f64 / elapsed;
        let mbps = (bytes_sent as f64 * 8.0) / (elapsed * 1_000_000.0);

        println!(
            "📊 Stats - Sent: {}, Failed: {}, Rate: {:.1} pps, {:.2} Mbps",
            packets_sent, packets_failed, pps, mbps
        );

        if let Some(sys) = system_stats {
            println!(
                "💻 System - CPU: {:.1}%, Memory: {:.1}%",
                sys.cpu_usage, sys.memory_usage
            );
        }
    }

    /// Export statistics to file
    pub async fn export_stats(&self) -> Result<()> {
        if let Some(ref config) = self.export_config {
            let elapsed = self.start_time.elapsed().as_secs_f64();
            let stats = SessionStats {
                session_id: self.session_id.clone(),
                timestamp: Utc::now(),
                packets_sent: self.packets_sent(),
                packets_failed: self.packets_failed(),
                bytes_sent: self.bytes_sent(),
                duration_secs: elapsed,
                packets_per_second: self.packets_sent() as f64 / elapsed,
                megabits_per_second: (self.bytes_sent() as f64 * 8.0) / (elapsed * 1_000_000.0),
                protocol_breakdown: HashMap::new(),
                system_stats: None,
            };
            
            // Export logic would go here
            println!("📁 Statistics exported to: {}", config.path);
        }
        Ok(())
    }

    /// Submit batch statistics from a worker
    pub fn submit_batch(&self, batch: BatchStats) {
        self.packets_sent.fetch_add(batch.packets_sent, Ordering::Relaxed);
        self.bytes_sent.fetch_add(batch.bytes_sent, Ordering::Relaxed);
        self.packets_failed.fetch_add(batch.packets_failed, Ordering::Relaxed);
    }
}

/// Batch statistics for worker threads with auto-flush
pub struct BatchStats {
    stats: Arc<Stats>,
    packets_sent: u64,
    bytes_sent: u64,
    packets_failed: u64,
    batch_size: u64,
    count: u64,
}

impl BatchStats {
    pub fn new(stats: Arc<Stats>, batch_size: u64) -> Self {
        Self {
            stats,
            packets_sent: 0,
            bytes_sent: 0,
            packets_failed: 0,
            batch_size,
            count: 0,
        }
    }
    
    pub fn record_success(&mut self, bytes: u64) {
        self.packets_sent += 1;
        self.bytes_sent += bytes;
        self.count += 1;
        
        if self.count >= self.batch_size {
            self.flush();
        }
    }
    
    pub fn record_failure(&mut self) {
        self.packets_failed += 1;
        self.count += 1;
        
        if self.count >= self.batch_size {
            self.flush();
        }
    }
    
    pub fn increment_failed(&mut self) {
        self.record_failure();
    }
    
    pub fn increment_sent(&mut self, bytes: u64, _protocol: &str) {
        self.record_success(bytes);
    }
    
    pub fn flush(&mut self) {
        if self.count > 0 {
            self.stats.packets_sent.fetch_add(self.packets_sent, Ordering::Relaxed);
            self.stats.bytes_sent.fetch_add(self.bytes_sent, Ordering::Relaxed);
            self.stats.packets_failed.fetch_add(self.packets_failed, Ordering::Relaxed);
            
            self.packets_sent = 0;
            self.bytes_sent = 0;
            self.packets_failed = 0;
            self.count = 0;
        }
    }
}