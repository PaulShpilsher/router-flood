//! Adapter implementations for existing buffer pools
//!
//! This module provides trait implementations for existing buffer pool types,
//! maintaining backward compatibility while providing a consistent interface.

use super::pool_trait::{BufferPool as BufferPoolTrait, ObservablePool, PoolStatistics, BasicPoolStats};
use crate::performance::numa_buffer_pool::{NumaBufferPool, AlignedBuffer, PoolStatistics as NumaStats};
use crate::utils::buffer_pool::BufferPool;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

// ===== Main BufferPool Implementation =====

impl BufferPoolTrait for BufferPool {
    type Buffer = Vec<u8>;
    
    #[inline]
    fn get(&self) -> Option<Self::Buffer> {
        // BufferPool always returns a buffer (allocates if needed)
        Some(self.get_buffer())
    }
    
    #[inline]
    fn put(&self, buffer: Self::Buffer) {
        self.return_buffer(buffer)
    }
    
    fn capacity(&self) -> usize {
        self.pool_size()
    }
    
    fn available(&self) -> usize {
        // Use utilization method to calculate available buffers
        let utilization = self.utilization();
        (self.pool_size() as f64 * utilization) as usize
    }
}

// ===== NumaBufferPool Implementation =====

impl BufferPoolTrait for NumaBufferPool {
    type Buffer = AlignedBuffer;
    
    #[inline]
    fn get(&self) -> Option<Self::Buffer> {
        // Default to getting a buffer of default size
        self.get_buffer(1024)
    }
    
    #[inline]
    fn put(&self, buffer: Self::Buffer) {
        self.return_buffer(buffer)
    }
    
    fn capacity(&self) -> usize {
        // Sum of all size class capacities
        800 // 8 size classes * 100 buffers each
    }
    
    fn available(&self) -> usize {
        // Would need to lock all pools to count
        0 // Conservative estimate
    }
}

impl ObservablePool for NumaBufferPool {
    type Stats = NumaPoolStatsAdapter;
    
    fn statistics(&self) -> Self::Stats {
        NumaPoolStatsAdapter(self.get_stats())
    }
}

// ===== Statistics Adapter =====

/// Adapter to convert NumaBufferPool statistics to our trait
pub struct NumaPoolStatsAdapter(pub NumaStats);

impl PoolStatistics for NumaPoolStatsAdapter {
    fn total_allocations(&self) -> u64 {
        self.0.total_allocated as u64
    }
    
    fn total_gets(&self) -> u64 {
        self.0.total_hits as u64
    }
    
    fn total_returns(&self) -> u64 {
        // NumaStats doesn't track returns separately
        self.0.total_hits as u64
    }
    
    fn in_use(&self) -> usize {
        self.0.memory_usage / 1024 // Estimate based on memory usage
    }
}

impl std::fmt::Debug for NumaPoolStatsAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PoolStats")
            .field("allocations", &self.total_allocations())
            .field("gets", &self.total_gets())
            .field("returns", &self.total_returns())
            .field("in_use", &self.in_use())
            .field("hit_rate", &self.hit_rate())
            .finish()
    }
}

impl Clone for NumaPoolStatsAdapter {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// ===== Observable Pool with Statistics Tracking =====

/// Wrapper to add statistics tracking to any buffer pool
pub struct ObservablePoolWrapper<P: BufferPoolTrait> {
    pool: P,
    stats: Arc<InternalStats>,
}

struct InternalStats {
    allocations: AtomicU64,
    gets: AtomicU64,
    returns: AtomicU64,
    current_size: AtomicUsize,
}

impl<P: BufferPoolTrait> ObservablePoolWrapper<P> {
    /// Create a new observable wrapper around a pool
    pub fn new(pool: P) -> Self {
        Self {
            pool,
            stats: Arc::new(InternalStats {
                allocations: AtomicU64::new(0),
                gets: AtomicU64::new(0),
                returns: AtomicU64::new(0),
                current_size: AtomicUsize::new(0),
            }),
        }
    }
}

impl<P: BufferPoolTrait> BufferPoolTrait for ObservablePoolWrapper<P> {
    type Buffer = P::Buffer;
    
    fn get(&self) -> Option<Self::Buffer> {
        self.stats.gets.fetch_add(1, Ordering::Relaxed);
        match self.pool.get() {
            Some(buffer) => {
                self.stats.current_size.fetch_add(1, Ordering::Relaxed);
                Some(buffer)
            }
            None => {
                self.stats.allocations.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }
    
    fn put(&self, buffer: Self::Buffer) {
        self.stats.returns.fetch_add(1, Ordering::Relaxed);
        self.stats.current_size.fetch_sub(1, Ordering::Relaxed);
        self.pool.put(buffer)
    }
    
    fn capacity(&self) -> usize {
        self.pool.capacity()
    }
    
    fn available(&self) -> usize {
        self.pool.available()
    }
}

impl<P: BufferPoolTrait> ObservablePool for ObservablePoolWrapper<P> {
    type Stats = BasicPoolStats;
    
    fn statistics(&self) -> Self::Stats {
        BasicPoolStats {
            allocations: self.stats.allocations.load(Ordering::Relaxed),
            gets: self.stats.gets.load(Ordering::Relaxed),
            returns: self.stats.returns.load(Ordering::Relaxed),
            current_size: self.stats.current_size.load(Ordering::Relaxed),
        }
    }
}

// ===== Helper Functions =====

/// Create a buffer pool based on configuration
pub fn create_pool(_pool_type: &str, buffer_size: usize, capacity: usize) -> Box<dyn BufferPoolTrait<Buffer = Vec<u8>>> {
    // Always use the high-performance BufferPool
    Box::new(BufferPool::new(buffer_size, capacity))
}

/// Create an observable pool with statistics
pub fn create_observable_pool<P: BufferPoolTrait + 'static>(pool: P) -> ObservablePoolWrapper<P> {
    ObservablePoolWrapper::new(pool)
}