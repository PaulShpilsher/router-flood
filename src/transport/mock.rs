//! Mock transport implementation for testing

use super::layer::{TransportLayer, ChannelType};
use crate::error::Result;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Mock transport for testing and dry-run mode
pub struct MockTransport {
    packets_sent: Arc<AtomicU64>,
    should_fail: bool,
}

impl MockTransport {
    /// Create a new mock transport
    pub fn new() -> Self {
        Self {
            packets_sent: Arc::new(AtomicU64::new(0)),
            should_fail: false,
        }
    }
    
    /// Create a mock transport that simulates failures
    pub fn with_failures() -> Self {
        Self {
            packets_sent: Arc::new(AtomicU64::new(0)),
            should_fail: true,
        }
    }
    
    /// Get the number of packets "sent"
    pub fn packets_sent(&self) -> u64 {
        self.packets_sent.load(Ordering::Relaxed)
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportLayer for MockTransport {
    fn send_packet(&self, _data: &[u8], _target: IpAddr, _channel_type: ChannelType) -> Result<()> {
        if self.should_fail && self.packets_sent() % 100 == 99 {
            // Simulate 1% failure rate
            return Err(crate::error::RouterFloodError::Network(
                "Mock transport simulated failure".to_string()
            ).into());
        }
        
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn name(&self) -> &'static str {
        "Mock"
    }
}