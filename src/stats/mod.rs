//! Statistics collection and export system
//!
//! This module provides a trait-based architecture for statistics collection
//! with support for different collection strategies and export formats.

pub mod adapter;
pub mod collector;
pub mod export;
pub mod batch_accumulator;
pub mod lockfree;
pub mod observer;
pub mod display;
pub mod stats_aggregator;
pub mod internal_lockfree;

pub use adapter::{LockFreeStatsAdapter, BatchStatsExt};
pub use collector::{StatsCollector, SessionStats, SystemStats};
pub use export::StatsExporter;
pub use batch_accumulator::BatchAccumulator;
pub use lockfree::{LockFreeStats, LockFreeLocalStats, PerCpuStats, ProtocolId, StatsSnapshot};
pub use observer::{StatsObserver, StatsSubject, StatsEvent, ObserverBuilder};
pub use display::{StatsDisplay, init_display, get_display};
pub use stats_aggregator::StatsAggregator;

// Re-export the internal lock-free implementation for advanced users
pub use internal_lockfree::LockFreeStatsCollector as InternalLockFreeCollector;

// Include the protocol breakdown module
pub mod protocol_breakdown;
pub use protocol_breakdown::{ProtocolBreakdown, ProtocolIndex};
