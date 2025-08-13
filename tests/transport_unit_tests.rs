//! Unit tests for transport functionality
//!
//! These tests were moved from src/transport.rs to follow best practices
//! of keeping all tests in the tests/ directory.

use router_flood::transport::{WorkerChannels, ChannelFactory};

#[test]
fn test_dry_run_channels() {
    let channels = WorkerChannels::new(None, true).unwrap();
    assert!(!channels.has_channels());
}

#[test]
fn test_channel_factory_capacity() {
    let channels = ChannelFactory::create_worker_channels(4, None, true).unwrap();
    assert_eq!(channels.len(), 4);
}
