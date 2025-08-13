//! Unit tests for RNG functionality
//!
//! These tests were moved from src/rng.rs to follow best practices
//! of keeping all tests in the tests/ directory.

use router_flood::rng::{BatchedRng, RandomValueType, DEFAULT_BATCH_SIZE};

#[test]
fn test_batched_rng_creation() {
    let rng = BatchedRng::new();
    assert_eq!(rng.batch_size(), DEFAULT_BATCH_SIZE);
    assert!(rng.batch_remaining(RandomValueType::Port) > 0);
}

#[test]
fn test_custom_batch_size() {
    let batch_size = 500;
    let rng = BatchedRng::with_batch_size(batch_size);
    assert_eq!(rng.batch_size(), batch_size);
}

#[test]
fn test_port_generation() {
    let mut rng = BatchedRng::with_batch_size(10);
    for _ in 0..15 {
        let port = rng.port();
        assert!(port >= 1024 && port < 65535);
    }
}

#[test]
fn test_ttl_generation() {
    let mut rng = BatchedRng::with_batch_size(10);
    for _ in 0..15 {
        let ttl = rng.ttl();
        assert!(ttl >= 32 && ttl < 128);
    }
}

#[test]
fn test_payload_generation() {
    let mut rng = BatchedRng::with_batch_size(100);
    let payload = rng.payload(256);
    assert_eq!(payload.len(), 256);
}

#[test]
fn test_batch_replenishment() {
    let mut rng = BatchedRng::with_batch_size(5);
    
    // Consume all ports
    for _ in 0..5 {
        let _ = rng.port();
    }
    assert_eq!(rng.batch_remaining(RandomValueType::Port), 0);
    
    // Getting one more should trigger replenishment
    let _ = rng.port();
    assert!(rng.batch_remaining(RandomValueType::Port) > 0);
}

#[test]
fn test_multiple_values() {
    let mut rng = BatchedRng::new();
    let ports = rng.ports(5);
    assert_eq!(ports.len(), 5);
    
    let ttls = rng.ttls(3);
    assert_eq!(ttls.len(), 3);
}
