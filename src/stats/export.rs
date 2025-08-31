//! Statistics export functionality

use super::collector::SessionStats;
use crate::config::{Export, ExportFormat};
// STATS_EXPORT_DIR removed - now using config values directly
use crate::error::{StatsError, Result};
use chrono::Utc;
use csv::Writer;

use tokio::fs;
use tracing::info;

/// Trait for statistics export functionality
pub trait StatsExporter: Send + Sync {
    /// Export statistics in the configured format
    fn export_stats(&self, stats: &SessionStats, config: &Export) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// Default statistics exporter implementation
pub struct DefaultStatsExporter;

impl DefaultStatsExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultStatsExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsExporter for DefaultStatsExporter {
    async fn export_stats(&self, stats: &SessionStats, config: &Export) -> Result<()> {
        if !config.enabled {
            return Ok(());
        }

        // Ensure export directory exists
        fs::create_dir_all(&config.path)
            .await
            .map_err(|e| StatsError::new(format!("Failed to create export directory: {}", e)))?;

        match config.format {
            ExportFormat::Json => {
                self.export_json(stats, config).await?;
            }
            ExportFormat::Csv => {
                self.export_csv(stats, config).await?;
            }
            ExportFormat::Yaml => {
                // TODO: Implement YAML export
                return Err(StatsError::new("YAML export not yet implemented").into());
            }
            ExportFormat::Text => {
                // TODO: Implement text export
                return Err(StatsError::new("Text export not yet implemented").into());
            }
        }
        
        Ok(())
    }
}

impl DefaultStatsExporter {
    async fn export_json(&self, stats: &SessionStats, config: &Export) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/router_flood_stats_{}.json",
            config.path, timestamp
        );

        let json = serde_json::to_string_pretty(stats)
            .map_err(|e| StatsError::new(format!("Failed to serialize stats: {}", e)))?;

        fs::write(&filename, json)
            .await
            .map_err(|e| StatsError::new(format!("Failed to write JSON stats: {}", e)))?;

        info!("Stats exported to {}", filename);
        Ok(())
    }

    async fn export_csv(&self, stats: &SessionStats, config: &Export) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/router_flood_stats_{}.csv",
            config.path, timestamp
        );

        let file = std::fs::File::create(&filename)
            .map_err(|e| StatsError::new(format!("Failed to create CSV file: {}", e)))?;

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
            .map_err(|e| StatsError::new(format!("Failed to write CSV header: {}", e)))?;

        // Write data - using constants for protocol names
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
                &stats.protocol_breakdown.get(crate::constants::protocols::UDP).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(crate::constants::protocols::TCP).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(crate::constants::protocols::ICMP).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(crate::constants::protocols::IPV6).unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get(crate::constants::protocols::ARP).unwrap_or(&0).to_string(),
            ])
            .map_err(|e| StatsError::new(format!("Failed to write CSV data: {}", e)))?;

        writer
            .flush()
            .map_err(|e| StatsError::new(format!("Failed to flush CSV: {}", e)))?;
        
        info!("Stats exported to {}", filename);
        Ok(())
    }
}