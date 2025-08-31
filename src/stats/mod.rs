//! Statistics collection and export system
//!
//! Simplified statistics module for tracking packet generation metrics.

pub mod collector;
pub mod export;
pub mod display;
pub mod stats_aggregator;
pub mod protocol_breakdown;

// Main stats implementation
pub use stats_aggregator::{Stats, BatchStats};

// Core types
pub use collector::{SessionStats, SystemStats};
pub use export::StatsExporter;
pub use display::{init_display, display};
pub use protocol_breakdown::ProtocolBreakdown;