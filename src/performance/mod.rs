//! Performance optimization modules
//!
//! This module contains various performance optimizations including
//! SIMD operations, CPU affinity management, and advanced memory management.

pub mod advanced_buffer_pool;
pub mod buffer_pool;
pub mod constants;
pub mod cpu_affinity;
pub mod inline_hints;
pub mod lockfree_stats;
pub mod memory_pool;
pub mod optimized_constants;
pub mod optimized_pipeline;
pub mod simd_packet;
pub mod string_interning;
pub mod unified_buffer_pool;
pub mod zero_copy;

// Re-export commonly used types
pub use advanced_buffer_pool::{AdvancedBufferPool, AlignedBuffer};
pub use buffer_pool::{LockFreeBufferPool, SharedBufferPool};
pub use cpu_affinity::{CpuAffinityManager, CpuTopology};
pub use lockfree_stats::{
    LockFreeStatsCollector, BatchedStatsCollector, StatsSnapshot, PerCpuStats
};
pub use memory_pool::{
    LockFreeMemoryPool, MemoryPoolManager, ManagedMemory, PoolStats as MemoryPoolStats
};
pub use optimized_pipeline::{
    OptimizedPacketProcessor, ProcessedPacket, PipelineMetrics
};
pub use simd_packet::SimdPacketBuilder;
pub use string_interning::{
    InternedString, StringInterner, GlobalStringInterner, intern, protocols, errors, fields
};
pub use unified_buffer_pool::{
    UnifiedBufferPool, BufferPoolFactory, ContentionLevel, PoolStats
};
pub use zero_copy::{
    ZeroCopyBuffer, ZeroCopyStr, ZeroCopyBufferPool, ZeroCopyPacketBuilder
};