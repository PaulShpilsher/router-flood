//! Lock-free buffer pool implementation for high-performance packet building
//!
//! This implementation uses atomic operations and lock-free data structures
//! to provide better performance than the original mutex-based approach.

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;
use std::ptr;

/// Lock-free buffer pool using atomic operations
pub struct LockFreeBufferPool {
    buffers: Vec<AtomicPtr<Vec<u8>>>,
    buffer_size: usize,
    pool_size: usize,
    next_index: AtomicUsize,
}

impl LockFreeBufferPool {
    /// Create a new lock-free buffer pool
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
    
    /// Get a buffer from the pool (lock-free)
    #[inline]
    pub fn get_buffer(&self) -> Option<Vec<u8>> {
        for _ in 0..self.pool_size {
            let index = self.next_index.fetch_add(1, Ordering::Relaxed) % self.pool_size;
            let buffer_ptr = self.buffers[index].swap(ptr::null_mut(), Ordering::Acquire);
            
            if !buffer_ptr.is_null() {
                unsafe {
                    let mut buffer = *Box::from_raw(buffer_ptr);
                    buffer.clear();
                    buffer.resize(self.buffer_size, 0);
                    return Some(buffer);
                }
            }
        }
        
        // If no buffer available, allocate a new one
        Some(vec![0u8; self.buffer_size])
    }
    
    /// Return a buffer to the pool (lock-free)
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
    pub fn utilization(&self) -> f64 {
        let mut available = 0;
        for buffer in &self.buffers {
            if !buffer.load(Ordering::Relaxed).is_null() {
                available += 1;
            }
        }
        available as f64 / self.pool_size as f64
    }
}

impl Drop for LockFreeBufferPool {
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

unsafe impl Send for LockFreeBufferPool {}
unsafe impl Sync for LockFreeBufferPool {}

/// Shared buffer pool for use across multiple workers
pub struct SharedBufferPool {
    inner: Arc<LockFreeBufferPool>,
}

impl SharedBufferPool {
    /// Create a new shared buffer pool
    pub fn new(buffer_size: usize, pool_size: usize) -> Self {
        Self {
            inner: Arc::new(LockFreeBufferPool::new(buffer_size, pool_size)),
        }
    }
    
    /// Get a buffer from the shared pool
    #[inline(always)]
    pub fn get_buffer(&self) -> Option<Vec<u8>> {
        self.inner.get_buffer()
    }
    
    /// Return a buffer to the shared pool
    #[inline(always)]
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        self.inner.return_buffer(buffer);
    }
    
    /// Get pool utilization
    pub fn utilization(&self) -> f64 {
        self.inner.utilization()
    }
    
    /// Clone the shared pool (cheap - just clones the Arc)
    pub fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Barrier;
    
    #[test]
    fn test_lock_free_buffer_pool() {
        let pool = LockFreeBufferPool::new(1024, 10);
        
        // Test getting and returning buffers
        let buffer1 = pool.get_buffer().unwrap();
        assert_eq!(buffer1.len(), 1024);
        
        let buffer2 = pool.get_buffer().unwrap();
        assert_eq!(buffer2.len(), 1024);
        
        pool.return_buffer(buffer1);
        pool.return_buffer(buffer2);
        
        // Should be able to get buffers again
        let buffer3 = pool.get_buffer().unwrap();
        assert_eq!(buffer3.len(), 1024);
    }
    
    #[test]
    fn test_concurrent_access() {
        let pool = Arc::new(LockFreeBufferPool::new(1024, 100));
        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];
        
        for _ in 0..10 {
            let pool_clone = Arc::clone(&pool);
            let barrier_clone = Arc::clone(&barrier);
            
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                
                // Each thread gets and returns 100 buffers
                for _ in 0..100 {
                    if let Some(buffer) = pool_clone.get_buffer() {
                        assert_eq!(buffer.len(), 1024);
                        pool_clone.return_buffer(buffer);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
    
    #[test]
    fn test_shared_buffer_pool() {
        let pool = SharedBufferPool::new(512, 5);
        let pool_clone = pool.clone();
        
        let buffer = pool.get_buffer().unwrap();
        assert_eq!(buffer.len(), 512);
        
        pool_clone.return_buffer(buffer);
        
        let buffer2 = pool.get_buffer().unwrap();
        assert_eq!(buffer2.len(), 512);
    }
}