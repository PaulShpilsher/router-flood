//! Performance optimization modules
//!
//! This module contains various performance optimizations including
//! buffer pools, inline hints, SIMD optimizations, CPU affinity, and optimized constants.

pub mod advanced_buffer_pool;
pub mod buffer_pool;
pub mod cpu_affinity;
pub mod inline_hints;
pub mod optimized_constants;
pub mod simd_packet;

pub use advanced_buffer_pool::{AdvancedBufferPool, AlignedBuffer, PoolStatistics};
pub use buffer_pool::{LockFreeBufferPool, SharedBufferPool};
pub use cpu_affinity::{CpuAffinityManager, CpuTopology, CpuAssignment};
pub use optimized_constants::*;
pub use simd_packet::{SimdPacketBuilder, SimdPerformanceInfo};