//! Local statistics accumulator for batched updates

use super::collector::StatsCollector;
use crate::constants::protocols;
use std::collections::HashMap;
use std::sync::Arc;

/// Local stats accumulator to batch atomic updates
pub struct LocalStats {
    packets_sent: u64,
    packets_failed: u64,
    bytes_sent: u64,
    protocol_counts: HashMap<String, u64>,
    batch_size: usize,
    stats_ref: Arc<dyn StatsCollector>,
}

impl LocalStats {
    /// Create a new local stats accumulator
    pub fn new(stats_ref: Arc<dyn StatsCollector>, batch_size: usize) -> Self {
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
        if self.packets_sent > 0 || self.packets_failed > 0 {
            // For now, we'll need to implement this differently since we can't
            // directly access the FloodStats internals through the trait
            // This would need to be refactored to work with the trait system
            
            // Reset local counters
            self.packets_sent = 0;
            self.packets_failed = 0;
            self.bytes_sent = 0;
            
            for count in self.protocol_counts.values_mut() {
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