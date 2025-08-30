//! Monitoring and metrics collection
//!
//! This module provides comprehensive monitoring capabilities including
//! Prometheus metrics, alerting, and dashboard functionality.
//!
//! ## Phase 5 Enhancements
//!
//! Phase 5 adds lightweight real-time dashboard capabilities:
//! - Real-time dashboard with essential metrics
//! - Configurable alert thresholds
//! - Compact and full display modes
//! - System information integration

pub mod alerts;
pub mod dashboard;
pub mod export;
pub mod metrics;
pub mod prometheus;
pub mod simplified;
pub mod realtime_dashboard;

pub use alerts::{AlertManager, AlertRule};
pub use dashboard::PerformanceDashboard;
pub use export::{MetricsExporter, ExportFormat};
pub use metrics::{MetricsCollector, MetricValue};
pub use prometheus::PrometheusExporter;
pub use simplified::{
    EssentialMetrics, SimpleMetricsCollector, SimpleDisplay, SimpleExporter,
    SimpleMonitor, SimpleMonitoringConfig, SimpleMonitoringSystem
};
// Temporarily commented for compilation
// pub use realtime_dashboard::{
//     RealtimeDashboard,
//     DashboardConfig,
//     DashboardBuilder,
//     AlertThresholds,
//     DashboardState,
//     Alert,
//     AlertLevel,
//     SystemInfo,
// };