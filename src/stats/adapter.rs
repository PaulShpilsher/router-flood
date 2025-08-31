//! Adapter module for backward compatibility with lock-free stats
//!
//! This module provides compatibility adapters for transitioning to the new
//! high-performance statistics implementation.

use super::{FloodStatsTracker, lockfree::{LockFreeStats, ProtocolId}};
use std::sync::Arc;
use crate::config::ExportConfig;

/// Adapter that wraps lock-free stats to maintain backward compatibility
pub struct LockFreeStatsAdapter {
    /// The underlying FloodStats (FloodStatsTracker)
    stats: Arc<FloodStatsTracker>,
    /// Internal lock-free stats reference (if needed for specialized access)
    inner: Arc<LockFreeStats>,
}

impl LockFreeStatsAdapter {
    pub fn new(export_config: Option<ExportConfig>) -> Self {
        let stats = Arc::new(FloodStatsTracker::new(export_config));
        let inner = Arc::new(LockFreeStats::new());
        
        Self {
            stats,
            inner,
        }
    }
    
    /// Get the internal lock-free stats
    pub fn inner(&self) -> Arc<LockFreeStats> {
        self.inner.clone()
    }
    
    /// Get the FloodStats reference
    pub fn stats(&self) -> Arc<FloodStatsTracker> {
        self.stats.clone()
    }
    
    /// Convert to FloodStats for compatibility
    pub fn to_flood_stats(&self) -> Arc<FloodStatsTracker> {
        self.stats.clone()
    }
    
    /// Increment sent packet using protocol string for compatibility
    pub fn increment_sent(&self, bytes: u64, protocol: &str) {
        self.stats.increment_sent(bytes, protocol);
        
        // Also update internal lock-free stats if needed
        if let Some(protocol_id) = ProtocolId::from_str(protocol) {
            self.inner.increment_sent(bytes, protocol_id);
        }
    }
    
    /// Increment failed packet
    pub fn increment_failed(&self) {
        self.stats.increment_failed();
        self.inner.increment_failed();
    }
}

/// Extension trait to add lock-free support to LocalStats
pub trait LocalStatsExt {
    fn with_lock_free(stats: Arc<LockFreeStats>, batch_size: usize) -> Self;
}

impl LocalStatsExt for super::LocalStats {
    fn with_lock_free(_stats: Arc<LockFreeStats>, batch_size: usize) -> Self {
        // Create a FloodStats instance for the LocalStats to use
        let flood_stats = Arc::new(FloodStatsTracker::default());
        
        Self::new(flood_stats, batch_size)
    }
}