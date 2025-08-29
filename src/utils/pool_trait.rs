//! Unified buffer pool trait for consistent abstraction
//!
//! This module provides a common interface for all buffer pool implementations,
//! following the Interface Segregation Principle and Dependency Inversion Principle.

use std::fmt::Debug;

/// Core buffer pool trait for getting and returning buffers
pub trait BufferPool: Send + Sync {
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
pub trait SizedBufferPool: BufferPool {
    /// Get a buffer with minimum size requirement
    fn get_with_size(&self, min_size: usize) -> Option<Self::Buffer>;
}

/// Buffer pool with statistics tracking
pub trait ObservablePool: BufferPool {
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
    type Pool: BufferPool;
    
    /// Set the buffer size
    fn buffer_size(self, size: usize) -> Self;
    
    /// Set the pool capacity
    fn capacity(self, capacity: usize) -> Self;
    
    /// Build the pool
    fn build(self) -> Self::Pool;
}

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


/// Macro to implement BufferPool trait for types with get_buffer/return_buffer methods
#[macro_export]
macro_rules! impl_buffer_pool {
    ($type:ty, $buffer:ty) => {
        impl $crate::utils::pool_trait::BufferPool for $type {
            type Buffer = $buffer;
            
            fn get(&self) -> Option<Self::Buffer> {
                self.get_buffer()
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