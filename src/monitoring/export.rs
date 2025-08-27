//! Enhanced metrics export system

use super::metrics::{MetricsCollector, MetricsSummary};
use super::dashboard::DashboardState;
use crate::error::{StatsError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use chrono::Utc;
use serde::{Serialize, Deserialize};

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
    InfluxDB,
    Custom(String),
}

/// Metrics exporter with multiple format support
pub struct MetricsExporter {
    metrics_collector: Arc<MetricsCollector>,
    export_config: ExportConfig,
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub enabled: bool,
    pub format: ExportFormat,
    pub output_path: String,
    pub filename_pattern: String,
    pub include_history: bool,
    pub compression: bool,
    pub custom_fields: HashMap<String, String>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: ExportFormat::Json,
            output_path: "exports".to_string(),
            filename_pattern: "router_flood_metrics".to_string(),
            include_history: false,
            compression: false,
            custom_fields: HashMap::new(),
        }
    }
}

impl MetricsExporter {
    pub fn new(metrics_collector: Arc<MetricsCollector>, config: ExportConfig) -> Self {
        Self {
            metrics_collector,
            export_config: config,
        }
    }
    
    /// Export current metrics
    pub async fn export_metrics(&self) -> Result<String> {
        if !self.export_config.enabled {
            return Ok("Export disabled".to_string());
        }
        
        // Ensure export directory exists
        fs::create_dir_all(&self.export_config.output_path)
            .await
            .map_err(|e| StatsError::ExportFailed(format!("Failed to create export directory: {}", e)))?;
        
        let summary = self.metrics_collector.get_summary();
        let export_data = self.prepare_export_data(summary).await?;
        
        let filename = self.generate_filename();
        let filepath = format!("{}/{}", self.export_config.output_path, filename);
        
        match self.export_config.format {
            ExportFormat::Json => self.export_json(&export_data, &filepath).await,
            ExportFormat::Csv => self.export_csv(&export_data, &filepath).await,
            ExportFormat::Prometheus => self.export_prometheus(&export_data, &filepath).await,
            ExportFormat::InfluxDB => self.export_influxdb(&export_data, &filepath).await,
            ExportFormat::Custom(ref format) => self.export_custom(&export_data, &filepath, format).await,
        }
    }
    
    /// Export dashboard state
    pub async fn export_dashboard_state(&self, dashboard_state: &DashboardState) -> Result<String> {
        if !self.export_config.enabled {
            return Ok("Export disabled".to_string());
        }
        
        fs::create_dir_all(&self.export_config.output_path)
            .await
            .map_err(|e| StatsError::ExportFailed(format!("Failed to create export directory: {}", e)))?;
        
        let filename = format!("{}_dashboard_{}.json", 
            self.export_config.filename_pattern,
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = format!("{}/{}", self.export_config.output_path, filename);
        
        let json = serde_json::to_string_pretty(dashboard_state)
            .map_err(|e| StatsError::SerializationError(format!("Failed to serialize dashboard state: {}", e)))?;
        
        fs::write(&filepath, json)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write dashboard state: {}", e)))?;
        
        Ok(filepath)
    }
    
    /// Prepare export data with additional metadata
    async fn prepare_export_data(&self, summary: MetricsSummary) -> Result<ExportData> {
        let mut export_data = ExportData {
            timestamp: Utc::now(),
            summary,
            metadata: ExportMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                export_format: format!("{:?}", self.export_config.format),
                custom_fields: self.export_config.custom_fields.clone(),
            },
            history: None,
        };
        
        // Include history if requested
        if self.export_config.include_history {
            export_data.history = Some(self.collect_metric_history().await);
        }
        
        Ok(export_data)
    }
    
    /// Collect metric history
    async fn collect_metric_history(&self) -> HashMap<String, Vec<super::metrics::MetricValue>> {
        let metrics = self.metrics_collector.get_all_metrics();
        let mut history = HashMap::new();
        
        for (name, metric) in metrics {
            let recent_history = metric.recent_history(std::time::Duration::from_secs(3600)); // 1 hour
            if !recent_history.is_empty() {
                history.insert(name, recent_history);
            }
        }
        
        history
    }
    
    /// Generate filename with timestamp
    fn generate_filename(&self) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let extension = match self.export_config.format {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Prometheus => "txt",
            ExportFormat::InfluxDB => "txt",
            ExportFormat::Custom(_) => "txt",
        };
        
