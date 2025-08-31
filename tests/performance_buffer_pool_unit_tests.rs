//! Unit tests for buffer pool module
//!
//! These tests verify the high-performance lock-free buffer pool implementation.

use router_flood::utils::buffer_pool::BufferPool;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_buffer_pool_basic() {
    let pool = BufferPool::new(1024, 10);
    
    // Test getting and returning buffers
    let buffer1 = pool.buffer();
    assert_eq!(buffer1.len(), 1024);
    
    let buffer2 = pool.buffer();
    assert_eq!(buffer2.len(), 1024);
    
    pool.return_buffer(buffer1);
    pool.return_buffer(buffer2);
    
    // Should be able to get buffers again
    let buffer3 = pool.buffer();
    assert_eq!(buffer3.len(), 1024);
}

#[test]
fn test_concurrent_access() {
    let pool = Arc::new(BufferPool::new(1024, 100));
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let pool_clone = Arc::clone(&pool);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            // Each thread gets and returns 100 buffers
            for _ in 0..100 {
                let buffer = pool_clone.buffer();
                assert_eq!(buffer.len(), 1024);
                pool_clone.return_buffer(buffer);
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_buffer_pool_utilization() {
    let pool = BufferPool::new(512, 5);
    
    // Initially should have high utilization (all buffers available)
    let initial_utilization = pool.utilization();
    assert!(initial_utilization > 0.8); // Should be close to 1.0
    
    // Get some buffers
    let _buffer1 = pool.buffer();
    let _buffer2 = pool.buffer();
    
    // Utilization should decrease
    let reduced_utilization = pool.utilization();
    assert!(reduced_utilization < initial_utilization);
}

#[test]
fn test_buffer_pool_properties() {
    let pool = BufferPool::new(2048, 50);
    
    assert_eq!(pool.buffer_size(), 2048);
    assert_eq!(pool.pool_size(), 50);
}

#[test]
fn test_buffer_pool_always_succeeds() {
    let pool = BufferPool::new(1024, 1); // Very small pool
    
    // Even with a small pool, should always get buffers
    let mut buffers = Vec::new();
    for _ in 0..10 {
        buffers.push(pool.buffer());
    }
    
    // All buffers should be valid
    for buffer in &buffers {
        assert_eq!(buffer.len(), 1024);
    }
    
    // Return them
    for buffer in buffers {
        pool.return_buffer(buffer);
    }
}