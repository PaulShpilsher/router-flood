//! Statistics collection and export system
//!
//! Simplified statistics module with lock-free collection and export capabilities.

pub mod collector;
pub mod export;
pub mod display;

// Main stats implementation that combines aggregation and lock-free collection
pub mod stats_aggregator;
pub use stats_aggregator::Stats;

// Batch statistics for workers
pub mod batch_accumulator;
pub use batch_accumulator::BatchStats;

// Core types
pub use collector::{SessionStats, SystemStats};
pub use export::StatsExporter;
pub use display::{init_display, display};

// Internal lock-free implementation
pub mod internal_lockfree;

// Protocol breakdown tracking
pub mod protocol_breakdown;
pub use protocol_breakdown::ProtocolBreakdown;