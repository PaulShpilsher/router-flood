//! Local statistics accumulator for batched updates
//!
//! This module provides a local accumulator that batches updates to reduce
//! contention on the global statistics counters.

use super::FloodStats;
use std::sync::Arc;

/// Local stats accumulator to batch atomic updates
pub struct LocalStats {
    packets_sent: u64,
    packets_failed: u64,
    bytes_sent: u64,
    protocol_counts: ProtocolCounts,
    batch_size: usize,
    update_count: usize,
    stats_ref: Arc<FloodStats>,
}

/// Protocol-specific counters
struct ProtocolCounts {
    udp: u64,
    tcp: u64,
    icmp: u64,
    ipv6: u64,
    arp: u64,
    other: u64,
}

impl ProtocolCounts {
    fn new() -> Self {
        Self {
            udp: 0,
            tcp: 0,
            icmp: 0,
            ipv6: 0,
            arp: 0,
            other: 0,
        }
    }
    
    fn increment(&mut self, protocol: &str) {
        match protocol {
            "UDP" => self.udp += 1,
            "TCP" | "TCP-SYN" | "TCP-ACK" => self.tcp += 1,
            "ICMP" | "IPv6-ICMP" => self.icmp += 1,
            "IPv6" | "IPv6-UDP" | "IPv6-TCP" => self.ipv6 += 1,
            "ARP" => self.arp += 1,
            _ => self.other += 1,
        }
    }
    
    fn reset(&mut self) {
        self.udp = 0;
        self.tcp = 0;
        self.icmp = 0;
        self.ipv6 = 0;
        self.arp = 0;
        self.other = 0;
    }
}

impl LocalStats {
    /// Create a new local stats accumulator
    pub fn new(stats_ref: Arc<FloodStats>, batch_size: usize) -> Self {
        Self {
            packets_sent: 0,
            packets_failed: 0,
            bytes_sent: 0,
            protocol_counts: ProtocolCounts::new(),
            batch_size,
            update_count: 0,
            stats_ref,
        }
    }
    
    /// Increment sent packet count locally
    pub fn increment_sent(&mut self, bytes: u64, protocol: &str) {
        self.packets_sent += 1;
        self.bytes_sent += bytes;
        self.protocol_counts.increment(protocol);
        self.update_count += 1;
        
        // Flush to global stats if batch is full
        if self.update_count >= self.batch_size {
            self.flush();
        }
    }
    
    /// Increment failed packet count locally
    pub fn increment_failed(&mut self) {
        self.packets_failed += 1;
        self.update_count += 1;
        
        if self.update_count >= self.batch_size {
            self.flush();
        }
    }
    
    /// Flush accumulated stats to global counters
    pub fn flush(&mut self) {
        if self.update_count == 0 {
            return;
        }
        
        // Flush all accumulated stats at once using the new API
        // We send each protocol's stats separately
        if self.protocol_counts.udp > 0 {
            for _ in 0..self.protocol_counts.udp {
                self.stats_ref.increment_sent(self.bytes_sent / self.packets_sent.max(1), "UDP");
            }
        }
        if self.protocol_counts.tcp > 0 {
            for _ in 0..self.protocol_counts.tcp {
                self.stats_ref.increment_sent(self.bytes_sent / self.packets_sent.max(1), "TCP");
            }
        }
        if self.protocol_counts.icmp > 0 {
            for _ in 0..self.protocol_counts.icmp {
                self.stats_ref.increment_sent(self.bytes_sent / self.packets_sent.max(1), "ICMP");
            }
        }
        if self.protocol_counts.ipv6 > 0 {
            for _ in 0..self.protocol_counts.ipv6 {
                self.stats_ref.increment_sent(self.bytes_sent / self.packets_sent.max(1), "IPv6");
            }
        }
        if self.protocol_counts.arp > 0 {
            for _ in 0..self.protocol_counts.arp {
                self.stats_ref.increment_sent(self.bytes_sent / self.packets_sent.max(1), "ARP");
            }
        }
        
        // Flush failed packets
        for _ in 0..self.packets_failed {
            self.stats_ref.increment_failed();
        }
        
        // Reset local counters
        self.packets_sent = 0;
        self.packets_failed = 0;
        self.bytes_sent = 0;
        self.protocol_counts.reset();
        self.update_count = 0;
    }
}

impl Drop for LocalStats {
    fn drop(&mut self) {
        // Ensure any remaining stats are flushed when worker terminates
        self.flush();
    }
}