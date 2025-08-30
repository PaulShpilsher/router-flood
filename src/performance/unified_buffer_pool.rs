//! Unified buffer pool implementation consolidating all buffer pool variants
//!
//! This module provides a single, optimized buffer pool implementation that
//! replaces multiple scattered implementations, following YAGNI and KISS principles.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::ptr;

/// Statistics for buffer pool monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub hits: u64,
    pub misses: u64,
    pub current_size: usize,
    pub max_size: usize,
}

impl PoolStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            100.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Unified buffer pool supporting both shared and per-worker usage patterns
pub enum UnifiedBufferPool {
    /// Lock-free implementation for high-contention scenarios
    LockFree(LockFreePool),
    /// Per-worker implementation for zero-contention scenarios
    PerWorker(PerWorkerPool),
    /// Shared implementation with mutex for moderate contention
    Shared(SharedPool),
}

impl UnifiedBufferPool {
    /// Create a lock-free buffer pool for high-contention scenarios
    pub fn lock_free(buffer_size: usize, pool_size: usize) -> Self {
        Self::LockFree(LockFreePool::new(buffer_size, pool_size))
    }
    
    /// Create a per-worker buffer pool for zero-contention scenarios
    pub fn per_worker(buffer_size: usize, initial_count: usize, max_size: usize) -> Self {
        Self::PerWorker(PerWorkerPool::new(buffer_size, initial_count, max_size))
    }
    
    /// Create a shared buffer pool for moderate contention scenarios
    pub fn shared(buffer_size: usize, initial_count: usize, max_size: usize) -> Self {
        Self::Shared(SharedPool::new(buffer_size, initial_count, max_size))
    }
    
    /// Get a buffer from the pool
    pub fn get_buffer(&mut self) -> Vec<u8> {
        match self {
            Self::LockFree(pool) => pool.get_buffer(),
            Self::PerWorker(pool) => pool.get_buffer(),
            Self::Shared(pool) => pool.get_buffer(),
        }
    }
    
    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, buffer: Vec<u8>) {
        match self {
            Self::LockFree(pool) => pool.return_buffer(buffer),
            Self::PerWorker(pool) => pool.return_buffer(buffer),
            Self::Shared(pool) => pool.return_buffer(buffer),
        }
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        match self {
            Self::LockFree(pool) => pool.stats(),
            Self::PerWorker(pool) => pool.stats(),
            Self::Shared(pool) => pool.stats(),
        }
    }
}

/// Lock-free buffer pool implementation
pub struct LockFreePool {
    buffers: Vec<AtomicPtr<Vec<u8>>>,
    buffer_size: usize,
    pool_size: usize,
    next_index: AtomicUsize,
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl LockFreePool {
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
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }
    
    pub fn get_buffer(&self) -> Vec<u8> {
        // Try to get a buffer from the pool
        for _ in 0..self.pool_size {
            let index = self.next_index.fetch_add(1, Ordering::Relaxed) % self.pool_size;
            let buffer_ptr = self.buffers[index].swap(ptr::null_mut(), Ordering::Acquire);
            
            if !buffer_ptr.is_null() {
                self.hits.fetch_add(1, Ordering::Relaxed);
                unsafe {
                    let mut buffer = *Box::from_raw(buffer_ptr);
                    buffer.clear();
                    buffer.resize(self.buffer_size, 0);
                    return buffer;
                }
            }
        }
        
        // Pool is empty, allocate new buffer
        self.misses.fetch_add(1, Ordering::Relaxed);
        vec![0u8; self.buffer_size]
    }
    
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
            
            // Pool is full, drop the buffer
            unsafe {
                let _ = Box::from_raw(buffer_ptr);
            }
        }
    }
    
    pub fn stats(&self) -> PoolStats {
        let hits = self.hits.load(Ordering::Relaxed) as u64;
        let misses = self.misses.load(Ordering::Relaxed) as u64;
        
        // Count available buffers
        let mut available = 0;
        for buffer in &self.buffers {
            if !buffer.load(Ordering::Relaxed).is_null() {
                available += 1;
            }
        }
        
        PoolStats {
            hits,
            misses,
            current_size: available,
            max_size: self.pool_size,
        }
    }
}

