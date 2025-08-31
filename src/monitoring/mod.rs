//! Monitoring and metrics collection
//!
//! This module provides comprehensive monitoring capabilities including
//! Prometheus metrics, alerting, and dashboard functionality.
//!
//! ## Enhanced Monitoring
//!
//! Enhanced monitoring adds lightweight dashboard capabilities:
//! - Dashboard with essential metrics
//! - Configurable alert thresholds
//! - Compact and full display modes
//! - System information integration

pub mod alerts;
pub mod dashboard;
pub mod export;
pub mod metrics;
pub mod prometheus;
pub mod essential;


pub use alerts::{Alerts, AlertRule};
pub use dashboard::{Dashboard, DashboardConfig, DashboardBuilder, AlertThresholds, DashboardState, Alert, AlertLevel, SystemInfo};
pub use export::{MetricsExporter, ExportFormat};
pub use metrics::{MetricsCollector, MetricValue};
pub use prometheus::PrometheusExporter;
pub use essential::{
    Metrics, EssentialMetricsCollector, Display, Exporter,
    Monitor, Monitoring, MonitoringSystem
};
