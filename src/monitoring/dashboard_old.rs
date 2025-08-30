//! Real-time performance dashboard

use super::metrics::{MetricsCollector, RouterFloodMetrics, MetricsSummary};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Real-time performance dashboard
pub struct PerformanceDashboard {
    metrics_collector: Arc<MetricsCollector>,
    router_metrics: RouterFloodMetrics,
    update_interval: Duration,
    start_time: Instant,
}

impl PerformanceDashboard {
    pub fn new(metrics_collector: Arc<MetricsCollector>, update_interval: Duration) -> Self {
        let router_metrics = RouterFloodMetrics::new(&metrics_collector);
        
        Self {
            metrics_collector,
            router_metrics,
            update_interval,
            start_time: Instant::now(),
        }
    }
    
    /// Start the dashboard update loop
    pub async fn start(&self) {
        let mut interval = time::interval(self.update_interval);
        
        loop {
            interval.tick().await;
            self.update_dashboard().await;
        }
    }
    
    /// Update dashboard metrics
    async fn update_dashboard(&self) {
        // This would typically collect system metrics
        // For now, we'll simulate some updates
        
        let uptime = self.start_time.elapsed().as_secs_f64();
        
        // Update system metrics (these would come from actual system monitoring)
        self.router_metrics.cpu_usage.update(
            self.simulate_cpu_usage(uptime),
            std::collections::HashMap::new(),
        );
        
        self.router_metrics.memory_usage.update(
            self.simulate_memory_usage(uptime),
            std::collections::HashMap::new(),
        );
    }
    
    /// Get current dashboard state
    pub fn get_dashboard_state(&self) -> DashboardState {
        let summary = self.metrics_collector.get_summary();
        let uptime = self.start_time.elapsed();
        
        DashboardState {
            timestamp: Utc::now(),
            uptime_seconds: uptime.as_secs_f64(),
            metrics_summary: summary,
            performance_indicators: self.calculate_performance_indicators(),
            alerts: self.check_alerts(),
        }
    }
    
    /// Calculate key performance indicators
    fn calculate_performance_indicators(&self) -> PerformanceIndicators {
        let packets_sent = self.router_metrics.packets_sent.current_value();
        let packets_failed = self.router_metrics.packets_failed.current_value();
        let uptime = self.start_time.elapsed().as_secs_f64();
        
        let success_rate = if packets_sent + packets_failed > 0.0 {
            packets_sent / (packets_sent + packets_failed) * 100.0
        } else {
            100.0
        };
        
        let packets_per_second = if uptime > 0.0 {
            packets_sent / uptime
        } else {
            0.0
        };
        
        PerformanceIndicators {
            packets_per_second,
            success_rate_percent: success_rate,
            average_packet_build_time_ms: self.router_metrics.packet_build_time.current_value(),
            average_send_time_ms: self.router_metrics.send_time.current_value(),
            buffer_pool_efficiency: self.router_metrics.buffer_pool_utilization.current_value(),
            cpu_usage_percent: self.router_metrics.cpu_usage.current_value(),
            memory_usage_mb: self.router_metrics.memory_usage.current_value() / 1024.0 / 1024.0,
            network_throughput_mbps: self.router_metrics.network_throughput.current_value(),
        }
    }
    
    /// Check for performance alerts
    fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let indicators = self.calculate_performance_indicators();
        
        // CPU usage alert
        if indicators.cpu_usage_percent > 90.0 {
            alerts.push(Alert {
                level: AlertLevel::Critical,
                message: format!("High CPU usage: {:.1}%", indicators.cpu_usage_percent),
                timestamp: Utc::now(),
            });
        } else if indicators.cpu_usage_percent > 75.0 {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                message: format!("Elevated CPU usage: {:.1}%", indicators.cpu_usage_percent),
                timestamp: Utc::now(),
            });
        }
        
        // Success rate alert
        if indicators.success_rate_percent < 95.0 {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                message: format!("Low success rate: {:.1}%", indicators.success_rate_percent),
                timestamp: Utc::now(),
            });
        }
        
        // Performance alert
        if indicators.average_packet_build_time_ms > 10.0 {
            alerts.push(Alert {
                level: AlertLevel::Info,
                message: format!("Slow packet building: {:.2}ms", indicators.average_packet_build_time_ms),
                timestamp: Utc::now(),
            });
        }
        
        alerts
    }
    
    // Simulation functions for demo purposes
    fn simulate_cpu_usage(&self, uptime: f64) -> f64 {
        // Simulate varying CPU usage
        let base = 30.0;
        let variation = 20.0 * (uptime * 0.1).sin();
        (base + variation).max(0.0).min(100.0)
    }
    
    fn simulate_memory_usage(&self, uptime: f64) -> f64 {
        // Simulate gradually increasing memory usage
        let base = 50.0 * 1024.0 * 1024.0; // 50MB base
        let growth = uptime * 1024.0 * 10.0; // 10KB per second growth
        base + growth
    }
}

/// Complete dashboard state
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardState {
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: f64,
    pub metrics_summary: MetricsSummary,
    pub performance_indicators: PerformanceIndicators,
    pub alerts: Vec<Alert>,
}

/// Key performance indicators
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceIndicators {
    pub packets_per_second: f64,
    pub success_rate_percent: f64,
    pub average_packet_build_time_ms: f64,
    pub average_send_time_ms: f64,
    pub buffer_pool_efficiency: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub network_throughput_mbps: f64,
}

/// Alert levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

