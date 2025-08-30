//! Core traits for dependency injection and module decoupling
//!
//! This module provides streamlined traits without async trait objects
//! for better performance and simpler code.

use std::net::IpAddr;
use crate::error::Result;
use crate::packet::PacketType;

/// Statistics collector trait
pub trait StatsCollector: Send + Sync {
    /// Record a successfully sent packet
    fn record_packet_sent(&self, protocol: &str, size: usize);
    
    /// Record a failed packet attempt
    fn record_packet_failed(&self);
    
    /// Get current packet count for monitoring
    fn get_packet_count(&self) -> u64;
    
    /// Get current failure count for monitoring
    fn get_failure_count(&self) -> u64;
}

/// Packet builder trait
pub trait PacketBuilder: Send + Sync {
    /// Build a packet for the given parameters
    fn build_packet(
        &mut self,
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
    ) -> Result<(Vec<u8>, &'static str)>;
    
    /// Get the next packet type based on protocol mix
    fn next_packet_type(&mut self) -> PacketType;
    
    /// Get packet type appropriate for the target IP
    fn next_packet_type_for_ip(&mut self, target_ip: IpAddr) -> PacketType;
}

/// Target port provider trait
pub trait TargetProvider: Send + Sync {
    /// Get the next target port in rotation
    fn next_port(&self) -> u16;
    
    /// Get all configured ports
    fn get_ports(&self) -> &[u16];
}

/// Worker configuration trait
pub trait WorkerConfig: Send + Sync {
    /// Get the number of worker threads
    fn thread_count(&self) -> usize;
    
    /// Get the target packet rate
    fn packet_rate(&self) -> u64;
    
    /// Get packet size range
    fn packet_size_range(&self) -> (usize, usize);
    
    /// Check if timing should be randomized
    fn randomize_timing(&self) -> bool;
    
    /// Check if perfect simulation is enabled
    fn perfect_simulation(&self) -> bool;
    
    /// Check if dry run mode is enabled
    fn dry_run(&self) -> bool;
}