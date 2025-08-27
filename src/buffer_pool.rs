//! Buffer pool for optimized packet allocation
//!
//! This module implements a thread-safe buffer pool to reduce allocation
//! overhead during high-frequency packet generation.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Thread-safe buffer pool for packet data
pub struct BufferPool {
    buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl BufferPool {
    /// Create a new buffer pool with the specified configuration
    pub fn new(buffer_size: usize, initial_count: usize, max_pool_size: usize) -> Self {
        let mut buffers = VecDeque::with_capacity(initial_count);
        
        // Pre-allocate initial buffers
        for _ in 0..initial_count {
            buffers.push_back(vec![0u8; buffer_size]);
        }
        
        Self {
            buffers: Arc::new(Mutex::new(buffers)),
            buffer_size,
            max_pool_size,
        }
    }
    
    /// Get a buffer from the pool, or allocate a new one if pool is empty
    pub fn get_buffer(&self) -> Vec<u8> {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.pop_front().unwrap_or_else(|| vec![0u8; self.buffer_size])
    }
    
    /// Return a buffer to the pool (if there's space)
    pub fn return_buffer(&self, mut buffer: Vec<u8>) {
        // Resize buffer to standard size if needed
        buffer.clear();
        if buffer.capacity() >= self.buffer_size {
            buffer.resize(self.buffer_size, 0);
            
            let mut buffers = self.buffers.lock().unwrap();
            if buffers.len() < self.max_pool_size {
                buffers.push_back(buffer);
            }
            // Otherwise, let it drop to avoid unbounded growth
        }
    }
    
    /// Get current pool size (for monitoring)
    pub fn pool_size(&self) -> usize {
        self.buffers.lock().unwrap().len()
    }
}

impl Clone for BufferPool {
    fn clone(&self) -> Self {
        Self {
            buffers: Arc::clone(&self.buffers),
            buffer_size: self.buffer_size,
            max_pool_size: self.max_pool_size,
        }
    }
}

/// Per-worker buffer pool (no mutex contention)
pub struct WorkerBufferPool {
    buffers: VecDeque<Vec<u8>>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl WorkerBufferPool {
    /// Create a new per-worker buffer pool
    pub fn new(buffer_size: usize, initial_count: usize, max_pool_size: usize) -> Self {
        let mut buffers = VecDeque::with_capacity(initial_count);
        
        for _ in 0..initial_count {
            buffers.push_back(vec![0u8; buffer_size]);
        }
        
        Self {
            buffers,
            buffer_size,
            max_pool_size,
        }
    }
    
    /// Get a buffer from the pool
    pub fn get_buffer(&mut self) -> Vec<u8> {
        self.buffers.pop_front().unwrap_or_else(|| vec![0u8; self.buffer_size])
    }
    
    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        if self.buffers.len() < self.max_pool_size {
            buffer.clear();
            buffer.resize(self.buffer_size, 0);
            self.buffers.push_back(buffer);
        }
    }
    
    /// Get current pool size
    pub fn pool_size(&self) -> usize {
        self.buffers.len()
    }
}
