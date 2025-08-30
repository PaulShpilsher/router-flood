//! Lightweight dashboard for monitoring enhancements
//!
//! This module provides a minimal, efficient dashboard that focuses
//! on essential metrics without over-engineering, following YAGNI principles.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;
use serde::{Deserialize, Serialize};

use crate::monitoring::essential::{EssentialMetrics, EssentialMetricsCollector};
use crate::utils::shared::{format_bytes, format_duration};

/// Lightweight dashboard
pub struct Dashboard {
    collector: Arc<EssentialMetricsCollector>,
    config: DashboardConfig,
    start_time: Instant,
    last_update: Arc<AtomicU64>,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub update_interval: Duration,
    pub show_system_info: bool,
    pub show_progress_bar: bool,
    pub compact_mode: bool,
    pub alert_thresholds: AlertThresholds,
}

/// Alert thresholds for monitoring
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_failure_rate: f64,    // Percentage
    pub min_success_rate: f64,    // Percentage
    pub max_response_time: f64,   // Milliseconds
    pub min_throughput: f64,      // Packets per second
}

/// Dashboard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub timestamp: String,
    pub uptime: String,
    pub metrics: EssentialMetrics,
    pub alerts: Vec<Alert>,
    pub system_info: Option<SystemInfo>,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_utilization: f64,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: String,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(1),
            show_system_info: false,
            show_progress_bar: true,
            compact_mode: false,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_failure_rate: 10.0,   // 10% failure rate triggers warning
            min_success_rate: 95.0,   // Below 95% success rate triggers warning
            max_response_time: 100.0, // Above 100ms response time triggers info
            min_throughput: 10.0,     // Below 10 pps triggers info
        }
    }
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(collector: Arc<EssentialMetricsCollector>, config: DashboardConfig) -> Self {
        Self {
            collector,
            config,
            start_time: Instant::now(),
            last_update: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Start the dashboard
    pub async fn start(&self, running: Arc<AtomicBool>) {
        let mut interval = time::interval(self.config.update_interval);
        
        // Clear screen and show header
        if !self.config.compact_mode {
            self.clear_screen();
            self.show_header();
        }
        
        while running.load(Ordering::Relaxed) {
            interval.tick().await;
            self.update_display().await;
        }
        
        // Show final summary
        if !self.config.compact_mode {
            self.show_final_summary();
        }
    }

    /// Update the dashboard display
    async fn update_display(&self) {
        let state = self.get_dashboard_state().await;
        
        if self.config.compact_mode {
            self.display_compact(&state);
        } else {
            self.display_full(&state);
        }
        
        // Update last update timestamp
        self.last_update.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            Ordering::Relaxed,
        );
    }

    /// Get current dashboard state
    async fn get_dashboard_state(&self) -> DashboardState {
        let metrics = self.collector.get_metrics();
        let uptime = self.start_time.elapsed();
        let alerts = self.check_alerts(&metrics);
        let system_info = if self.config.show_system_info {
            Some(self.get_system_info().await)
        } else {
            None
        };

        DashboardState {
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            uptime: format_duration(uptime),
            metrics,
            alerts,
            system_info,
        }
    }

    /// Check for alerts based on current metrics
    fn check_alerts(&self, metrics: &EssentialMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let now = chrono::Utc::now().format("%H:%M:%S").to_string();
        
        // Check failure rate
        let failure_rate = if metrics.packets_sent + metrics.packets_failed > 0 {
            (metrics.packets_failed as f64 / (metrics.packets_sent + metrics.packets_failed) as f64) * 100.0
        } else {
            0.0
        };
        
        if failure_rate > self.config.alert_thresholds.max_failure_rate {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                message: format!("High failure rate: {:.1}%", failure_rate),
                timestamp: now.clone(),
            });
        }
        
        // Check success rate
        if metrics.success_rate < self.config.alert_thresholds.min_success_rate {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                message: format!("Low success rate: {:.1}%", metrics.success_rate),
                timestamp: now.clone(),
            });
        }
        
        // Check throughput
        if metrics.packets_per_second < self.config.alert_thresholds.min_throughput {
            alerts.push(Alert {
                level: AlertLevel::Info,
                message: format!("Low throughput: {:.1} pps", metrics.packets_per_second),
                timestamp: now,
            });
        }
        
        alerts
    }

    /// Get basic system information
    async fn get_system_info(&self) -> SystemInfo {
        // Essential system info - in a real implementation, this would
        // use system monitoring libraries like sysinfo
        SystemInfo {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            network_utilization: 0.0,
        }
    }

    /// Display compact dashboard
    fn display_compact(&self, state: &DashboardState) {
        let status_icon = if state.alerts.is_empty() { "✅" } else { "⚠️" };
        
        print!("\r{} [{}] Sent: {} | Failed: {} | Rate: {:.1} pps | Success: {:.1}% | {}",
            status_icon,
            state.timestamp,
            state.metrics.packets_sent,
            state.metrics.packets_failed,
            state.metrics.packets_per_second,
            state.metrics.success_rate,
            state.uptime
        );
        
        // Flush stdout to ensure immediate display
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }

    /// Display full dashboard
    fn display_full(&self, state: &DashboardState) {
        // Move cursor to top and clear screen content
        print!("\x1b[H");
        
        // Dashboard header
        println!("🚀 Router Flood - Dashboard");
        println!("═══════════════════════════════════════");
        println!("Time: {} | Uptime: {}", state.timestamp, state.uptime);
        println!();
        
        // Core metrics
        println!("📊 Core Metrics");
        println!("───────────────");
        println!("Packets Sent:     {:>10}", state.metrics.packets_sent);
        println!("Packets Failed:   {:>10}", state.metrics.packets_failed);
        println!("Success Rate:     {:>9.1}%", state.metrics.success_rate);
        println!("Data Sent:        {:>10}", format_bytes(state.metrics.bytes_sent));
        println!();
        
        // Performance metrics
        println!("⚡ Performance");
        println!("──────────────");
        println!("Rate:             {:>9.1} pps", state.metrics.packets_per_second);
        println!("Bandwidth:        {:>9.2} Mbps", state.metrics.bandwidth_mbps);
        println!("Duration:         {:>10}", format_duration(Duration::from_secs_f64(state.metrics.duration_secs)));
        
        // Progress bar
        if self.config.show_progress_bar {
            println!();
            self.display_progress_bar(&state.metrics);
        }
        
        // System info
        if let Some(ref sys_info) = state.system_info {
            println!();
            println!("💻 System");
            println!("─────────");
            println!("CPU Usage:        {:>9.1}%", sys_info.cpu_usage);
            println!("Memory Usage:     {:>9.1}%", sys_info.memory_usage);
            println!("Network:          {:>9.1}%", sys_info.network_utilization);
        }
        
        // Alerts
        if !state.alerts.is_empty() {
            println!();
            println!("🚨 Alerts");
            println!("─────────");
            for alert in &state.alerts {
                let icon = match alert.level {
                    AlertLevel::Info => "ℹ️",
                    AlertLevel::Warning => "⚠️",
                    AlertLevel::Critical => "🚨",
                };
                println!("{} [{}] {}", icon, alert.timestamp, alert.message);
            }
        }
        
        // Footer
        println!();
        println!("Press Ctrl+C to stop");
        
        // Clear any remaining lines
        print!("\x1b[J");
    }

    /// Display a simple progress bar
    fn display_progress_bar(&self, metrics: &EssentialMetrics) {
        let width = 40;
        let success_ratio = metrics.success_rate / 100.0;
        let filled = (width as f64 * success_ratio) as usize;
        let empty = width - filled;
        
        let bar = "█".repeat(filled) + &"░".repeat(empty);
        println!("Progress: [{}] {:.1}%", bar, metrics.success_rate);
    }

    /// Clear screen
    fn clear_screen(&self) {
        print!("\x1b[2J\x1b[H");
    }

    /// Show dashboard header
    fn show_header(&self) {
        println!("🚀 Router Flood - Dashboard");
        println!("═══════════════════════════════════════");
        println!("Starting monitoring...");
        println!();
    }

    /// Show final summary
    fn show_final_summary(&self) {
        println!();
        println!("🎯 Final Summary");
        println!("═══════════════");
        
        let final_metrics = self.collector.get_metrics();
        let uptime = self.start_time.elapsed();
        
        println!("Total Runtime:    {}", format_duration(uptime));
        println!("Packets Sent:     {}", final_metrics.packets_sent);
        println!("Packets Failed:   {}", final_metrics.packets_failed);
        println!("Success Rate:     {:.1}%", final_metrics.success_rate);
        println!("Average Rate:     {:.1} pps", final_metrics.packets_per_second);
        println!("Total Data:       {}", format_bytes(final_metrics.bytes_sent));
        println!("Average Bandwidth: {:.2} Mbps", final_metrics.bandwidth_mbps);
        println!();
    }

    /// Export dashboard state to JSON
    pub async fn export_state(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.get_dashboard_state().await;
        let json = serde_json::to_string_pretty(&state)?;
        tokio::fs::write(filename, json).await?;
        Ok(())
    }
}

/// Builder for dashboard configuration
pub struct DashboardBuilder {
    config: DashboardConfig,
}

impl DashboardBuilder {
    pub fn new() -> Self {
        Self {
            config: DashboardConfig::default(),
        }
    }

    pub fn update_interval(mut self, interval: Duration) -> Self {
        self.config.update_interval = interval;
        self
    }

    pub fn compact_mode(mut self, enabled: bool) -> Self {
        self.config.compact_mode = enabled;
        self
    }

    pub fn show_system_info(mut self, enabled: bool) -> Self {
        self.config.show_system_info = enabled;
        self
    }

    pub fn show_progress_bar(mut self, enabled: bool) -> Self {
        self.config.show_progress_bar = enabled;
        self
    }

    pub fn alert_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.config.alert_thresholds = thresholds;
        self
    }

    pub fn build(self) -> DashboardConfig {
        self.config
    }
}

impl Default for DashboardBuilder {
    fn default() -> Self {
        Self::new()
    }
}