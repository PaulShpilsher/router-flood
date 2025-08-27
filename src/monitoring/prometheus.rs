//! Prometheus metrics export functionality
//!
//! This module provides Prometheus-compatible metrics export for monitoring
//! router-flood performance and statistics in production environments.

use crate::error::{StatsError, Result};
use crate::stats::SessionStats;
use std::collections::HashMap;
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    namespace: String,
    labels: HashMap<String, String>,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            labels: HashMap::new(),
        }
    }

    /// Add a label to all metrics
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Export session statistics as Prometheus metrics
    pub fn export_session_stats(&self, stats: &SessionStats) -> Result<String> {
        let mut output = String::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| StatsError::ExportFailed(format!("Time error: {}", e)))?
            .as_millis();

        // Helper function to format labels
        let format_labels = |additional: &[(&str, &str)]| -> String {
            let mut labels = self.labels.clone();
            for (key, value) in additional {
                labels.insert(key.to_string(), value.to_string());
            }
            
            if labels.is_empty() {
                String::new()
            } else {
                let label_pairs: Vec<String> = labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", label_pairs.join(","))
            }
        };

        // Packet metrics
        writeln!(
            output,
            "# HELP {}_packets_sent_total Total number of packets sent",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_packets_sent_total counter",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_packets_sent_total{} {} {}",
            self.namespace,
            format_labels(&[]),
            stats.packets_sent,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        writeln!(
            output,
            "# HELP {}_packets_failed_total Total number of failed packets",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_packets_failed_total counter",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_packets_failed_total{} {} {}",
            self.namespace,
            format_labels(&[]),
            stats.packets_failed,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        // Bytes metrics
        writeln!(
            output,
            "# HELP {}_bytes_sent_total Total number of bytes sent",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_bytes_sent_total counter",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_bytes_sent_total{} {} {}",
            self.namespace,
            format_labels(&[]),
            stats.bytes_sent,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        // Duration metric
        writeln!(
            output,
            "# HELP {}_duration_seconds Total duration of the session in seconds",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_duration_seconds gauge",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_duration_seconds{} {:.3} {}",
            self.namespace,
            format_labels(&[]),
            stats.duration_secs,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        // Rate metrics
        let packets_per_second = if stats.duration_secs > 0.0 {
            stats.packets_sent as f64 / stats.duration_secs
        } else {
            0.0
        };

        writeln!(
            output,
            "# HELP {}_packets_per_second Current packet rate",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_packets_per_second gauge",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_packets_per_second{} {:.2} {}",
            self.namespace,
            format_labels(&[]),
            packets_per_second,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        let megabits_per_second = if stats.duration_secs > 0.0 {
            (stats.bytes_sent as f64 * 8.0) / (stats.duration_secs * 1_000_000.0)
        } else {
            0.0
        };

        writeln!(
            output,
            "# HELP {}_megabits_per_second Current throughput in Mbps",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_megabits_per_second gauge",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_megabits_per_second{} {:.2} {}",
            self.namespace,
            format_labels(&[]),
            megabits_per_second,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        // Protocol breakdown metrics
        writeln!(
            output,
            "# HELP {}_packets_by_protocol_total Packets sent by protocol",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_packets_by_protocol_total counter",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        for (protocol, count) in &stats.protocol_breakdown {
            writeln!(
                output,
                "{}_packets_by_protocol_total{} {} {}",
                self.namespace,
                format_labels(&[("protocol", protocol)]),
                count,
                timestamp
            ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        }

        // Success rate metric
        let success_rate = if stats.packets_sent + stats.packets_failed > 0 {
            (stats.packets_sent as f64 / (stats.packets_sent + stats.packets_failed) as f64) * 100.0
        } else {
            100.0
        };

        writeln!(
            output,
            "# HELP {}_success_rate_percent Success rate percentage",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_success_rate_percent gauge",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_success_rate_percent{} {:.2} {}",
            self.namespace,
            format_labels(&[]),
            success_rate,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        Ok(output)
    }

    /// Export system statistics as Prometheus metrics
    pub fn export_system_stats(&self, stats: &crate::stats::collector::SystemStats) -> Result<String> {
        let mut output = String::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| StatsError::ExportFailed(format!("Time error: {}", e)))?
            .as_millis();

        let format_labels = |additional: &[(&str, &str)]| -> String {
            let mut labels = self.labels.clone();
            for (key, value) in additional {
                labels.insert(key.to_string(), value.to_string());
            }
            
            if labels.is_empty() {
                String::new()
            } else {
                let label_pairs: Vec<String> = labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", label_pairs.join(","))
            }
        };

        // CPU usage
        writeln!(
            output,
            "# HELP {}_cpu_usage_percent CPU usage percentage",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_cpu_usage_percent gauge",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_cpu_usage_percent{} {:.2} {}",
            self.namespace,
            format_labels(&[]),
            stats.cpu_usage,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        // Memory usage
        writeln!(
            output,
            "# HELP {}_memory_usage_bytes Memory usage in bytes",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_memory_usage_bytes gauge",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_memory_usage_bytes{} {} {}",
            self.namespace,
            format_labels(&[]),
            stats.memory_usage,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        // Memory total
        writeln!(
            output,
            "# HELP {}_memory_total_bytes Total memory in bytes",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "# TYPE {}_memory_total_bytes gauge",
            self.namespace
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;
        
        writeln!(
            output,
            "{}_memory_total_bytes{} {} {}",
            self.namespace,
            format_labels(&[]),
            stats.memory_total,
            timestamp
        ).map_err(|e| StatsError::SerializationError(e.to_string()))?;

        Ok(output)
    }

    /// Export combined metrics (session + system)
    pub fn export_combined_metrics(
        &self,
        session_stats: &SessionStats,
        system_stats: Option<&crate::stats::collector::SystemStats>,
    ) -> Result<String> {
        let mut output = self.export_session_stats(session_stats)?;
        
        if let Some(sys_stats) = system_stats {
            output.push('\n');
            output.push_str(&self.export_system_stats(sys_stats)?);
        }
        
        Ok(output)
    }

    /// Save metrics to file in Prometheus format
    pub async fn save_to_file(
        &self,
        session_stats: &SessionStats,
        system_stats: Option<&crate::stats::collector::SystemStats>,
        file_path: &str,
    ) -> Result<()> {
        let metrics = self.export_combined_metrics(session_stats, system_stats)?;
        
        tokio::fs::write(file_path, metrics)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write metrics file: {}", e)))?;
        
        Ok(())
    }
}

/// HTTP server for serving Prometheus metrics
#[cfg(feature = "http-server")]
pub mod http_server {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use warp::Filter;

    /// Metrics HTTP server
    pub struct MetricsServer {
        exporter: PrometheusExporter,
        session_stats: Arc<RwLock<Option<SessionStats>>>,
        system_stats: Arc<RwLock<Option<crate::stats::SystemStats>>>,
        port: u16,
    }

    impl MetricsServer {
        /// Create a new metrics server
        pub fn new(exporter: PrometheusExporter, port: u16) -> Self {
            Self {
                exporter,
                session_stats: Arc::new(RwLock::new(None)),
                system_stats: Arc::new(RwLock::new(None)),
                port,
            }
        }

        /// Update session statistics
        pub async fn update_session_stats(&self, stats: SessionStats) {
            *self.session_stats.write().await = Some(stats);
        }

        /// Update system statistics
        pub async fn update_system_stats(&self, stats: crate::stats::SystemStats) {
            *self.system_stats.write().await = Some(stats);
        }

        /// Start the HTTP server
        pub async fn start(&self) -> Result<()> {
            let session_stats = self.session_stats.clone();
            let system_stats = self.system_stats.clone();
            let exporter = self.exporter.clone();

            let metrics_route = warp::path("metrics")
                .and(warp::get())
                .and_then(move || {
                    let session_stats = session_stats.clone();
                    let system_stats = system_stats.clone();
                    let exporter = exporter.clone();
                    
                    async move {
                        let session = session_stats.read().await;
                        let system = system_stats.read().await;
                        
                        match session.as_ref() {
                            Some(session_stats) => {
                                match exporter.export_combined_metrics(session_stats, system.as_ref()) {
                                    Ok(metrics) => Ok(warp::reply::with_header(
                                        metrics,
                                        "content-type",
                                        "text/plain; version=0.0.4; charset=utf-8",
                                    )),
                                    Err(_) => Err(warp::reject::not_found()),
                                }
                            }
                            None => Err(warp::reject::not_found()),
                        }
                    }
                });

            let health_route = warp::path("health")
                .and(warp::get())
                .map(|| "OK");

            let routes = metrics_route.or(health_route);

            warp::serve(routes)
                .run(([0, 0, 0, 0], self.port))
                .await;

            Ok(())
        }
    }
}

