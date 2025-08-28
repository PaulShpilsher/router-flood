//! Local statistics accumulator for batched updates

use super::FloodStats;
use crate::constants::protocols;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Local stats accumulator to batch atomic updates
pub struct LocalStats {
    packets_sent: u64,
    packets_failed: u64,
    bytes_sent: u64,
    protocol_counts: HashMap<String, u64>,
    batch_size: usize,
    stats_ref: Arc<FloodStats>,
}

impl LocalStats {
    /// Create a new local stats accumulator
    pub fn new(stats_ref: Arc<FloodStats>, batch_size: usize) -> Self {
        let protocol_counts = protocols::ALL_PROTOCOLS
            .iter()
            .map(|&protocol| (protocol.to_string(), 0u64))
            .collect();
            
        Self {
            packets_sent: 0,
            packets_failed: 0,
            bytes_sent: 0,
            protocol_counts,
            batch_size,
            stats_ref,
        }
    }
    
    /// Increment sent packet count locally
    pub fn increment_sent(&mut self, bytes: u64, protocol: &str) {
        self.packets_sent += 1;
        self.bytes_sent += bytes;
        
        if let Some(count) = self.protocol_counts.get_mut(protocol) {
            *count += 1;
        }
        
        // Flush to global stats if batch is full
        if self.packets_sent >= self.batch_size as u64 {
            self.flush();
        }
    }
    
    /// Increment failed packet count locally
    pub fn increment_failed(&mut self) {
        self.packets_failed += 1;
        
        if self.packets_failed >= self.batch_size as u64 {
            self.flush();
        }
    }
    
    /// Flush accumulated stats to global counters
    pub fn flush(&mut self) {
        if self.packets_sent > 0 {
            self.stats_ref.packets_sent.fetch_add(self.packets_sent, Ordering::Relaxed);
            self.packets_sent = 0;
        }
        
        if self.packets_failed > 0 {
            self.stats_ref.packets_failed.fetch_add(self.packets_failed, Ordering::Relaxed);
            self.packets_failed = 0;
        }
        
        if self.bytes_sent > 0 {
            self.stats_ref.bytes_sent.fetch_add(self.bytes_sent, Ordering::Relaxed);
            self.bytes_sent = 0;
        }
        
        for (protocol, count) in &mut self.protocol_counts {
            if *count > 0 {
                if let Some(global_counter) = self.stats_ref.protocol_stats.get(protocol) {
                    global_counter.fetch_add(*count, Ordering::Relaxed);
                }
                *count = 0;
            }
        }
    }
}

impl Drop for LocalStats {
    fn drop(&mut self) {
        // Ensure any remaining stats are flushed when worker terminates
        self.flush();
    }
}