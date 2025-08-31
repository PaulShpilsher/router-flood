//! High-performance lock-free buffer pool for packet allocation
//!
//! This module implements a lock-free, thread-safe buffer pool to minimize
//! allocation overhead during high-frequency packet generation.

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;

/// High-performance lock-free buffer pool for packet data
/// 
/// This buffer pool uses atomic operations to provide thread-safe access
/// without mutex contention, making it suitable for high-throughput scenarios.
pub struct BufferPool {
    buffers: Vec<AtomicPtr<Vec<u8>>>,
    buffer_size: usize,
    pool_size: usize,
    next_index: AtomicUsize,
}

impl BufferPool {
    /// Create a new lock-free buffer pool
    /// 
    /// # Arguments
    /// * `buffer_size` - Size of each buffer in bytes
    /// * `pool_size` - Maximum number of buffers to keep in the pool
    pub fn new(buffer_size: usize, pool_size: usize) -> Self {
        let mut buffers = Vec::with_capacity(pool_size);
        
        // Pre-allocate all buffers
        for _ in 0..pool_size {
            let buffer = Box::new(vec![0u8; buffer_size]);
            buffers.push(AtomicPtr::new(Box::into_raw(buffer)));
        }
        
        Self {
            buffers,
            buffer_size,
            pool_size,
            next_index: AtomicUsize::new(0),
        }
    }
    
    /// Get a buffer from the pool (always succeeds)
    /// 
    /// Returns a buffer from the pool if available, otherwise allocates a new one.
    /// This operation is lock-free and thread-safe.
    #[inline]
    pub fn buffer(&self) -> Vec<u8> {
        // Try to get a buffer from the pool
        for _ in 0..self.pool_size {
            let index = self.next_index.fetch_add(1, Ordering::Relaxed) % self.pool_size;
            let buffer_ptr = self.buffers[index].swap(ptr::null_mut(), Ordering::Acquire);
            
            if !buffer_ptr.is_null() {
                unsafe {
                    let mut buffer = *Box::from_raw(buffer_ptr);
                    buffer.clear();
                    buffer.resize(self.buffer_size, 0);
                    return buffer;
                }
            }
        }
        
        // If no buffer available, allocate a new one
        vec![0u8; self.buffer_size]
    }
    
    /// Return a buffer to the pool
    /// 
    /// Returns the buffer to the pool if there's space and the buffer is large enough.
    /// This operation is lock-free and thread-safe.
    #[inline]
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        if buffer.capacity() >= self.buffer_size {
            let boxed_buffer = Box::new(buffer);
            let buffer_ptr = Box::into_raw(boxed_buffer);
            
            // Try to find an empty slot
            for i in 0..self.pool_size {
                let expected = ptr::null_mut();
                if self.buffers[i].compare_exchange_weak(
                    expected,
                    buffer_ptr,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    return;
                }
            }
            
            // If pool is full, just drop the buffer
            unsafe {
                let _ = Box::from_raw(buffer_ptr);
            }
        }
        // If buffer is too small, just drop it
    }
    
    /// Get pool utilization statistics
    /// 
    /// Returns a value between 0.0 and 1.0 indicating how full the pool is.
    pub fn utilization(&self) -> f64 {
        let mut available = 0;
        for buffer in &self.buffers {
            if !buffer.load(Ordering::Relaxed).is_null() {
                available += 1;
            }
        }
        available as f64 / self.pool_size as f64
    }
    
    /// Get the buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
    
    /// Get the maximum pool size
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        // Clean up any remaining buffers
        for buffer in &self.buffers {
            let buffer_ptr = buffer.load(Ordering::Relaxed);
            if !buffer_ptr.is_null() {
                unsafe {
                    let _ = Box::from_raw(buffer_ptr);
                }
            }
        }
    }
}

unsafe impl Send for BufferPool {}
unsafe impl Sync for BufferPool {}
