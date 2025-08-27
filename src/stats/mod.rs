//! Statistics collection and export system
//!
//! This module provides a trait-based architecture for statistics collection
//! with support for different collection strategies and export formats.

pub mod collector;
pub mod export;

pub use collector::{StatsCollector, SessionStats, SystemStats};
pub use export::StatsExporter;

// Re-export the main FloodStats for backward compatibility
pub use crate::stats_original::FloodStats;