        format!("{}_{}.{}", self.export_config.filename_pattern, timestamp, extension)
    }
    
    /// Export as JSON
    async fn export_json(&self, data: &ExportData, filepath: &str) -> Result<String> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| StatsError::SerializationError(format!("Failed to serialize JSON: {}", e)))?;
        
        fs::write(filepath, json)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write JSON: {}", e)))?;
        
        Ok(filepath.to_string())
    }
    
    /// Export as CSV
    async fn export_csv(&self, data: &ExportData, filepath: &str) -> Result<String> {
        let mut csv_content = String::new();
        
        // Header
        csv_content.push_str("timestamp,metric_name,metric_type,value\n");
        
        // Counters
        for (name, value) in &data.summary.counters {
            csv_content.push_str(&format!("{},{},counter,{}\n", 
                data.timestamp.to_rfc3339(), name, value));
        }
        
        // Gauges
        for (name, value) in &data.summary.gauges {
            csv_content.push_str(&format!("{},{},gauge,{}\n", 
                data.timestamp.to_rfc3339(), name, value));
        }
        
        // Timers
        for (name, value) in &data.summary.timers {
            csv_content.push_str(&format!("{},{},timer,{}\n", 
                data.timestamp.to_rfc3339(), name, value));
        }
        
        fs::write(filepath, csv_content)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write CSV: {}", e)))?;
        
        Ok(filepath.to_string())
    }
    
    /// Export in Prometheus format
    async fn export_prometheus(&self, data: &ExportData, filepath: &str) -> Result<String> {
        let mut prometheus_content = String::new();
        
        // Add metadata
        prometheus_content.push_str(&format!("# HELP router_flood_info Router Flood metrics\n"));
        prometheus_content.push_str(&format!("# TYPE router_flood_info gauge\n"));
        prometheus_content.push_str(&format!("router_flood_info{{version=\"{}\"}} 1\n\n", 
            data.metadata.version));
        
        // Counters
        for (name, value) in &data.summary.counters {
            prometheus_content.push_str(&format!("# TYPE {} counter\n", name));
            prometheus_content.push_str(&format!("{} {}\n\n", name, value));
        }
        
        // Gauges
        for (name, value) in &data.summary.gauges {
            prometheus_content.push_str(&format!("# TYPE {} gauge\n", name));
            prometheus_content.push_str(&format!("{} {}\n\n", name, value));
        }
        
        // Timers (as histograms)
        for (name, value) in &data.summary.timers {
            prometheus_content.push_str(&format!("# TYPE {}_duration_ms histogram\n", name));
            prometheus_content.push_str(&format!("{}_duration_ms {}\n\n", name, value));
        }
        
        fs::write(filepath, prometheus_content)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write Prometheus format: {}", e)))?;
        
        Ok(filepath.to_string())
    }
    
    /// Export in InfluxDB line protocol format
    async fn export_influxdb(&self, data: &ExportData, filepath: &str) -> Result<String> {
        let mut influx_content = String::new();
        let timestamp_ns = data.timestamp.timestamp_nanos_opt().unwrap_or(0);
        
        // Counters
        for (name, value) in &data.summary.counters {
            influx_content.push_str(&format!("router_flood,metric_type=counter {}={} {}\n", 
                name, value, timestamp_ns));
        }
        
        // Gauges
        for (name, value) in &data.summary.gauges {
            influx_content.push_str(&format!("router_flood,metric_type=gauge {}={} {}\n", 
                name, value, timestamp_ns));
        }
        
        // Timers
        for (name, value) in &data.summary.timers {
            influx_content.push_str(&format!("router_flood,metric_type=timer {}={} {}\n", 
                name, value, timestamp_ns));
        }
        
        fs::write(filepath, influx_content)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write InfluxDB format: {}", e)))?;
        
        Ok(filepath.to_string())
    }
    
    /// Export in custom format
    async fn export_custom(&self, data: &ExportData, filepath: &str, format: &str) -> Result<String> {
        // For now, just export as JSON with custom extension
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| StatsError::SerializationError(format!("Failed to serialize custom format: {}", e)))?;
        
        let custom_filepath = format!("{}.{}", filepath, format);
        fs::write(&custom_filepath, json)
            .await
            .map_err(|e| StatsError::FileWriteError(format!("Failed to write custom format: {}", e)))?;
        
        Ok(custom_filepath)
    }
}

/// Complete export data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub timestamp: chrono::DateTime<Utc>,
    pub summary: MetricsSummary,
    pub metadata: ExportMetadata,
    pub history: Option<HashMap<String, Vec<super::metrics::MetricValue>>>,
}

/// Export metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub version: String,
    pub export_format: String,
    pub custom_fields: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_metrics_exporter() {
        let temp_dir = TempDir::new().unwrap();
        let collector = Arc::new(MetricsCollector::new());
        
        let config = ExportConfig {
            enabled: true,
            format: ExportFormat::Json,
            output_path: temp_dir.path().to_string_lossy().to_string(),
            filename_pattern: "test_metrics".to_string(),
            include_history: false,
            compression: false,
            custom_fields: HashMap::new(),
        };
        
        let exporter = MetricsExporter::new(collector.clone(), config);
        
        // Add some test metrics
        let metric = collector.register_metric(
            "test_counter".to_string(),
            super::super::metrics::MetricType::Counter,
            "Test counter".to_string(),
        );
        metric.update(42.0, HashMap::new());
        
        // Export metrics
        let result = exporter.export_metrics().await;
        assert!(result.is_ok());
        
        let filepath = result.unwrap();
        assert!(tokio::fs::metadata(&filepath).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_export_formats() {
        let temp_dir = TempDir::new().unwrap();
        let collector = Arc::new(MetricsCollector::new());
        
        let formats = vec![
            ExportFormat::Json,
            ExportFormat::Csv,
            ExportFormat::Prometheus,
            ExportFormat::InfluxDB,
        ];
        
        for format in formats {
            let config = ExportConfig {
                enabled: true,
                format: format.clone(),
                output_path: temp_dir.path().to_string_lossy().to_string(),
                filename_pattern: format!("test_{:?}", format),
                include_history: false,
                compression: false,
                custom_fields: HashMap::new(),
            };
            
            let exporter = MetricsExporter::new(collector.clone(), config);
            let result = exporter.export_metrics().await;
            assert!(result.is_ok(), "Failed to export format: {:?}", format);
        }
    }
}