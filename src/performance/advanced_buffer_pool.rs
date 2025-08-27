//! Advanced buffer pool with memory alignment and NUMA awareness
//!
//! This module provides an enhanced buffer pool implementation with
//! memory alignment, NUMA awareness, and advanced allocation strategies.

use crate::error::{SystemError, Result};
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::collections::VecDeque;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Cache line size for alignment (typically 64 bytes on modern CPUs)
const CACHE_LINE_SIZE: usize = 64;

/// Memory-aligned buffer with automatic cleanup
pub struct AlignedBuffer {
    ptr: NonNull<u8>,
    size: usize,
    layout: Layout,
}

impl AlignedBuffer {
    /// Create a new aligned buffer
    pub fn new(size: usize, alignment: usize) -> Result<Self> {
        let layout = Layout::from_size_align(size, alignment)
            .map_err(|e| SystemError::ResourceUnavailable(format!("Invalid layout: {}", e)))?;

        let ptr = unsafe { alloc_zeroed(layout) };
        
        if ptr.is_null() {
            return Err(SystemError::ResourceUnavailable("Failed to allocate memory".to_string()).into());
        }

        Ok(Self {
            ptr: NonNull::new(ptr).unwrap(),
            size,
            layout,
        })
    }

    /// Get a mutable slice to the buffer data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }

