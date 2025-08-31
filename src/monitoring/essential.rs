//! Essential monitoring system following YAGNI principles
//!
//! This module provides essential monitoring capabilities without
//! over-engineering, focusing on core metrics and simple export.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::time;

use crate::utils::shared::{AtomicCounter, format_bytes, format_duration};

/// Essential metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub bytes_sent: u64,
    pub duration_secs: f64,
    pub packets_per_second: f64,
    pub success_rate: f64,
    pub bandwidth_mbps: f64,
}

impl Metrics {
    pub fn new(
        packets_sent: u64,
        packets_failed: u64,
        bytes_sent: u64,
        duration: Duration,
    ) -> Self {
        let duration_secs = duration.as_secs_f64();
        let packets_per_second = if duration_secs > 0.0 {
            packets_sent as f64 / duration_secs
        } else {
            0.0
        };
        let success_rate = if packets_sent + packets_failed > 0 {
            (packets_sent as f64 / (packets_sent + packets_failed) as f64) * 100.0
        } else {
            100.0
        };
        let bandwidth_mbps = if duration_secs > 0.0 {
            (bytes_sent as f64 * 8.0) / (duration_secs * 1_000_000.0)
        } else {
            0.0
        };

        Self {
            packets_sent,
            packets_failed,
            bytes_sent,
            duration_secs,
            packets_per_second,
            success_rate,
            bandwidth_mbps,
        }
    }
}

/// Essential metrics collector
pub struct EssentialMetricsCollector {
    packets_sent: AtomicCounter,
    packets_failed: AtomicCounter,
    bytes_sent: AtomicCounter,
    start_time: Instant,
    // rate_calculator: RateCalculator, // Unused field removed
}

impl EssentialMetricsCollector {
    pub fn new() -> Self {
        Self {
            packets_sent: AtomicCounter::new("packets_sent"),
            packets_failed: AtomicCounter::new("packets_failed"),
            bytes_sent: AtomicCounter::new("bytes_sent"),
            start_time: Instant::now(),
            // rate_calculator: RateCalculator::new(), // Unused field removed
        }
    }

    pub fn record_packet_sent(&self, size: usize) {
        self.packets_sent.increment();
        self.bytes_sent.add(size as u64);
    }

    pub fn record_packet_failed(&self) {
        self.packets_failed.increment();
    }

    pub fn get_metrics(&self) -> Metrics {
        let packets_sent = self.packets_sent.get();
        let packets_failed = self.packets_failed.get();
        let bytes_sent = self.bytes_sent.get();
        let duration = self.start_time.elapsed();

        Metrics::new(packets_sent, packets_failed, bytes_sent, duration)
    }

    pub fn reset(&self) {
        self.packets_sent.reset();
        self.packets_failed.reset();
        self.bytes_sent.reset();
    }
}

impl Default for EssentialMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Essential display for metrics
pub struct Display;

impl Display {
    pub fn display_metrics(metrics: &Metrics) {
        println!("📊 Network Testing Metrics");
        println!("═══════════════════════════");
        println!("Packets Sent:     {}", metrics.packets_sent);
        println!("Packets Failed:   {}", metrics.packets_failed);
        println!("Success Rate:     {:.1}%", metrics.success_rate);
        println!("Data Sent:        {}", format_bytes(metrics.bytes_sent));
        println!("Duration:         {}", format_duration(Duration::from_secs_f64(metrics.duration_secs)));
        println!("Rate:             {:.1} packets/sec", metrics.packets_per_second);
        println!("Bandwidth:        {:.2} Mbps", metrics.bandwidth_mbps);
    }

    pub fn display_compact(metrics: &Metrics) {
        println!(
            "📊 Sent: {} | Failed: {} | Rate: {:.1} pps | Success: {:.1}% | Bandwidth: {:.2} Mbps",
            metrics.packets_sent,
            metrics.packets_failed,
            metrics.packets_per_second,
            metrics.success_rate,
            metrics.bandwidth_mbps
        );
    }
}

/// Essential export functionality
pub struct Exporter;

impl Exporter {
    pub async fn export_json(metrics: &Metrics, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(metrics)?;
        tokio::fs::write(filename, json).await?;
        println!("📄 Metrics exported to {}", filename);
        Ok(())
    }

    pub async fn export_csv(metrics: &Metrics, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let csv_content = format!(
            "packets_sent,packets_failed,bytes_sent,duration_secs,packets_per_second,success_rate,bandwidth_mbps\n{},{},{},{:.3},{:.1},{:.2},{:.3}",
            metrics.packets_sent,
            metrics.packets_failed,
            metrics.bytes_sent,
            metrics.duration_secs,
            metrics.packets_per_second,
            metrics.success_rate,
            metrics.bandwidth_mbps
        );
        
        tokio::fs::write(filename, csv_content).await?;
        println!("📄 Metrics exported to {}", filename);
        Ok(())
    }
}

/// Essential monitoring task that periodically displays metrics
pub struct Monitor {
    collector: Arc<EssentialMetricsCollector>,
    interval: Duration,
}

impl Monitor {
    pub fn new(collector: Arc<EssentialMetricsCollector>, interval: Duration) -> Self {
        Self { collector, interval }
    }

    pub async fn run(&self, running: Arc<std::sync::atomic::AtomicBool>) {
        let mut interval_timer = time::interval(self.interval);
        
        while running.load(Ordering::Relaxed) {
            interval_timer.tick().await;
            
            let metrics = self.collector.get_metrics();
            Display::display_compact(&metrics);
        }
    }
}

/// Configuration for essential monitoring
#[derive(Debug, Clone)]
pub struct Monitoring {
    pub display_interval: Duration,
    pub export_enabled: bool,
    pub export_format: ExportFormat,
    pub export_filename: String,
}

impl Default for Monitoring {
    fn default() -> Self {
        Self {
            display_interval: Duration::from_secs(5),
            export_enabled: false,
            export_format: ExportFormat::Json,
            export_filename: "metrics".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// All-in-one essential monitoring system
pub struct MonitoringSystem {
    collector: Arc<EssentialMetricsCollector>,
    config: Monitoring,
}

impl MonitoringSystem {
    pub fn new(config: Monitoring) -> Self {
        Self {
            collector: Arc::new(EssentialMetricsCollector::new()),
            config,
        }
    }

    pub fn collector(&self) -> Arc<EssentialMetricsCollector> {
        Arc::clone(&self.collector)
    }

    pub async fn run(&self, running: Arc<std::sync::atomic::AtomicBool>) {
        let monitor = Monitor::new(
            Arc::clone(&self.collector),
            self.config.display_interval,
        );

        monitor.run(running).await;
    }

    pub async fn export_final_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.export_enabled {
            return Ok(());
        }

        let metrics = self.collector.get_metrics();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

        match self.config.export_format {
            ExportFormat::Json => {
                let filename = format!("{}_{}.json", self.config.export_filename, timestamp);
                Exporter::export_json(&metrics, &filename).await?;
            }
            ExportFormat::Csv => {
                let filename = format!("{}_{}.csv", self.config.export_filename, timestamp);
                Exporter::export_csv(&metrics, &filename).await?;
            }
        }

        Ok(())
    }

    pub fn display_final_summary(&self) {
        let metrics = self.collector.get_metrics();
        println!("\n🎯 Final Summary");
        Display::display_metrics(&metrics);
    }
}

// Tests moved to tests/ directory
