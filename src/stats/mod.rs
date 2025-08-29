//! Statistics collection and export system
//!
//! This module provides a trait-based architecture for statistics collection
//! with support for different collection strategies and export formats.

pub mod adapter;
pub mod collector;
pub mod export;
pub mod local;
pub mod lockfree;
pub mod observer;

pub use adapter::{LockFreeStatsAdapter, LocalStatsExt};
pub use collector::{StatsCollector, SessionStats, SystemStats};
pub use export::StatsExporter;
pub use local::LocalStats;
pub use lockfree::{LockFreeStats, LockFreeLocalStats, PerCpuStats, ProtocolId, StatsSnapshot};
pub use observer::{StatsObserver, StatsSubject, StatsEvent, ObserverBuilder};

use chrono::Utc;
use csv::Writer;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

use crate::config::{ExportConfig, ExportFormat};
use crate::constants::{protocols, stats as stats_constants, STATS_EXPORT_DIR};
use crate::error::{Result, StatsError};

/// Enhanced statistics tracking with export capabilities
pub struct FloodStats {
    pub packets_sent: Arc<AtomicU64>,
    pub packets_failed: Arc<AtomicU64>,
    pub bytes_sent: Arc<AtomicU64>,
    pub start_time: Instant,
    pub session_id: String,
    pub protocol_stats: Arc<HashMap<String, AtomicU64>>,
    pub export_config: Option<ExportConfig>,
}

impl Default for FloodStats {
    fn default() -> Self {
        let protocol_stats = Self::init_protocol_stats();
        Self {
            packets_sent: Arc::new(AtomicU64::new(0)),
            packets_failed: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            session_id: Uuid::new_v4().to_string(),
            protocol_stats: Arc::new(protocol_stats),
            export_config: None,
        }
    }
}