impl Drop for LockFreePool {
    fn drop(&mut self) {
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

unsafe impl Send for LockFreePool {}
unsafe impl Sync for LockFreePool {}

/// Per-worker buffer pool (no synchronization needed)
pub struct PerWorkerPool {
    buffers: VecDeque<Vec<u8>>,
    buffer_size: usize,
    max_size: usize,
    hits: u64,
    misses: u64,
}

impl PerWorkerPool {
    pub fn new(buffer_size: usize, initial_count: usize, max_size: usize) -> Self {
        let mut buffers = VecDeque::with_capacity(initial_count);
        
        for _ in 0..initial_count {
            buffers.push_back(vec![0u8; buffer_size]);
        }
        
        Self {
            buffers,
            buffer_size,
            max_size,
            hits: 0,
            misses: 0,
        }
    }
    
    pub fn get_buffer(&mut self) -> Vec<u8> {
        if let Some(mut buffer) = self.buffers.pop_front() {
            self.hits += 1;
            buffer.clear();
            buffer.resize(self.buffer_size, 0);
            buffer
        } else {
            self.misses += 1;
            vec![0u8; self.buffer_size]
        }
    }
    
    pub fn return_buffer(&mut self, buffer: Vec<u8>) {
        if self.buffers.len() < self.max_size && buffer.capacity() >= self.buffer_size {
            self.buffers.push_back(buffer);
        }
    }
    
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            hits: self.hits,
            misses: self.misses,
            current_size: self.buffers.len(),
            max_size: self.max_size,
        }
    }
}

/// Shared buffer pool with mutex protection
pub struct SharedPool {
    inner: Arc<Mutex<PerWorkerPool>>,
}

impl SharedPool {
    pub fn new(buffer_size: usize, initial_count: usize, max_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PerWorkerPool::new(buffer_size, initial_count, max_size))),
        }
    }
    
    pub fn get_buffer(&self) -> Vec<u8> {
        self.inner.lock().unwrap().get_buffer()
    }
    
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        self.inner.lock().unwrap().return_buffer(buffer);
    }
    
    pub fn stats(&self) -> PoolStats {
        self.inner.lock().unwrap().stats()
    }
}

impl Clone for SharedPool {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Factory for creating appropriate buffer pool types based on usage pattern
pub struct BufferPoolFactory;

impl BufferPoolFactory {
    /// Create the most appropriate buffer pool for the given scenario
    pub fn create_optimal(
        buffer_size: usize,
        worker_count: usize,
        expected_contention: ContentionLevel,
    ) -> UnifiedBufferPool {
        match expected_contention {
            ContentionLevel::None => {
                // Per-worker pools for zero contention
                UnifiedBufferPool::per_worker(buffer_size, 5, 20)
            }
            ContentionLevel::Low => {
                // Shared pool with mutex for low contention
                UnifiedBufferPool::shared(buffer_size, worker_count * 2, worker_count * 10)
            }
            ContentionLevel::High => {
                // Lock-free pool for high contention
                UnifiedBufferPool::lock_free(buffer_size, worker_count * 5)
            }
        }
    }
}

/// Expected contention level for buffer pool selection
#[derive(Debug, Clone, Copy)]
pub enum ContentionLevel {
    None,   // Single worker or per-worker pools
    Low,    // Few workers, infrequent access
    High,   // Many workers, frequent access
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_per_worker_pool() {
        let mut pool = UnifiedBufferPool::per_worker(1024, 2, 5);
        
        // Get a buffer
        let buffer1 = pool.get_buffer();
        assert_eq!(buffer1.len(), 1024);
        
        // Return it
        pool.return_buffer(buffer1);
        
        // Get stats
        let stats = pool.stats();
        assert_eq!(stats.current_size, 2); // Should have 2 buffers (1 returned + 1 initial)
    }
    
    #[test]
    fn test_lock_free_pool() {
        let mut pool = UnifiedBufferPool::lock_free(512, 3);
        
        // Get multiple buffers
        let buffer1 = pool.get_buffer();
        let buffer2 = pool.get_buffer();
        
        assert_eq!(buffer1.len(), 512);
        assert_eq!(buffer2.len(), 512);
        
        // Return them
        pool.return_buffer(buffer1);
        pool.return_buffer(buffer2);
        
        let stats = pool.stats();
        assert!(stats.hits > 0);
    }
    
    #[test]
    fn test_shared_pool() {
        let mut pool = UnifiedBufferPool::shared(256, 1, 3);
        
        let buffer = pool.get_buffer();
        assert_eq!(buffer.len(), 256);
        
        pool.return_buffer(buffer);
        
        let stats = pool.stats();
        assert_eq!(stats.current_size, 1);
    }
    
    #[test]
    fn test_factory() {
        let pool = BufferPoolFactory::create_optimal(1024, 4, ContentionLevel::Low);
        
        match pool {
            UnifiedBufferPool::Shared(_) => {
                // Expected for low contention
            }
            _ => panic!("Expected shared pool for low contention"),
        }
    }
    
    #[test]
    fn test_pool_stats() {
        let mut pool = UnifiedBufferPool::per_worker(128, 1, 2);
        
        // Initial stats
        let stats = pool.stats();
        assert_eq!(stats.current_size, 1);
        assert_eq!(stats.max_size, 2);
        
        // Get buffer (should be a hit)
        let buffer = pool.get_buffer();
        let stats = pool.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        
        // Get another buffer (should be a miss since pool is empty)
        let buffer2 = pool.get_buffer();
        let stats = pool.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        
        // Return buffers
        pool.return_buffer(buffer);
        pool.return_buffer(buffer2);
        
        let stats = pool.stats();
        assert_eq!(stats.current_size, 2);
    }
}