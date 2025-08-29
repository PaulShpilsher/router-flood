//! Utility modules
//!
//! This module contains utility functions and helpers used throughout the application.

pub mod buffer_pool;
pub mod raii;
pub mod rng;
pub mod terminal;

// Re-export commonly used utilities
pub use buffer_pool::{BufferPool, WorkerBufferPool};
pub use raii::{ResourceGuard, WorkerGuard, SignalGuard, StatsGuard, TerminalRAIIGuard};
pub use rng::BatchedRng;
pub use terminal::TerminalController;