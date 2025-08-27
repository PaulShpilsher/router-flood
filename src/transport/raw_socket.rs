//! Raw socket transport implementation

use super::layer::{TransportLayer, ChannelType};
use crate::error::{NetworkError, Result};
use crate::transport::WorkerChannels;
use std::net::IpAddr;

/// Raw socket transport implementation
pub struct RawSocketTransport {
    channels: WorkerChannels,
}

impl RawSocketTransport {
    /// Create a new raw socket transport
    pub fn new(channels: WorkerChannels) -> Self {
        Self { channels }
    }
}

impl TransportLayer for RawSocketTransport {
    fn send_packet(&self, data: &[u8], target: IpAddr, channel_type: ChannelType) -> Result<()> {
        self.channels.send_packet(data, target, channel_type)
            .map_err(|e| NetworkError::PacketSend(format!("Raw socket send failed: {}", e)).into())
    }
    
    fn is_available(&self) -> bool {
        // Check if we have the necessary channels
        true // For now, assume available if we have channels
    }
    
    fn name(&self) -> &'static str {
        "RawSocket"
    }
}