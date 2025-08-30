//! Simplified monitoring system following YAGNI principles
//!
//! This module provides essential monitoring capabilities without
//! over-engineering, focusing on core metrics and simple export.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::time;

use crate::utils::shared::{AtomicCounter, RateCalculator, format_bytes, format_duration};

/// Essential metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssentialMetrics {
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub bytes_sent: u64,
    pub duration_secs: f64,
    pub packets_per_second: f64,
    pub success_rate: f64,
    pub bandwidth_mbps: f64,
}

impl EssentialMetrics {
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

/// Simplified metrics collector
pub struct SimpleMetricsCollector {
    packets_sent: AtomicCounter,
    packets_failed: AtomicCounter,
    bytes_sent: AtomicCounter,
    start_time: Instant,
    // rate_calculator: RateCalculator, // Unused field removed
}

impl SimpleMetricsCollector {
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

    pub fn get_metrics(&self) -> EssentialMetrics {
        let packets_sent = self.packets_sent.get();
        let packets_failed = self.packets_failed.get();
        let bytes_sent = self.bytes_sent.get();
        let duration = self.start_time.elapsed();

        EssentialMetrics::new(packets_sent, packets_failed, bytes_sent, duration)
    }

    pub fn reset(&self) {
        self.packets_sent.reset();
        self.packets_failed.reset();
        self.bytes_sent.reset();
    }
}

impl Default for SimpleMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple display for metrics
pub struct SimpleDisplay;

impl SimpleDisplay {
    pub fn display_metrics(metrics: &EssentialMetrics) {
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

    pub fn display_compact(metrics: &EssentialMetrics) {
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

/// Simple export functionality
pub struct SimpleExporter;

impl SimpleExporter {
    pub async fn export_json(metrics: &EssentialMetrics, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(metrics)?;
        tokio::fs::write(filename, json).await?;
        println!("📄 Metrics exported to {}", filename);
        Ok(())
    }

    pub async fn export_csv(metrics: &EssentialMetrics, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
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

/// Simple monitoring task that periodically displays metrics
pub struct SimpleMonitor {
    collector: Arc<SimpleMetricsCollector>,
    interval: Duration,
}

impl SimpleMonitor {
    pub fn new(collector: Arc<SimpleMetricsCollector>, interval: Duration) -> Self {
        Self { collector, interval }
    }

    pub async fn run(&self, running: Arc<std::sync::atomic::AtomicBool>) {
        let mut interval_timer = time::interval(self.interval);
        
        while running.load(Ordering::Relaxed) {
            interval_timer.tick().await;
            
            let metrics = self.collector.get_metrics();
            SimpleDisplay::display_compact(&metrics);
        }
    }
}

/// Configuration for simple monitoring
#[derive(Debug, Clone)]
pub struct SimpleMonitoringConfig {
    pub display_interval: Duration,
    pub export_enabled: bool,
    pub export_format: ExportFormat,
    pub export_filename: String,
}

impl Default for SimpleMonitoringConfig {
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

/// All-in-one simple monitoring system
pub struct SimpleMonitoringSystem {
    collector: Arc<SimpleMetricsCollector>,
    config: SimpleMonitoringConfig,
}

impl SimpleMonitoringSystem {
    pub fn new(config: SimpleMonitoringConfig) -> Self {
        Self {
            collector: Arc::new(SimpleMetricsCollector::new()),
            config,
        }
    }

    pub fn collector(&self) -> Arc<SimpleMetricsCollector> {
        Arc::clone(&self.collector)
    }

    pub async fn run(&self, running: Arc<std::sync::atomic::AtomicBool>) {
        let monitor = SimpleMonitor::new(
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
                SimpleExporter::export_json(&metrics, &filename).await?;
            }
            ExportFormat::Csv => {
                let filename = format!("{}_{}.csv", self.config.export_filename, timestamp);
                SimpleExporter::export_csv(&metrics, &filename).await?;
            }
        }

        Ok(())
    }

    pub fn display_final_summary(&self) {
        let metrics = self.collector.get_metrics();
        println!("\n🎯 Final Summary");
        SimpleDisplay::display_metrics(&metrics);
    }
}

// Tests moved to tests/ directory
