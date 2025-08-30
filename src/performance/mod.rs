//! Performance optimization modules
//!
//! This module contains various performance optimizations including
//! SIMD operations, CPU affinity management, and advanced memory management.

pub mod constants;
pub mod cpu_affinity;
pub mod inline_hints;
pub mod lockfree_stats;
pub mod memory_pool;
pub mod lookup_tables;
pub mod batch_pipeline;
pub mod simd_packet;
pub mod string_interning;
pub mod zero_copy;

// Re-export commonly used types
pub use cpu_affinity::{CpuAffinityManager, CpuTopology};
pub use lockfree_stats::{
    LockFreeStatsCollector, BatchedStatsCollector, StatsSnapshot, PerCpuStats
};
pub use memory_pool::{
    LockFreeMemoryPool, MemoryPoolManager, ManagedMemory, PoolStats as MemoryPoolStats
};
pub use batch_pipeline::{
    BatchPacketProcessor, ProcessedPacket, PipelineMetrics
};
pub use simd_packet::SimdPacketBuilder;
pub use string_interning::{
    InternedString, StringInterner, GlobalStringInterner, intern, protocols, errors, fields
};

// Re-export the main buffer pool from utils
pub use crate::utils::buffer_pool::BufferPool;
pub use zero_copy::{
    ZeroCopyBuffer, ZeroCopyStr, ZeroCopyBufferPool, ZeroCopyPacketBuilder
};