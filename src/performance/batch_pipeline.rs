//! Batch packet processing pipeline
//!
//! This module provides a high-performance batch packet processing pipeline
//! that combines zero-copy operations, memory pooling, and lock-free statistics.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use crate::error::{Result, PacketError};
use crate::packet::PacketType;
use crate::performance::{
    ZeroCopyPacketBuilder, Memory, ManagedMemory
};
use crate::stats::internal_lockfree::{LockFreeStatsCollector, BatchedStatsCollector};

/// High-performance batch packet processor
pub struct BatchPacketProcessor {
    memory_manager: Arc<Memory>,
    stats_collector: Arc<LockFreeStatsCollector>,
    packet_builder: ZeroCopyPacketBuilder,
}

impl BatchPacketProcessor {
    /// Create a new batch packet processor
    pub fn new() -> Self {
        Self {
            memory_manager: Arc::new(Memory::new()),
            stats_collector: Arc::new(LockFreeStatsCollector::new()),
            packet_builder: ZeroCopyPacketBuilder::new(1500),
        }
    }
    
    /// Create a batched stats collector for a worker
    pub fn create_batched_collector(&self, batch_size: usize) -> BatchedStatsCollector {
        BatchedStatsCollector::new(self.stats_collector.clone(), batch_size)
    }
    
    /// Process a packet using zero-copy operations
    pub fn process_packet(
        &mut self,
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
        payload_size: usize,
    ) -> Result<ProcessedPacket<'_>> {
        let start_time = Instant::now();
        
        // Reset the packet builder for reuse
        self.packet_builder.reset();
        
        // Build the packet headers using zero-copy operations
        let protocol_name = self.build_packet_headers(packet_type, target_ip, target_port)?;
        
        // Allocate memory for the complete packet
        let total_size = self.packet_builder.len() + payload_size;
        let mut memory = self.memory_manager.allocate(total_size)
            .unwrap_or_else(|| ManagedMemory::heap(total_size));
        
        // Copy headers to the allocated memory
        let packet_data = self.packet_builder.packet();
        memory.as_mut_slice()[..packet_data.len()].copy_from_slice(packet_data);
        
        // Fill payload (could be optimized with SIMD)
        if payload_size > 0 {
            self.fill_payload(&mut memory.as_mut_slice()[packet_data.len()..], payload_size);
        }
        
        let processing_time = start_time.elapsed();
        let is_pooled = memory.is_pooled();
        
