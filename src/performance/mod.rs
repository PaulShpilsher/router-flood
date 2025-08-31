//! Performance optimization modules
//!
//! This module contains critical performance optimizations including
//! SIMD operations, CPU affinity management, lock-free memory pools,
//! and zero-copy packet construction.

pub mod cpu_affinity;
pub mod memory_pool;
pub mod batch_pipeline;
pub mod simd_packet;
pub mod zero_copy;

// Re-export commonly used types
pub use cpu_affinity::{CpuAffinity, CpuTopology};
pub use memory_pool::{
    LockFreeMemoryPool, Memory, ManagedMemory, PoolStats as MemoryPoolStats
};
pub use batch_pipeline::{
    BatchPacketProcessor, ProcessedPacket, PipelineMetrics
};
pub use simd_packet::SimdPacketBuilder;

// Re-export the main buffer pool from utils
pub use crate::utils::buffer_pool::BufferPool;
pub use zero_copy::{
    ZeroCopyBuffer, ZeroCopyStr, ZeroCopyBufferPool, ZeroCopyPacketBuilder
};