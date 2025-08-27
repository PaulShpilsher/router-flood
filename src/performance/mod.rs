//! Performance optimization utilities and implementations
//!
//! This module contains optimized implementations for performance-critical
//! components including lock-free data structures and optimized algorithms.

pub mod buffer_pool;
pub mod inline_hints;
pub mod constants;
pub mod optimized_constants;

pub use buffer_pool::{LockFreeBufferPool, SharedBufferPool};
pub use inline_hints::*;
pub use optimized_constants::*;