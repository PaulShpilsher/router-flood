//! Advanced monitoring and observability system
//!
//! This module provides real-time performance monitoring, metrics collection,
//! and alerting capabilities for the router-flood tool.

pub mod dashboard;
pub mod metrics;
pub mod alerts;
pub mod export;

pub use dashboard::PerformanceDashboard;
pub use metrics::{MetricsCollector, MetricType, MetricValue};
pub use alerts::{AlertManager, AlertRule};
pub use dashboard::AlertLevel;
pub use export::{MetricsExporter, ExportFormat};