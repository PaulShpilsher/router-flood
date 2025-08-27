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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::stats::collector::SessionStats;

    fn create_test_session_stats() -> SessionStats {
        let mut protocol_breakdown = HashMap::new();
        protocol_breakdown.insert("UDP".to_string(), 1000);
        protocol_breakdown.insert("TCP".to_string(), 500);
        protocol_breakdown.insert("ICMP".to_string(), 100);

        SessionStats {
            session_id: "test-session".to_string(),
            timestamp: chrono::Utc::now(),
            packets_sent: 1600,
            packets_failed: 10,
            bytes_sent: 160000,
            duration_secs: 60.0,
            packets_per_second: 26.67,
            megabits_per_second: 0.213,
            protocol_breakdown,
            system_stats: None,
        }
    }

    fn create_test_system_stats() -> crate::stats::collector::SystemStats {
        crate::stats::collector::SystemStats {
            cpu_usage: 25.5,
            memory_usage: 1024 * 1024 * 512, // 512 MB
            memory_total: 1024 * 1024 * 1024 * 8, // 8 GB
            network_sent: 160000,
            network_received: 5000,
        }
    }

    #[test]
    fn test_prometheus_exporter_creation() {
        let exporter = PrometheusExporter::new("router_flood")
            .with_label("instance", "test")
            .with_label("version", "1.0.0");

        assert_eq!(exporter.namespace, "router_flood");
        assert_eq!(exporter.labels.len(), 2);
    }

    #[test]
    fn test_session_stats_export() {
        let exporter = PrometheusExporter::new("router_flood");
        let stats = create_test_session_stats();
        
        let result = exporter.export_session_stats(&stats);
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert!(metrics.contains("router_flood_packets_sent_total"));
        assert!(metrics.contains("router_flood_packets_failed_total"));
        assert!(metrics.contains("router_flood_bytes_sent_total"));
        assert!(metrics.contains("router_flood_duration_seconds"));
        assert!(metrics.contains("router_flood_packets_per_second"));
        assert!(metrics.contains("router_flood_megabits_per_second"));
        assert!(metrics.contains("router_flood_success_rate_percent"));
        assert!(metrics.contains("router_flood_packets_by_protocol_total"));
    }

    #[test]
    fn test_system_stats_export() {
        let exporter = PrometheusExporter::new("router_flood");
        let stats = create_test_system_stats();
        
        let result = exporter.export_system_stats(&stats);
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert!(metrics.contains("router_flood_cpu_usage_percent"));
        assert!(metrics.contains("router_flood_memory_usage_bytes"));
        assert!(metrics.contains("router_flood_memory_total_bytes"));
    }

    #[test]
    fn test_combined_metrics_export() {
        let exporter = PrometheusExporter::new("router_flood")
            .with_label("test", "true");
        let session_stats = create_test_session_stats();
        let system_stats = create_test_system_stats();
        
        let result = exporter.export_combined_metrics(&session_stats, Some(&system_stats));
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        // Should contain both session and system metrics
        assert!(metrics.contains("router_flood_packets_sent_total"));
        assert!(metrics.contains("router_flood_cpu_usage_percent"));
        assert!(metrics.contains("test=\"true\""));
    }

    #[test]
    fn test_protocol_breakdown_metrics() {
        let exporter = PrometheusExporter::new("test");
        let stats = create_test_session_stats();
        
        let metrics = exporter.export_session_stats(&stats).unwrap();
        
        // Should contain protocol-specific metrics
        assert!(metrics.contains("protocol=\"UDP\""));
        assert!(metrics.contains("protocol=\"TCP\""));
        assert!(metrics.contains("protocol=\"ICMP\""));
    }

    #[tokio::test]
    async fn test_save_to_file() {
        let exporter = PrometheusExporter::new("test");
        let session_stats = create_test_session_stats();
        let system_stats = create_test_system_stats();
        
        let temp_file = "/tmp/test_metrics.txt";
        let result = exporter.save_to_file(&session_stats, Some(&system_stats), temp_file).await;
        
        assert!(result.is_ok());
        
        // Verify file was created and contains metrics
        let content = tokio::fs::read_to_string(temp_file).await.unwrap();
        assert!(content.contains("test_packets_sent_total"));
        assert!(content.contains("test_cpu_usage_percent"));
        
        // Clean up
        let _ = tokio::fs::remove_file(temp_file).await;
    }
}