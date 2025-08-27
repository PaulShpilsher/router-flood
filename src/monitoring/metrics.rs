//! Real-time metrics collection system

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicU64 as AtomicF64; // Use AtomicU64 for f64 storage
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Types of metrics that can be collected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// Metric value with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

/// Individual metric with history
#[derive(Debug)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub current_value: AtomicF64,
    pub history: RwLock<Vec<MetricValue>>,
    pub max_history: usize,
}

impl Metric {
    pub fn new(name: String, metric_type: MetricType, description: String) -> Self {
        Self {
            name,
            metric_type,
            description,
            current_value: AtomicF64::new(0.0_f64.to_bits()),
            history: RwLock::new(Vec::new()),
            max_history: 1000, // Keep last 1000 values
        }
    }
    
    /// Update metric value
    pub fn update(&self, value: f64, labels: HashMap<String, String>) {
        self.current_value.store(value.to_bits(), Ordering::Relaxed);
        
        let metric_value = MetricValue {
            value,
            timestamp: Utc::now(),
            labels,
        };
        
        if let Ok(mut history) = self.history.write() {
            history.push(metric_value);
            
            // Keep only recent history
            if history.len() > self.max_history {
                let excess = history.len() - self.max_history;
                history.drain(0..excess);
            }
        }
    }
    
    /// Get current value
    pub fn current_value(&self) -> f64 {
        f64::from_bits(self.current_value.load(Ordering::Relaxed))
    }
    
    /// Get recent history
    pub fn recent_history(&self, duration: Duration) -> Vec<MetricValue> {
        let cutoff = Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default();
        
        if let Ok(history) = self.history.read() {
            history.iter()
                .filter(|v| v.timestamp > cutoff)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// High-performance metrics collector
pub struct MetricsCollector {
    metrics: RwLock<HashMap<String, Arc<Metric>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }
    
    /// Register a new metric
    pub fn register_metric(&self, name: String, metric_type: MetricType, description: String) -> Arc<Metric> {
        let metric = Arc::new(Metric::new(name.clone(), metric_type, description));
        
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.insert(name, Arc::clone(&metric));
        }
        
        metric
    }
    
    /// Update a metric value
    pub fn update_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        if let Ok(metrics) = self.metrics.read() {
            if let Some(metric) = metrics.get(name) {
                metric.update(value, labels);
            }
        }
    }
    
    /// Increment a counter metric
    pub fn increment_counter(&self, name: &str, labels: HashMap<String, String>) {
        if let Ok(metrics) = self.metrics.read() {
            if let Some(metric) = metrics.get(name) {
                let current = metric.current_value();
                metric.update(current + 1.0, labels);
            }
        }
    }
    
    /// Record a timer value
    pub fn record_timer(&self, name: &str, duration: Duration, labels: HashMap<String, String>) {
        let value = duration.as_secs_f64() * 1000.0; // Convert to milliseconds
        self.update_metric(name, value, labels);
    }
    
    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<String, Arc<Metric>> {
        if let Ok(metrics) = self.metrics.read() {
            metrics.clone()
        } else {
            HashMap::new()
        }
    }
    
    /// Get uptime in seconds
    pub fn uptime(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
    
    /// Get metrics summary
    pub fn get_summary(&self) -> MetricsSummary {
        let metrics = self.get_all_metrics();
        let mut summary = MetricsSummary {
            uptime_seconds: self.uptime(),
            total_metrics: metrics.len(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
            timers: HashMap::new(),
        };
        
        for (name, metric) in metrics {
            let current_value = metric.current_value();
            match metric.metric_type {
                MetricType::Counter => {
                    summary.counters.insert(name, current_value);
                }
                MetricType::Gauge => {
                    summary.gauges.insert(name, current_value);
                }
                MetricType::Timer => {
                    summary.timers.insert(name, current_value);
                }
                MetricType::Histogram => {
                    // For now, treat histograms as gauges
                    summary.gauges.insert(name, current_value);
                }
            }
        }
        
        summary
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of all metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub uptime_seconds: f64,
    pub total_metrics: usize,
    pub counters: HashMap<String, f64>,
    pub gauges: HashMap<String, f64>,
    pub timers: HashMap<String, f64>,
}

/// Pre-defined metrics for router-flood
pub struct RouterFloodMetrics {
    pub packets_sent: Arc<Metric>,
    pub packets_failed: Arc<Metric>,
    pub bytes_sent: Arc<Metric>,
    pub packet_build_time: Arc<Metric>,
    pub send_time: Arc<Metric>,
    pub buffer_pool_utilization: Arc<Metric>,
    pub worker_threads_active: Arc<Metric>,
    pub cpu_usage: Arc<Metric>,
    pub memory_usage: Arc<Metric>,
    pub network_throughput: Arc<Metric>,
}

impl RouterFloodMetrics {
    pub fn new(collector: &MetricsCollector) -> Self {
        Self {
            packets_sent: collector.register_metric(
                "packets_sent_total".to_string(),
                MetricType::Counter,
                "Total number of packets sent".to_string(),
            ),
            packets_failed: collector.register_metric(
                "packets_failed_total".to_string(),
                MetricType::Counter,
                "Total number of failed packet sends".to_string(),
            ),
            bytes_sent: collector.register_metric(
                "bytes_sent_total".to_string(),
                MetricType::Counter,
                "Total bytes sent".to_string(),
            ),
            packet_build_time: collector.register_metric(
                "packet_build_duration_ms".to_string(),
                MetricType::Timer,
                "Time taken to build packets in milliseconds".to_string(),
            ),
            send_time: collector.register_metric(
                "packet_send_duration_ms".to_string(),
                MetricType::Timer,
                "Time taken to send packets in milliseconds".to_string(),
            ),
            buffer_pool_utilization: collector.register_metric(
                "buffer_pool_utilization_ratio".to_string(),
                MetricType::Gauge,
                "Buffer pool utilization ratio (0.0 to 1.0)".to_string(),
            ),
            worker_threads_active: collector.register_metric(
                "worker_threads_active".to_string(),
                MetricType::Gauge,
                "Number of active worker threads".to_string(),
            ),
            cpu_usage: collector.register_metric(
                "cpu_usage_percent".to_string(),
                MetricType::Gauge,
                "CPU usage percentage".to_string(),
            ),
            memory_usage: collector.register_metric(
                "memory_usage_bytes".to_string(),
                MetricType::Gauge,
                "Memory usage in bytes".to_string(),
            ),
            network_throughput: collector.register_metric(
                "network_throughput_mbps".to_string(),
                MetricType::Gauge,
                "Network throughput in Mbps".to_string(),
            ),
        }
    }
}