impl FloodStats {
    pub fn new(export_config: Option<ExportConfig>) -> Self {
        let protocol_stats = Self::init_protocol_stats();
        
        Self {
            start_time: Instant::now(),
            session_id: Uuid::new_v4().to_string(),
            protocol_stats: Arc::new(protocol_stats),
            export_config,
            packets_sent: Arc::new(AtomicU64::new(0)),
            packets_failed: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Initialize protocol statistics map with all supported protocols
    fn init_protocol_stats() -> HashMap<String, AtomicU64> {
        protocols::ALL_PROTOCOLS
            .iter()
            .map(|&protocol| (protocol.to_string(), AtomicU64::new(0)))
            .collect()
    }

    pub fn increment_sent(&self, bytes: u64, protocol: &str) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);

        if let Some(counter) = self.protocol_stats.get(protocol) {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn increment_failed(&self) {
        self.packets_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn print_stats(&self, system_stats: Option<&SystemStats>) {
        let sent = self.packets_sent.load(Ordering::Relaxed);
        let failed = self.packets_failed.load(Ordering::Relaxed);
        let bytes = self.bytes_sent.load(Ordering::Relaxed);

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let pps = sent as f64 / elapsed;
        let mbps = (bytes as f64 * 8.0) / (elapsed * stats_constants::MEGABITS_DIVISOR);

        println!(
            "📊 Stats - Sent: {}, Failed: {}, Rate: {:.1} pps, {:.2} Mbps",
            sent, failed, pps, mbps
        );

        // Protocol breakdown
        for (protocol, counter) in self.protocol_stats.iter() {
            let count = counter.load(Ordering::Relaxed);
            if count > 0 {
                println!("   {}: {} packets", protocol, count);
            }
        }

        // System stats if available
        if let Some(sys_stats) = system_stats {
            println!(
                "   System: CPU {:.1}%, Memory: {:.1} MB",
                sys_stats.cpu_usage,
                sys_stats.memory_usage / stats_constants::BYTES_TO_MB_DIVISOR
            );
        }
    }

    pub async fn export_stats(&self, system_stats: Option<&SystemStats>) -> Result<()> {
        if let Some(export_config) = &self.export_config {
            if !export_config.enabled {
                return Ok(());
            }

            let stats = self.get_session_stats(system_stats);

            // Ensure export directory exists
            fs::create_dir_all(STATS_EXPORT_DIR)
                .await
                .map_err(|e| StatsError::ExportFailed(format!("Failed to create export directory: {}", e)))?;

            match export_config.format {
                ExportFormat::Json => {
                    self.export_json(&stats, export_config).await?;
                }
                ExportFormat::Csv => {
                    self.export_csv(&stats, export_config).await?;
                }
                ExportFormat::Both => {
                    self.export_json(&stats, export_config).await?;
                    self.export_csv(&stats, export_config).await?;
                }
            }
        }
        Ok(())
    }

    fn get_session_stats(&self, system_stats: Option<&SystemStats>) -> SessionStats {
        let sent = self.packets_sent.load(Ordering::Relaxed);
        let failed = self.packets_failed.load(Ordering::Relaxed);
        let bytes = self.bytes_sent.load(Ordering::Relaxed);

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let pps = sent as f64 / elapsed;
        let mbps = (bytes as f64 * 8.0) / (elapsed * stats_constants::MEGABITS_DIVISOR);

        let protocol_breakdown: HashMap<String, u64> = self
            .protocol_stats
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();

        SessionStats {
            session_id: self.session_id.clone(),
            timestamp: Utc::now(),
            packets_sent: sent,
            packets_failed: failed,
            bytes_sent: bytes,
            duration_secs: elapsed,
            packets_per_second: pps,
            megabits_per_second: mbps,
            protocol_breakdown,
            system_stats: system_stats.cloned(),
        }
    }

    async fn export_json(
        &self,
        stats: &SessionStats,
        config: &ExportConfig,
    ) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/{}_stats_{}.json",
            STATS_EXPORT_DIR, config.filename_pattern, timestamp
        );

        let json = serde_json::to_string_pretty(stats)
            .map_err(|e| StatsError::SerializationError(format!("Failed to serialize stats: {}", e)))?;

        fs::write(&filename, json)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write JSON stats: {}", e)))?;

        info!("Stats exported to {}", filename);
        Ok(())
    }

    async fn export_csv(
        &self,
        stats: &SessionStats,
        config: &ExportConfig,
    ) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/{}_stats_{}.csv",
            STATS_EXPORT_DIR, config.filename_pattern, timestamp
        );

        let file = std::fs::File::create(&filename)
            .map_err(|e| StatsError::FileWriteError(format!("Failed to create CSV file: {}", e)))?;

        let mut writer = Writer::from_writer(file);

        // Write header
        writer
            .write_record(&[
                "session_id",
                "timestamp",
                "packets_sent",
                "packets_failed",
                "bytes_sent",
                "duration_secs",
                "packets_per_second",
                "megabits_per_second",
                "udp_packets",
                "tcp_packets",
                "icmp_packets",
                "ipv6_packets",
                "arp_packets",
            ])
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write CSV header: {}", e)))?;

        // Write data - using constants for protocol names
        writer
            .write_record(&[
                &stats.session_id,
                &stats.timestamp.to_rfc3339(),
                &stats.packets_sent.to_string(),
                &stats.packets_failed.to_string(),
                &stats.bytes_sent.to_string(),
                &stats.duration_secs.to_string(),
                &stats.packets_per_second.to_string(),
                &stats.megabits_per_second.to_string(),
                &stats.protocol_breakdown.get(protocols::UDP).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(protocols::TCP).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(protocols::ICMP).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(protocols::IPV6).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(protocols::ARP).unwrap_or(&0).to_string(),
            ])
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write CSV data: {}", e)))?;

        writer
            .flush()
            .map_err(|e| StatsError::FileWriteError(format!("Failed to flush CSV: {}", e)))?;
        info!("Stats exported to {}", filename);
        Ok(())
    }
}