//! Statistics export functionality

use super::collector::SessionStats;
use crate::config::{Export, ExportFormat};
use crate::error::{StatsError, Result};
use chrono::Utc;
use csv::Writer;
use serde_yaml;
use std::fmt::Write;
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
                self.export_yaml(stats, config).await?;
            }
            ExportFormat::Text => {
                self.export_text(stats, config).await?;
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

    async fn export_yaml(&self, stats: &SessionStats, config: &Export) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/router_flood_stats_{}.yaml",
            config.path, timestamp
        );

        let yaml = serde_yaml::to_string(stats)
            .map_err(|e| StatsError::new(format!("Failed to serialize stats to YAML: {}", e)))?;

        fs::write(&filename, yaml)
            .await
            .map_err(|e| StatsError::new(format!("Failed to write YAML stats: {}", e)))?;

        info!("Stats exported to {}", filename);
        Ok(())
    }

    async fn export_text(&self, stats: &SessionStats, config: &Export) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/router_flood_stats_{}.txt",
            config.path, timestamp
        );

        // Create human-readable text format
        let mut text = String::new();
        writeln!(&mut text, "=== Router Flood Statistics Report ===").unwrap();
        writeln!(&mut text, "Session ID:          {}", stats.session_id).unwrap();
        writeln!(&mut text, "Timestamp:           {}", stats.timestamp.to_rfc3339()).unwrap();
        writeln!(&mut text, "Duration:            {:.2} seconds", stats.duration_secs).unwrap();
        writeln!(&mut text).unwrap();
        
        writeln!(&mut text, "=== Performance Metrics ===").unwrap();
        writeln!(&mut text, "Packets Sent:        {:>12}", stats.packets_sent).unwrap();
        writeln!(&mut text, "Packets Failed:      {:>12}", stats.packets_failed).unwrap();
        writeln!(&mut text, "Bytes Sent:          {:>12}", stats.bytes_sent).unwrap();
        writeln!(&mut text, "Packets/Second:      {:>12.2}", stats.packets_per_second).unwrap();
        writeln!(&mut text, "Megabits/Second:     {:>12.2}", stats.megabits_per_second).unwrap();
        writeln!(&mut text).unwrap();
        
        if !stats.protocol_breakdown.is_empty() {
            writeln!(&mut text, "=== Protocol Breakdown ===").unwrap();
            for (protocol, count) in &stats.protocol_breakdown {
                writeln!(&mut text, "{:<20} {:>12}", protocol, count).unwrap();
            }
            writeln!(&mut text).unwrap();
        }
        
        if let Some(ref system_stats) = stats.system_stats {
            writeln!(&mut text, "=== System Resources ===").unwrap();
            writeln!(&mut text, "CPU Usage:           {:>11.1}%", system_stats.cpu_usage).unwrap();
            writeln!(&mut text, "Memory Usage:        {:>12} bytes", system_stats.memory_usage).unwrap();
            writeln!(&mut text, "Memory Total:        {:>12} bytes", system_stats.memory_total).unwrap();
            writeln!(&mut text, "Network Sent:        {:>12} bytes", system_stats.network_sent).unwrap();
            writeln!(&mut text, "Network Received:    {:>12} bytes", system_stats.network_received).unwrap();
        }

        fs::write(&filename, text)
            .await
            .map_err(|e| StatsError::new(format!("Failed to write text stats: {}", e)))?;

        info!("Stats exported to {}", filename);
        Ok(())
    }
}