    /// Get an immutable slice to the buffer data
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
    }

    /// Get the size of the buffer
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if the buffer is properly aligned
    pub fn is_aligned(&self, alignment: usize) -> bool {
        self.ptr.as_ptr() as usize % alignment == 0
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

unsafe impl Send for AlignedBuffer {}
unsafe impl Sync for AlignedBuffer {}

/// Advanced buffer pool with multiple size classes and NUMA awareness
pub struct AdvancedBufferPool {
    pools: Vec<SizeClassPool>,
    stats: PoolStats,
    numa_node: Option<usize>,
}

/// Pool for a specific buffer size class
struct SizeClassPool {
    size: usize,
    buffers: Mutex<VecDeque<AlignedBuffer>>,
    max_buffers: usize,
    allocated: AtomicUsize,
    hits: AtomicUsize,
    misses: AtomicUsize,
}

/// Buffer pool statistics
#[derive(Debug)]
pub struct PoolStats {
    pub total_allocated: AtomicUsize,
    pub total_hits: AtomicUsize,
    pub total_misses: AtomicUsize,
    pub memory_usage: AtomicUsize,
}

impl AdvancedBufferPool {
    /// Create a new advanced buffer pool
    pub fn new() -> Self {
        let size_classes = vec![64, 128, 256, 512, 1024, 1500, 2048, 4096];
        let pools = size_classes
            .into_iter()
            .map(|size| SizeClassPool::new(size, 100)) // Max 100 buffers per size class
            .collect();

        Self {
            pools,
            stats: PoolStats::new(),
            numa_node: Self::detect_numa_node(),
        }
    }

    /// Create a buffer pool with custom size classes
    pub fn with_size_classes(size_classes: Vec<usize>, max_buffers_per_class: usize) -> Self {
        let pools = size_classes
            .into_iter()
            .map(|size| SizeClassPool::new(size, max_buffers_per_class))
            .collect();

        Self {
            pools,
            stats: PoolStats::new(),
            numa_node: Self::detect_numa_node(),
        }
    }

    /// Get a buffer of at least the specified size
    pub fn get_buffer(&self, min_size: usize) -> Option<AlignedBuffer> {
        // Find the smallest size class that can accommodate the request
        for pool in &self.pools {
            if pool.size >= min_size {
                if let Some(buffer) = pool.get_buffer() {
                    self.stats.total_hits.fetch_add(1, Ordering::Relaxed);
                    return Some(buffer);
                }
            }
        }

        // No buffer available, try to allocate a new one
        self.stats.total_misses.fetch_add(1, Ordering::Relaxed);
        
        // Find the appropriate size class for allocation
        let target_size = self.pools
            .iter()
            .find(|pool| pool.size >= min_size)
            .map(|pool| pool.size)
            .unwrap_or(min_size.next_power_of_two().max(4096));

        match AlignedBuffer::new(target_size, CACHE_LINE_SIZE) {
            Ok(buffer) => {
                self.stats.total_allocated.fetch_add(1, Ordering::Relaxed);
                self.stats.memory_usage.fetch_add(target_size, Ordering::Relaxed);
                Some(buffer)
            }
            Err(_) => None,
        }
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, buffer: AlignedBuffer) {
        let size = buffer.size();
        
        // Find the matching size class
        for pool in &self.pools {
            if pool.size == size {
                if pool.return_buffer(buffer) {
                    return; // Successfully returned to pool
                }
                break; // Pool is full, buffer will be dropped
            }
        }
        
        // Buffer will be dropped here, update memory usage
        self.stats.memory_usage.fetch_sub(size, Ordering::Relaxed);
    }

    /// Get buffer pool statistics
    pub fn get_stats(&self) -> PoolStatistics {
        let mut size_class_stats = Vec::new();
        
        for pool in &self.pools {
            size_class_stats.push(SizeClassStats {
                size: pool.size,
                allocated: pool.allocated.load(Ordering::Relaxed),
                hits: pool.hits.load(Ordering::Relaxed),
                misses: pool.misses.load(Ordering::Relaxed),
                available: pool.buffers.lock().unwrap().len(),
                max_buffers: pool.max_buffers,
            });
        }

        PoolStatistics {
            total_allocated: self.stats.total_allocated.load(Ordering::Relaxed),
            total_hits: self.stats.total_hits.load(Ordering::Relaxed),
            total_misses: self.stats.total_misses.load(Ordering::Relaxed),
            memory_usage: self.stats.memory_usage.load(Ordering::Relaxed),
            hit_rate: self.calculate_hit_rate(),
            numa_node: self.numa_node,
            size_class_stats,
        }
    }

    /// Calculate the hit rate percentage
    fn calculate_hit_rate(&self) -> f64 {
        let hits = self.stats.total_hits.load(Ordering::Relaxed);
        let misses = self.stats.total_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            100.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Detect NUMA node (simplified implementation)
    fn detect_numa_node() -> Option<usize> {
        // In a real implementation, this would use libnuma or similar
        // For now, return None to indicate NUMA awareness is not available
        None
    }

    /// Warm up the buffer pool by pre-allocating buffers
    pub fn warm_up(&self, buffers_per_class: usize) -> Result<()> {
        for pool in &self.pools {
            for _ in 0..buffers_per_class {
                if let Ok(buffer) = AlignedBuffer::new(pool.size, CACHE_LINE_SIZE) {
                    if !pool.return_buffer(buffer) {
                        break; // Pool is full
                    }
                } else {
                    return Err(SystemError::ResourceUnavailable(
                        "Failed to warm up buffer pool".to_string()
                    ).into());
                }
            }
        }
        Ok(())
    }

    /// Shrink the pool by removing unused buffers
    pub fn shrink(&self) {
        for pool in &self.pools {
            pool.shrink();
        }
    }
}

impl SizeClassPool {
    fn new(size: usize, max_buffers: usize) -> Self {
        Self {
            size,
            buffers: Mutex::new(VecDeque::with_capacity(max_buffers)),
            max_buffers,
            allocated: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    fn get_buffer(&self) -> Option<AlignedBuffer> {
        let mut buffers = self.buffers.lock().unwrap();
        if let Some(buffer) = buffers.pop_front() {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(buffer)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    fn return_buffer(&self, buffer: AlignedBuffer) -> bool {
        let mut buffers = self.buffers.lock().unwrap();
        if buffers.len() < self.max_buffers {
            buffers.push_back(buffer);
            true
        } else {
            false // Pool is full
        }
    }

    fn shrink(&self) {
        let mut buffers = self.buffers.lock().unwrap();
        // Remove half of the buffers to free memory
        let target_size = buffers.len() / 2;
        buffers.truncate(target_size);
    }
}

impl PoolStats {
    fn new() -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            total_hits: AtomicUsize::new(0),
            total_misses: AtomicUsize::new(0),
            memory_usage: AtomicUsize::new(0),
        }
    }
}

impl Default for AdvancedBufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed buffer pool statistics
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    pub total_allocated: usize,
    pub total_hits: usize,
    pub total_misses: usize,
    pub memory_usage: usize,
    pub hit_rate: f64,
    pub numa_node: Option<usize>,
    pub size_class_stats: Vec<SizeClassStats>,
}

/// Statistics for a specific size class
#[derive(Debug, Clone)]
pub struct SizeClassStats {
    pub size: usize,
    pub allocated: usize,
    pub hits: usize,
    pub misses: usize,
    pub available: usize,
    pub max_buffers: usize,
}

/// Thread-safe shared buffer pool
pub type SharedAdvancedBufferPool = Arc<AdvancedBufferPool>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_buffer_creation() {
        let buffer = AlignedBuffer::new(1024, CACHE_LINE_SIZE).unwrap();
        assert_eq!(buffer.size(), 1024);
        assert!(buffer.is_aligned(CACHE_LINE_SIZE));
    }

    #[test]
    fn test_aligned_buffer_data_access() {
        let mut buffer = AlignedBuffer::new(100, CACHE_LINE_SIZE).unwrap();
        let slice = buffer.as_mut_slice();
        
        // Write some data
        slice[0] = 42;
        slice[99] = 24;
        
        // Read it back
        let read_slice = buffer.as_slice();
        assert_eq!(read_slice[0], 42);
        assert_eq!(read_slice[99], 24);
    }

    #[test]
    fn test_advanced_buffer_pool_creation() {
        let pool = AdvancedBufferPool::new();
        let stats = pool.get_stats();
        
        assert!(stats.size_class_stats.len() > 0);
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.hit_rate, 100.0); // No requests yet
    }

    #[test]
    fn test_buffer_allocation_and_return() {
        let pool = AdvancedBufferPool::new();
        
        // Get a buffer
        let buffer = pool.get_buffer(512);
        assert!(buffer.is_some());
        
        let buffer = buffer.unwrap();
        assert!(buffer.size() >= 512);
        
        // Return the buffer
        pool.return_buffer(buffer);
        
        // Get another buffer (should be reused)
        let buffer2 = pool.get_buffer(512);
        assert!(buffer2.is_some());
    }

    #[test]
    fn test_size_class_selection() {
        let pool = AdvancedBufferPool::new();
        
        // Request small buffer
        let small_buffer = pool.get_buffer(32).unwrap();
        assert_eq!(small_buffer.size(), 64); // Should get 64-byte buffer
        
        // Request medium buffer
        let medium_buffer = pool.get_buffer(200).unwrap();
        assert_eq!(medium_buffer.size(), 256); // Should get 256-byte buffer
        
        // Request large buffer
        let large_buffer = pool.get_buffer(1200).unwrap();
        assert_eq!(large_buffer.size(), 1500); // Should get 1500-byte buffer
    }

    #[test]
    fn test_pool_statistics() {
        let pool = AdvancedBufferPool::new();
        
        // Get some buffers
        let _buffer1 = pool.get_buffer(64);
        let _buffer2 = pool.get_buffer(128);
        let _buffer3 = pool.get_buffer(256);
        
        let stats = pool.get_stats();
        assert_eq!(stats.total_allocated, 3);
        assert_eq!(stats.total_misses, 3); // All misses since pool starts empty
    }

    #[test]
    fn test_pool_warm_up() {
        let pool = AdvancedBufferPool::new();
        
        // Warm up the pool
        assert!(pool.warm_up(5).is_ok());
        
        // Now getting buffers should be hits
        let _buffer = pool.get_buffer(64);
        let stats = pool.get_stats();
        assert!(stats.total_hits > 0);
    }

    #[test]
    fn test_custom_size_classes() {
        let custom_sizes = vec![32, 96, 160, 320];
        let pool = AdvancedBufferPool::with_size_classes(custom_sizes.clone(), 50);
        
        let stats = pool.get_stats();
        assert_eq!(stats.size_class_stats.len(), custom_sizes.len());
        
        for (i, size_stat) in stats.size_class_stats.iter().enumerate() {
            assert_eq!(size_stat.size, custom_sizes[i]);
            assert_eq!(size_stat.max_buffers, 50);
        }
    }

    #[test]
    fn test_pool_shrinking() {
        let pool = AdvancedBufferPool::new();
        
        // Warm up the pool
        pool.warm_up(10).unwrap();
        
        // Get initial stats
        let stats_before = pool.get_stats();
        let total_available_before: usize = stats_before.size_class_stats
            .iter()
            .map(|s| s.available)
            .sum();
        
        // Shrink the pool
        pool.shrink();
        
        // Check that buffers were removed
        let stats_after = pool.get_stats();
        let total_available_after: usize = stats_after.size_class_stats
            .iter()
            .map(|s| s.available)
            .sum();
        
        assert!(total_available_after < total_available_before);
    }

    #[test]
    fn test_memory_alignment() {
        let buffer = AlignedBuffer::new(1000, 256).unwrap();
        assert!(buffer.is_aligned(256));
        assert!(buffer.is_aligned(128));
        assert!(buffer.is_aligned(64));
    }
}