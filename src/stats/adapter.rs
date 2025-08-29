//! Adapter to bridge lock-free stats with existing FloodStats interface

use super::{FloodStats, lockfree::{LockFreeStats, ProtocolId}};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::collections::HashMap;
use uuid::Uuid;
use crate::config::ExportConfig;
use crate::constants::protocols;

/// Adapter that wraps lock-free stats to maintain backward compatibility
pub struct LockFreeStatsAdapter {
    /// Internal lock-free stats
    inner: Arc<LockFreeStats>,
    /// Session ID for compatibility
    pub session_id: String,
    /// Export config for compatibility
    pub export_config: Option<ExportConfig>,
}

impl LockFreeStatsAdapter {
    pub fn new(export_config: Option<ExportConfig>) -> Self {
        Self {
            inner: Arc::new(LockFreeStats::new()),
            session_id: Uuid::new_v4().to_string(),
            export_config,
        }
    }
    
    /// Get the internal lock-free stats
    pub fn inner(&self) -> Arc<LockFreeStats> {
        self.inner.clone()
    }
    
    /// Convert to FloodStats for compatibility
    pub fn to_flood_stats(&self) -> FloodStats {
        let snapshot = self.inner.snapshot();
        
        let protocol_stats = HashMap::from([
            (protocols::UDP.to_string(), AtomicU64::new(snapshot.protocol_counts[ProtocolId::Udp as usize])),
            (protocols::TCP.to_string(), AtomicU64::new(snapshot.protocol_counts[ProtocolId::Tcp as usize])),
            (protocols::ICMP.to_string(), AtomicU64::new(snapshot.protocol_counts[ProtocolId::Icmp as usize])),
            (protocols::IPV6.to_string(), AtomicU64::new(snapshot.protocol_counts[ProtocolId::Ipv6 as usize])),
            (protocols::ARP.to_string(), AtomicU64::new(snapshot.protocol_counts[ProtocolId::Arp as usize])),
        ]);
        
        FloodStats {
            packets_sent: Arc::new(AtomicU64::new(snapshot.packets_sent)),
            packets_failed: Arc::new(AtomicU64::new(snapshot.packets_failed)),
            bytes_sent: Arc::new(AtomicU64::new(snapshot.bytes_sent)),
            start_time: self.inner.start_time,
            session_id: self.session_id.clone(),
            protocol_stats: Arc::new(protocol_stats),
            export_config: self.export_config.clone(),
        }
    }
    
    /// Increment sent packet using protocol string for compatibility
    pub fn increment_sent(&self, bytes: u64, protocol: &str) {
        if let Some(protocol_id) = ProtocolId::from_str(protocol) {
            self.inner.increment_sent(bytes, protocol_id);
        }
    }
    
    /// Increment failed packet
    pub fn increment_failed(&self) {
        self.inner.increment_failed();
    }
}

/// Extension trait to add lock-free support to LocalStats
pub trait LocalStatsExt {
    fn with_lock_free(stats: Arc<LockFreeStats>, batch_size: usize) -> Self;
}

impl LocalStatsExt for super::LocalStats {
    fn with_lock_free(stats: Arc<LockFreeStats>, batch_size: usize) -> Self {
        let adapter = LockFreeStatsAdapter {
            inner: stats,
            session_id: Uuid::new_v4().to_string(),
            export_config: None,
        };
        
        let flood_stats = adapter.to_flood_stats();
        Self::new(Arc::new(flood_stats), batch_size)
    }
}