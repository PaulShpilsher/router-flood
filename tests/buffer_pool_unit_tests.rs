//! Unit tests for buffer pool functionality
//!
//! These tests were moved from src/buffer_pool.rs to follow best practices
//! of keeping all tests in the tests/ directory.

use router_flood::utils::buffer_pool::{BufferPool, WorkerBufferPool};

#[test]
fn test_buffer_pool_basic() {
    let pool = BufferPool::new(1400, 5, 10);
    assert_eq!(pool.pool_size(), 5);
    
    let buffer = pool.get_buffer();
    assert_eq!(buffer.len(), 1400);
    assert_eq!(pool.pool_size(), 4);
    
    pool.return_buffer(buffer);
    assert_eq!(pool.pool_size(), 5);
}

#[test]
fn test_worker_buffer_pool() {
    let mut pool = WorkerBufferPool::new(1024, 3, 5);
    assert_eq!(pool.pool_size(), 3);
    
    let buffer = pool.get_buffer();
    assert_eq!(buffer.len(), 1024);
    assert_eq!(pool.pool_size(), 2);
    
    pool.return_buffer(buffer);
    assert_eq!(pool.pool_size(), 3);
}

#[test]
fn test_pool_max_size_limit() {
    let mut pool = WorkerBufferPool::new(512, 2, 3);
    
    // Get all buffers
    let buf1 = pool.get_buffer();
    let buf2 = pool.get_buffer();
    let buf3 = pool.get_buffer(); // This creates a new one
    
    assert_eq!(pool.pool_size(), 0);
    
    // Return all buffers
    pool.return_buffer(buf1);
    pool.return_buffer(buf2);
    pool.return_buffer(buf3);
    
    // Should be limited to max_pool_size
    assert_eq!(pool.pool_size(), 3);
    
    // Try to return one more
    let buf4 = vec![0u8; 512];
    pool.return_buffer(buf4);
    
    // Should still be at max
    assert_eq!(pool.pool_size(), 3);
}
