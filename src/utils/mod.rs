//! Utility modules
//!
//! This module contains utility functions and helpers used throughout the application.

pub mod buffer_pool;
pub mod pool_trait;
pub mod pool_adapters;
pub mod raii;
pub mod rng;
pub mod terminal;

// Re-export commonly used utilities
pub use buffer_pool::{BufferPool, WorkerBufferPool};
pub use pool_trait::{BufferPool as BufferPoolTrait, ObservablePool, PoolStatistics, SizedBufferPool};
pub use pool_adapters::{create_pool, create_observable_pool};
pub use raii::{ResourceGuard, WorkerGuard, SignalGuard, StatsGuard, TerminalRAIIGuard};
pub use rng::BatchedRng;
pub use terminal::TerminalController;
pub mod protocol_utils;
pub use protocol_utils::{ProtocolUtils, PacketTypeExt};
