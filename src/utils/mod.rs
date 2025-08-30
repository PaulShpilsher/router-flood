//! Utility modules and helper functions
//!
//! This module contains various utility functions and helper modules
//! used throughout the application.

pub mod buffer_pool;
pub mod pool_adapters;
pub mod pool_trait;
pub mod protocol_utils;
pub mod raii;
pub mod rng;
pub mod shared;
pub mod terminal;

// Re-export commonly used types
pub use buffer_pool::BufferPool;
pub use pool_trait::{BufferPool as BufferPoolTrait, ObservablePool, PoolStatistics, SizedBufferPool};
pub use protocol_utils::PacketTypeExt;
pub use raii::{ResourceGuard, SignalGuard, StatsGuard};
pub use rng::BatchedRng;
pub use shared::{
    AtomicCounter, RunningFlag, RateCalculator, JitterApplier,
    calculate_percentage, calculate_success_rate, calculate_bandwidth_mbps,
    format_bytes, format_duration, validation, RetryConfig, retry_with_backoff
};
pub use terminal::TerminalGuard;