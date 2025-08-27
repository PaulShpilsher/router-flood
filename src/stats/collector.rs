//! Statistics collection traits and types

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use crate::error::Result;

/// Core trait for statistics collection
pub trait StatsCollector: Send + Sync {
    /// Increment sent packet count
    fn increment_sent(&self, bytes: u64, protocol: &str);
    
    /// Increment failed packet count
    fn increment_failed(&self);
    
    /// Get current session statistics
    fn get_stats(&self) -> SessionStats;
    
    /// Print current statistics to stdout
    fn print_stats(&self, system_stats: Option<&SystemStats>);
    
    /// Get the session ID
    fn session_id(&self) -> &str;
}

/// Separate trait for async export functionality to maintain dyn compatibility
pub trait StatsExporter: Send + Sync {
    /// Export statistics if configured
    async fn export_stats(&self, system_stats: Option<&SystemStats>) -> Result<()>;
}

/// Session statistics snapshot
#[derive(Debug, Serialize, Clone)]
pub struct SessionStats {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub bytes_sent: u64,
    pub duration_secs: f64,
    pub packets_per_second: f64,
    pub megabits_per_second: f64,
    pub protocol_breakdown: HashMap<String, u64>,
    pub system_stats: Option<SystemStats>,
}

/// System resource statistics
#[derive(Debug, Serialize, Clone)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub network_sent: u64,
    pub network_received: u64,
}