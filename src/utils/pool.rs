//! Buffer pool traits and adapters
//!
//! This module provides a unified interface for buffer pool implementations,
//! along with adapters and utilities for pool management.

use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use crate::utils::buffer_pool::BufferPool;

// ===== Core Traits =====

/// Core buffer pool trait for getting and returning buffers
pub trait BufferPoolTrait: Send + Sync {
    /// The type of buffer this pool manages
    type Buffer: Send;
    
    /// Get a buffer from the pool
    /// 
    /// Returns None if no buffer is available
    fn get(&self) -> Option<Self::Buffer>;
    
    /// Return a buffer to the pool for reuse
    fn put(&self, buffer: Self::Buffer);
    
    /// Get the capacity of the pool
    fn capacity(&self) -> usize;
    
    /// Get the number of available buffers
    fn available(&self) -> usize;
}

/// Extended buffer pool with size requirements
pub trait SizedBufferPool: BufferPoolTrait {
    /// Get a buffer with minimum size requirement
    fn get_with_size(&self, min_size: usize) -> Option<Self::Buffer>;
}

/// Buffer pool with statistics tracking
pub trait ObservablePool: BufferPoolTrait {
    /// Statistics type for this pool
    type Stats: PoolStatistics;
    
    /// Get current pool statistics
    fn statistics(&self) -> Self::Stats;
}

/// Common statistics interface for buffer pools
pub trait PoolStatistics: Debug + Clone {
    /// Total number of allocations
    fn total_allocations(&self) -> u64;
    
    /// Total number of successful gets
    fn total_gets(&self) -> u64;
    
    /// Total number of returns
    fn total_returns(&self) -> u64;
    
    /// Current number of buffers in use
    fn in_use(&self) -> usize;
    
    /// Hit rate percentage (0.0 - 100.0)
    fn hit_rate(&self) -> f64 {
        let gets = self.total_gets() as f64;
        let allocs = self.total_allocations() as f64;
        if gets > 0.0 {
            ((gets - allocs) / gets) * 100.0
        } else {
            0.0
        }
    }
}

/// Builder pattern for buffer pools
pub trait PoolBuilder {
    /// The type of pool this builder creates
    type Pool: BufferPoolTrait;
    
    /// Set the buffer size
    fn buffer_size(self, size: usize) -> Self;
    
    /// Set the pool capacity
    fn capacity(self, capacity: usize) -> Self;
    
    /// Build the pool
    fn build(self) -> Self::Pool;
}

// ===== Statistics Implementation =====

/// Generic pool statistics implementation
#[derive(Debug, Clone, Default)]
pub struct BasicPoolStats {
    pub allocations: u64,
    pub gets: u64,
    pub returns: u64,
    pub current_size: usize,
}

impl PoolStatistics for BasicPoolStats {
    fn total_allocations(&self) -> u64 {
        self.allocations
    }
    
    fn total_gets(&self) -> u64 {
        self.gets
    }
    
    fn total_returns(&self) -> u64 {
        self.returns
    }
    
    fn in_use(&self) -> usize {
        (self.gets - self.returns) as usize
    }
}

// ===== Adapter Implementations =====

impl BufferPoolTrait for BufferPool {
    type Buffer = Vec<u8>;
    
    #[inline]
    fn get(&self) -> Option<Self::Buffer> {
        // BufferPool always returns a buffer (allocates if needed)
        Some(self.buffer())
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

// ===== Observable Pool Wrapper =====

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

// ===== Macro for implementing BufferPoolTrait =====

/// Macro to implement BufferPoolTrait for types with buffer/return_buffer methods
#[macro_export]
macro_rules! impl_buffer_pool {
    ($type:ty, $buffer:ty) => {
        impl $crate::utils::pool::BufferPoolTrait for $type {
            type Buffer = $buffer;
            
            fn get(&self) -> Option<Self::Buffer> {
                self.buffer()
            }
            
            fn put(&self, buffer: Self::Buffer) {
                self.return_buffer(buffer)
            }
            
            fn capacity(&self) -> usize {
                // Default implementation, override if available
                100
            }
            
            fn available(&self) -> usize {
                // Default implementation, override if available
                0
            }
        }
    };
}