//! Performance optimization modules
//!
//! This module contains various performance optimizations including
//! SIMD operations, CPU affinity management, and buffer pooling.

pub mod advanced_buffer_pool;
pub mod buffer_pool;
pub mod constants;
pub mod cpu_affinity;
pub mod inline_hints;
pub mod optimized_constants;
pub mod simd_packet;
pub mod unified_buffer_pool;

// Re-export commonly used types
pub use advanced_buffer_pool::{AdvancedBufferPool, AlignedBuffer};
pub use buffer_pool::{LockFreeBufferPool, SharedBufferPool};
pub use cpu_affinity::{CpuAffinityManager, CpuTopology};
pub use simd_packet::SimdPacketBuilder;
pub use unified_buffer_pool::{
    UnifiedBufferPool, BufferPoolFactory, ContentionLevel, PoolStats
};