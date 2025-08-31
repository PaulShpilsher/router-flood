//! Statistics collection and export system
//!
//! This module provides a trait-based architecture for statistics collection
//! with support for different collection strategies and export formats.

pub mod adapter;
pub mod collector;
pub mod export;
pub mod local;
pub mod lockfree;
pub mod observer;
pub mod display;
pub mod stats_collector;

pub use adapter::{LockFreeStatsAdapter, LocalStatsExt};
pub use collector::{StatsCollector, SessionStats, SystemStats};
pub use export::StatsExporter;
pub use local::LocalStats;
pub use lockfree::{LockFreeStats, LockFreeLocalStats, PerCpuStats, ProtocolId, StatsSnapshot};
pub use observer::{StatsObserver, StatsSubject, StatsEvent, ObserverBuilder};
pub use display::{StatsDisplay, init_display, get_display};
pub use stats_collector::FloodStatsTracker;

// Re-export the internal lock-free implementation for advanced users
pub use crate::performance::lockfree_stats::LockFreeStatsCollector as InternalLockFreeCollector;

// Include the protocol breakdown module
pub mod protocol_breakdown;
pub use protocol_breakdown::{ProtocolBreakdown, ProtocolIndex};
