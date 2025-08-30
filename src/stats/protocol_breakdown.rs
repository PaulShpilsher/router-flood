//! Efficient protocol breakdown tracking
//!
//! This module provides optimized data structures for tracking protocol statistics
//! using arrays instead of HashMap for better performance.

use crate::constants::protocols;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Protocol index for array-based storage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtocolIndex {
    Udp = 0,
    Tcp = 1,
    Icmp = 2,
    Ipv6 = 3,
    Arp = 4,
}

impl ProtocolIndex {
    /// Convert from protocol name to index
    pub fn from_protocol_name(protocol: &str) -> Option<Self> {
        match protocol {
            protocols::UDP => Some(Self::Udp),
            protocols::TCP => Some(Self::Tcp),
            protocols::ICMP => Some(Self::Icmp),
            protocols::IPV6 => Some(Self::Ipv6),
            protocols::ARP => Some(Self::Arp),
            _ => None,
        }
    }
    
    /// Convert to protocol name
    pub const fn to_protocol_name(self) -> &'static str {
        match self {
            Self::Udp => protocols::UDP,
            Self::Tcp => protocols::TCP,
            Self::Icmp => protocols::ICMP,
            Self::Ipv6 => protocols::IPV6,
            Self::Arp => protocols::ARP,
        }
    }
    
    /// Get all protocol indices
    pub const fn all() -> [Self; 5] {
        [Self::Udp, Self::Tcp, Self::Icmp, Self::Ipv6, Self::Arp]
    }
}

/// Efficient protocol breakdown using arrays instead of HashMap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolBreakdown {
    /// Protocol counts indexed by ProtocolIndex
    counts: [u64; 5],
}

impl Default for ProtocolBreakdown {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolBreakdown {
    /// Create a new empty protocol breakdown
    pub const fn new() -> Self {
        Self {
            counts: [0; 5],
        }
    }
    
    /// Increment count for a protocol by name
    pub fn increment(&mut self, protocol: &str) {
        if let Some(index) = ProtocolIndex::from_protocol_name(protocol) {
            self.counts[index as usize] += 1;
        }
    }
    
    /// Increment count for a protocol by index (faster)
    pub fn increment_by_index(&mut self, index: ProtocolIndex) {
        self.counts[index as usize] += 1;
    }
    
    /// Add a value to a protocol count
    pub fn add(&mut self, protocol: &str, value: u64) {
        if let Some(index) = ProtocolIndex::from_protocol_name(protocol) {
            self.counts[index as usize] += value;
        }
    }
    
    /// Add a value to a protocol count by index (faster)
    pub fn add_by_index(&mut self, index: ProtocolIndex, value: u64) {
        self.counts[index as usize] += value;
    }
    
    /// Get count for a protocol by name
    pub fn get(&self, protocol: &str) -> u64 {
        ProtocolIndex::from_protocol_name(protocol)
            .map(|index| self.counts[index as usize])
            .unwrap_or(0)
    }
    
    /// Get count for a protocol by index (faster)
    pub fn get_by_index(&self, index: ProtocolIndex) -> u64 {
        self.counts[index as usize]
    }
    
    /// Get all counts as array
    pub const fn as_array(&self) -> &[u64; 5] {
        &self.counts
    }
    
    /// Convert to HashMap for backward compatibility
    pub fn to_hashmap(&self) -> HashMap<String, u64> {
        let mut map = HashMap::with_capacity(5);
        for index in ProtocolIndex::all() {
            map.insert(
                index.to_protocol_name().to_string(),
                self.counts[index as usize],
            );
        }
        map
    }
    
    /// Create from HashMap for backward compatibility
    pub fn from_hashmap(map: &HashMap<String, u64>) -> Self {
        let mut breakdown = Self::new();
        for (protocol, &count) in map {
            if let Some(index) = ProtocolIndex::from_protocol_name(protocol) {
                breakdown.counts[index as usize] = count;
            }
        }
        breakdown
    }
    
    /// Get total count across all protocols
    pub fn total(&self) -> u64 {
        self.counts.iter().sum()
    }
    
    /// Reset all counts to zero
    pub fn reset(&mut self) {
        self.counts = [0; 5];
    }
    
    /// Merge another breakdown into this one
    pub fn merge(&mut self, other: &Self) {
        for i in 0..5 {
            self.counts[i] += other.counts[i];
        }
    }
    
    /// Create an iterator over protocol names and counts
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        ProtocolIndex::all()
            .into_iter()
            .map(move |index| (index.to_protocol_name(), self.counts[index as usize]))
    }
    
    /// Create an iterator over non-zero protocol counts
    pub fn iter_non_zero(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        self.iter().filter(|(_, count)| *count > 0)
    }
}

impl std::fmt::Display for ProtocolBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (protocol, count) in self.iter_non_zero() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", protocol, count)?;
            first = false;
        }
        Ok(())
    }
}

// Tests moved to tests/ directory