        Ok(ProcessedPacket {
            data: memory,
            protocol_name,
            processing_time,
            is_pooled,
        })
    }
    
    /// Build packet headers using zero-copy operations
    fn build_packet_headers(
        &mut self,
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
    ) -> Result<&'static str> {
        match packet_type {
            PacketType::Udp => {
                self.build_udp_packet(target_ip, target_port)?;
                Ok("UDP")
            }
            PacketType::TcpSyn => {
                self.build_tcp_packet(target_ip, target_port, 0x02)?; // SYN flag
                Ok("TCP_SYN")
            }
            PacketType::TcpAck => {
                self.build_tcp_packet(target_ip, target_port, 0x10)?; // ACK flag
                Ok("TCP_ACK")
            }
            PacketType::Icmp => {
                self.build_icmp_packet(target_ip)?;
                Ok("ICMP")
            }
            _ => {
                // For now, return error for unsupported packet types
                Err(PacketError::BuildFailed {
                    packet_type: format!("{:?}", packet_type),
                    reason: "Packet type not yet implemented in optimized pipeline".to_string()
                }.into())
            }
        }
    }
    
    /// Build UDP packet headers
    fn build_udp_packet(&mut self, target_ip: IpAddr, target_port: u16) -> Result<()> {
        match target_ip {
            IpAddr::V4(ipv4) => {
                // Ethernet header (if needed)
                let dst_mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
                let src_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
                self.packet_builder.ethernet_header(&dst_mac, &src_mac, 0x0800)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "UDP".to_string(),
                        reason: e.to_string()
                    })?;
                
                // IPv4 header
                let src_ip = 0xC0A80001; // 192.168.0.1
                let dst_ip = u32::from(ipv4);
                self.packet_builder.ipv4_header(14, src_ip, dst_ip, 17, 28)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "UDP".to_string(),
                        reason: e.to_string()
                    })?;
                
                // UDP header
                self.packet_builder.udp_header(34, 12345, target_port, 8)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "UDP".to_string(),
                        reason: e.to_string()
                    })?;
            }
            IpAddr::V6(_) => {
                return Err(PacketError::BuildFailed {
                    packet_type: "UDP".to_string(),
                    reason: "IPv6 not supported for UDP packet type".to_string()
                }.into());
            }
        }
        Ok(())
    }
    
    /// Build TCP packet headers
    fn build_tcp_packet(&mut self, target_ip: IpAddr, target_port: u16, flags: u8) -> Result<()> {
        match target_ip {
            IpAddr::V4(ipv4) => {
                // Ethernet header
                let dst_mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
                let src_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
                self.packet_builder.ethernet_header(&dst_mac, &src_mac, 0x0800)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "TCP".to_string(),
                        reason: e.to_string()
                    })?;
                
                // IPv4 header
                let src_ip = 0xC0A80001; // 192.168.0.1
                let dst_ip = u32::from(ipv4);
                self.packet_builder.ipv4_header(14, src_ip, dst_ip, 6, 40)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "TCP".to_string(),
                        reason: e.to_string()
                    })?;
                
                // TCP header
                self.packet_builder.tcp_header(34, 12345, target_port, 1000, 0, flags)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "TCP".to_string(),
                        reason: e.to_string()
                    })?;
            }
            IpAddr::V6(_) => {
                return Err(PacketError::BuildFailed {
                    packet_type: "TCP".to_string(),
                    reason: "IPv6 not supported for TCP packet type".to_string()
                }.into());
            }
        }
        Ok(())
    }
    
    /// Build ICMP packet headers
    fn build_icmp_packet(&mut self, target_ip: IpAddr) -> Result<()> {
        match target_ip {
            IpAddr::V4(ipv4) => {
                // Ethernet header
                let dst_mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
                let src_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
                self.packet_builder.ethernet_header(&dst_mac, &src_mac, 0x0800)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "ICMP".to_string(),
                        reason: e.to_string()
                    })?;
                
                // IPv4 header
                let src_ip = 0xC0A80001; // 192.168.0.1
                let dst_ip = u32::from(ipv4);
                self.packet_builder.ipv4_header(14, src_ip, dst_ip, 1, 28)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "ICMP".to_string(),
                        reason: e.to_string()
                    })?;
                
                // ICMP header (simplified)
                self.packet_builder.buffer.write_u8(34, 8)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "ICMP".to_string(),
                        reason: e.to_string()
                    })?; // Type: Echo Request
                self.packet_builder.buffer.write_u8(35, 0)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "ICMP".to_string(),
                        reason: e.to_string()
                    })?; // Code
                self.packet_builder.buffer.write_u16_be(36, 0)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "ICMP".to_string(),
                        reason: e.to_string()
                    })?; // Checksum
                self.packet_builder.buffer.write_u16_be(38, 1)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "ICMP".to_string(),
                        reason: e.to_string()
                    })?; // ID
                self.packet_builder.buffer.write_u16_be(40, 1)
                    .map_err(|e| PacketError::BuildFailed {
                        packet_type: "ICMP".to_string(),
                        reason: e.to_string()
                    })?; // Sequence
            }
            IpAddr::V6(_) => {
                return Err(PacketError::BuildFailed {
                    packet_type: "ICMP".to_string(),
                    reason: "IPv6 not supported for ICMP packet type".to_string()
                }.into());
            }
        }
        Ok(())
    }
    
    /// Fill payload with pattern (could be SIMD optimized)
    fn fill_payload(&self, payload: &mut [u8], size: usize) {
        // Simple pattern fill - could be optimized with SIMD
        for (i, byte) in payload.iter_mut().take(size).enumerate() {
            *byte = (i % 256) as u8;
        }
    }
    
    /// Get statistics collector
    pub fn stats_collector(&self) -> Arc<LockFreeStatsCollector> {
        self.stats_collector.clone()
    }
    
    /// Get memory manager
    pub fn memory_manager(&self) -> Arc<Memory> {
        self.memory_manager.clone()
    }
}

impl Default for BatchPacketProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of packet processing
pub struct ProcessedPacket<'a> {
    pub data: ManagedMemory<'a>,
    pub protocol_name: &'static str,
    pub processing_time: std::time::Duration,
    pub is_pooled: bool,
}

impl<'a> ProcessedPacket<'a> {
    /// Get the packet data
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
    
    /// Get the packet size
    pub fn size(&self) -> usize {
        self.data.size()
    }
    
    /// Get the protocol name
    pub fn protocol(&self) -> &str {
        self.protocol_name
    }
    
    /// Get processing time
    pub fn processing_time(&self) -> std::time::Duration {
        self.processing_time
    }
    
    /// Check if memory was allocated from pool
    pub fn is_pooled(&self) -> bool {
        self.is_pooled
    }
}

/// Performance metrics for the batch pipeline
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    pub packets_processed: u64,
    pub total_processing_time: std::time::Duration,
    pub pooled_allocations: u64,
    pub heap_allocations: u64,
    pub average_packet_size: f64,
}

impl PipelineMetrics {
    /// Calculate average processing time per packet
    pub fn average_processing_time(&self) -> std::time::Duration {
        if self.packets_processed == 0 {
            std::time::Duration::ZERO
        } else {
            self.total_processing_time / self.packets_processed as u32
        }
    }
    
    /// Calculate pool hit rate
    pub fn pool_hit_rate(&self) -> f64 {
        let total_allocations = self.pooled_allocations + self.heap_allocations;
        if total_allocations == 0 {
            0.0
        } else {
            (self.pooled_allocations as f64 / total_allocations as f64) * 100.0
        }
    }
}

// Tests moved to tests/ directory
