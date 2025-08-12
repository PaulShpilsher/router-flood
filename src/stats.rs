use crate::config::{ExportConfig, ExportFormat, STATS_EXPORT_DIR};
use chrono::{DateTime, Utc};
use csv::Writer;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

/// Enhanced statistics tracking with export capabilities
#[derive(Default)]
pub struct FloodStats {
    pub packets_sent: Arc<AtomicU64>,
    pub packets_failed: Arc<AtomicU64>,
    pub bytes_sent: Arc<AtomicU64>,
    pub start_time: Instant,
    pub session_id: String,
    pub protocol_stats: Arc<HashMap<String, AtomicU64>>,
    pub export_config: Option<ExportConfig>,
}

#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub bytes_sent: u64,
    pub duration_secs: f64,
    pub packets_per_second: f64,
    pub megabits_per_second: f64,
    pub protocol_breakdown: HashMap<String, u64>,
    pub system_stats: Option<SystemStats>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub network_sent: u64,
    pub network_received: u64,
}

impl FloodStats {
    pub fn new(export_config: Option<ExportConfig>) -> Self {
        let mut protocol_stats = HashMap::new();
        protocol_stats.insert("UDP".to_string(), AtomicU64::new(0));
        protocol_stats.insert("TCP".to_string(), AtomicU64::new(0));
        protocol_stats.insert("ICMP".to_string(), AtomicU64::new(0));
        protocol_stats.insert("IPv6".to_string(), AtomicU64::new(0));
        protocol_stats.insert("ARP".to_string(), AtomicU64::new(0));

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
        let mbps = (bytes as f64 * 8.0) / (elapsed * 1_000_000.0);

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
                sys_stats.memory_usage / 1024 / 1024
            );
        }
    }

    pub async fn export_stats(&self, system_stats: Option<&SystemStats>) -> Result<(), String> {
        if let Some(export_config) = &self.export_config {
            if !export_config.enabled {
                return Ok(());
            }

            let stats = self.get_session_stats(system_stats);

            // Ensure export directory exists
            fs::create_dir_all(STATS_EXPORT_DIR)
                .await
                .map_err(|e| format!("Failed to create export directory: {}", e))?;

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
        let mbps = (bytes as f64 * 8.0) / (elapsed * 1_000_000.0);

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
    ) -> Result<(), String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/{}_stats_{}.json",
            STATS_EXPORT_DIR, config.filename_pattern, timestamp
        );

        let json = serde_json::to_string_pretty(stats)
            .map_err(|e| format!("Failed to serialize stats: {}", e))?;

        fs::write(&filename, json)
            .await
            .map_err(|e| format!("Failed to write JSON stats: {}", e))?;

        info!("Stats exported to {}", filename);
        Ok(())
    }

    async fn export_csv(
        &self,
        stats: &SessionStats,
        config: &ExportConfig,
    ) -> Result<(), String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "{}/{}_stats_{}.csv",
            STATS_EXPORT_DIR, config.filename_pattern, timestamp
        );

        let file = std::fs::File::create(&filename)
            .map_err(|e| format!("Failed to create CSV file: {}", e))?;

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
            .map_err(|e| format!("Failed to write CSV header: {}", e))?;

        // Write data
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
                &stats.protocol_breakdown.get("UDP").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("TCP").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("ICMP").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("IPv6").unwrap_or(&0).to_string(),
                &stats.protocol_breakdown.get("ARP").unwrap_or(&0).to_string(),
            ])
            .map_err(|e| format!("Failed to write CSV data: {}", e))?;

        writer
            .flush()
            .map_err(|e| format!("Failed to flush CSV: {}", e))?;
        info!("Stats exported to {}", filename);
        Ok(())
    }
}
