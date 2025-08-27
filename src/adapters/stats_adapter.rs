//! Statistics type compatibility adapter

use crate::stats::collector::SystemStats as NewSystemStats;
use crate::stats_original::SystemStats as OriginalSystemStats;

/// Adapter to convert between old and new SystemStats structs
pub struct SystemStatsAdapter;

impl SystemStatsAdapter {
    /// Convert from new SystemStats to original SystemStats
    #[inline]
    pub fn to_original(stats: &NewSystemStats) -> OriginalSystemStats {
        OriginalSystemStats {
            cpu_usage: stats.cpu_usage,
            memory_usage: stats.memory_usage,
            memory_total: stats.memory_total,
            network_sent: stats.network_sent,
            network_received: stats.network_received,
        }
    }
    
    /// Convert from original SystemStats to new SystemStats
    #[inline]
    pub fn from_original(stats: &OriginalSystemStats) -> NewSystemStats {
        NewSystemStats {
            cpu_usage: stats.cpu_usage,
            memory_usage: stats.memory_usage,
            memory_total: stats.memory_total,
            network_sent: stats.network_sent,
            network_received: stats.network_received,
        }
    }
}