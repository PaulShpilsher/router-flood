//! Monitoring and metrics collection
//!
//! This module provides comprehensive monitoring capabilities including
//! Prometheus metrics, alerting, and dashboard functionality.

pub mod alerts;
pub mod dashboard;
pub mod export;
pub mod metrics;
pub mod prometheus;
pub mod simplified;

pub use alerts::{AlertManager, AlertRule};
pub use dashboard::PerformanceDashboard;
pub use export::{MetricsExporter, ExportFormat};
pub use metrics::{MetricsCollector, MetricValue};
pub use prometheus::PrometheusExporter;
pub use simplified::{
    EssentialMetrics, SimpleMetricsCollector, SimpleDisplay, SimpleExporter,
    SimpleMonitor, SimpleMonitoringConfig, SimpleMonitoringSystem
};