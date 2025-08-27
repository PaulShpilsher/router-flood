//! Advanced monitoring and observability system
//!
//! This module provides real-time performance monitoring, metrics collection,
//! alerting capabilities, and Prometheus metrics export for the router-flood tool.

pub mod dashboard;
pub mod metrics;
pub mod alerts;
pub mod export;
pub mod prometheus;

pub use dashboard::PerformanceDashboard;
pub use metrics::{MetricsCollector, MetricType, MetricValue};
pub use alerts::{AlertManager, AlertRule};
pub use dashboard::AlertLevel;
pub use export::{MetricsExporter, ExportFormat};
pub use prometheus::PrometheusExporter;