//! Transport layer trait definitions

use crate::error::Result;
use std::net::IpAddr;

/// Core trait for transport layer implementations
pub trait TransportLayer: Send + Sync {
    /// Send a packet to the specified target
    fn send_packet(&self, data: &[u8], target: IpAddr, channel_type: ChannelType) -> Result<()>;
    
    /// Check if the transport layer is available
    fn is_available(&self) -> bool;
    
    /// Get transport layer name for logging
    fn name(&self) -> &'static str;
}

/// Channel type for different protocol layers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    IPv4,
    IPv6,
    Layer2,
}