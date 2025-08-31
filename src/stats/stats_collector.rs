//! High-performance statistics collector using lock-free data structures
//!
//! This module provides the primary statistics collection interface with
//! high-performance lock-free implementation for minimal contention.

use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;
use chrono::Utc;
use csv::Writer;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

use crate::config::{ExportConfig, ExportFormat};
use crate::constants::{stats as stats_constants, STATS_EXPORT_DIR};
use crate::error::{Result, StatsError};
use crate::performance::lockfree_stats::{LockFreeStatsCollector, StatsSnapshot};
use super::collector::{SessionStats, SystemStats};
use super::display::{get_display};

/// High-performance packet flood statistics tracker using lock-free implementation
pub struct FloodStatsTracker {
    collector: Arc<LockFreeStatsCollector>,
    pub start_time: Instant,
    pub session_id: String,
    pub export_config: Option<ExportConfig>,
}

impl Default for FloodStatsTracker {
    fn default() -> Self {
        Self {
            collector: Arc::new(LockFreeStatsCollector::new()),
            start_time: Instant::now(),
            session_id: Uuid::new_v4().to_string(),
            export_config: None,
        }
    }
}

impl FloodStatsTracker {
    /// Create a new high-performance stats collector
    pub fn new(export_config: Option<ExportConfig>) -> Self {
        Self {
            collector: Arc::new(LockFreeStatsCollector::new()),
            start_time: Instant::now(),
            session_id: Uuid::new_v4().to_string(),
            export_config,
        }
    }

    /// Record a sent packet (compatible with FloodStats API)
    pub fn increment_sent(&self, bytes: u64, protocol: &str) {
        self.collector.record_sent(protocol, bytes as usize);
    }

    /// Record a failed packet (compatible with FloodStats API)
    pub fn increment_failed(&self) {
        self.collector.record_failed();
    }

    /// Get current statistics snapshot
    pub fn snapshot(&self) -> StatsSnapshot {
        self.collector.aggregate()
    }

    /// Get packets sent count
    pub fn packets_sent(&self) -> u64 {
        self.snapshot().packets_sent
    }

    /// Get packets failed count
    pub fn packets_failed(&self) -> u64 {
        self.snapshot().packets_failed
    }

    /// Get bytes sent
    pub fn bytes_sent(&self) -> u64 {
        self.snapshot().bytes_sent
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.collector.reset();
    }

    /// Get reference to the internal collector
    pub fn collector(&self) -> &Arc<LockFreeStatsCollector> {
        &self.collector
    }

    /// Print statistics to console
    pub fn print_stats(&self, system_stats: Option<&SystemStats>) {
        let snapshot = self.snapshot();
        let elapsed = self.start_time.elapsed().as_secs_f64();
        
        // Try to use in-place display if available
        if let Some(display) = get_display() {
            // Convert snapshot to protocol_stats HashMap for compatibility
            use std::sync::atomic::AtomicU64;
            let mut protocol_stats = HashMap::new();
            protocol_stats.insert("UDP".to_string(), AtomicU64::new(snapshot.udp_packets));
            protocol_stats.insert("TCP".to_string(), AtomicU64::new(snapshot.tcp_packets));
            protocol_stats.insert("ICMP".to_string(), AtomicU64::new(snapshot.icmp_packets));
            protocol_stats.insert("IPv6".to_string(), AtomicU64::new(snapshot.ipv6_packets));
            protocol_stats.insert("ARP".to_string(), AtomicU64::new(snapshot.arp_packets));
            
            display.display_stats(
                snapshot.packets_sent,
                snapshot.packets_failed,
                snapshot.bytes_sent,
                elapsed,
                &Arc::new(protocol_stats),
                system_stats,
            );
        } else {
            // Fallback to regular printing
            let pps = snapshot.packets_sent as f64 / elapsed;
            let mbps = (snapshot.bytes_sent as f64 * 8.0) / (elapsed * stats_constants::MEGABITS_DIVISOR);

            println!(
                "📊 Stats - Sent: {}, Failed: {}, Rate: {:.1} pps, {:.2} Mbps",
                snapshot.packets_sent, snapshot.packets_failed, pps, mbps
            );

            // Protocol breakdown
            if snapshot.udp_packets > 0 {
                println!("   UDP: {} packets", snapshot.udp_packets);
            }
            if snapshot.tcp_packets > 0 {
                println!("   TCP: {} packets", snapshot.tcp_packets);
            }
            if snapshot.icmp_packets > 0 {
                println!("   ICMP: {} packets", snapshot.icmp_packets);
            }
            if snapshot.ipv6_packets > 0 {
                println!("   IPv6: {} packets", snapshot.ipv6_packets);
            }
            if snapshot.arp_packets > 0 {
                println!("   ARP: {} packets", snapshot.arp_packets);
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
    }

    /// Export statistics to configured format
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
        let snapshot = self.snapshot();
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let pps = snapshot.packets_sent as f64 / elapsed;
        let mbps = (snapshot.bytes_sent as f64 * 8.0) / (elapsed * stats_constants::MEGABITS_DIVISOR);

        let mut protocol_breakdown = HashMap::new();
        protocol_breakdown.insert("UDP".to_string(), snapshot.udp_packets);
        protocol_breakdown.insert("TCP".to_string(), snapshot.tcp_packets);
        protocol_breakdown.insert("ICMP".to_string(), snapshot.icmp_packets);
        protocol_breakdown.insert("IPv6".to_string(), snapshot.ipv6_packets);
        protocol_breakdown.insert("ARP".to_string(), snapshot.arp_packets);

        SessionStats {
            session_id: self.session_id.clone(),
            timestamp: Utc::now(),
            packets_sent: snapshot.packets_sent,
            packets_failed: snapshot.packets_failed,
            bytes_sent: snapshot.bytes_sent,
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
            .write_record([
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

        // Write data
        writer
            .write_record([
                &stats.session_id,
                &stats.timestamp.to_rfc3339(),
                &stats.packets_sent.to_string(),
                &stats.packets_failed.to_string(),
                &stats.bytes_sent.to_string(),
                &stats.duration_secs.to_string(),
                &stats.packets_per_second.to_string(),
                &stats.megabits_per_second.to_string(),
                &stats.protocol_breakdown.get("UDP").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("TCP").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("ICMP").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("IPv6").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("ARP").unwrap_or(&0).to_string(),
            ])
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write CSV data: {}", e)))?;

        writer
            .flush()
            .map_err(|e| StatsError::FileWriteError(format!("Failed to flush CSV: {}", e)))?;
        
        info!("Stats exported to {}", filename);
        Ok(())
    }
}

/// Clone implementation for FloodStatsTracker
impl Clone for FloodStatsTracker {
    fn clone(&self) -> Self {
        Self {
            collector: self.collector.clone(),
            start_time: self.start_time,
            session_id: self.session_id.clone(),
            export_config: self.export_config.clone(),
        }
    }
}