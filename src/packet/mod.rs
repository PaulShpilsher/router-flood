//! Packet building and protocol handling
//!
//! This module provides a trait-based architecture for packet construction
//! with support for multiple protocols and zero-copy operations.

pub mod builder;
pub mod strategies;
pub mod types;

pub use builder::PacketBuilder;
pub use types::PacketType;

use crate::error::Result;
use std::net::IpAddr;

/// Core trait for packet building strategies
pub trait PacketStrategy: Send + Sync {
    /// Build a packet directly into the provided buffer
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize>;
    
    /// Get the protocol name for statistics
    fn protocol_name(&self) -> &'static str;
    
    /// Get the maximum possible packet size for this strategy
    fn max_packet_size(&self) -> usize;
    
    /// Check if this strategy is compatible with the target IP version
    fn is_compatible_with(&self, target_ip: IpAddr) -> bool;
}

/// Target information for packet building
#[derive(Debug, Clone)]
pub struct Target {
    pub ip: IpAddr,
    pub port: u16,
}

impl Target {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }
}