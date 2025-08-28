//! Transport layer abstractions
//!
//! This module provides trait-based abstractions for different transport
//! mechanisms, enabling easy testing and multiple implementations.

pub mod layer;
pub mod mock;

pub use layer::{TransportLayer, ChannelType};
pub use mock::MockTransport;

use crate::constants::TRANSPORT_BUFFER_SIZE;
use crate::error::{NetworkError, Result};
use pnet::transport::{transport_channel, TransportChannelType, TransportSender};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::datalink::{channel, Channel, DataLinkSender, NetworkInterface};
use std::net::IpAddr;

/// Manages transport channels for a single worker thread
/// 
/// This eliminates mutex contention by giving each worker its own channels
pub struct WorkerChannels {
    pub ipv4_sender: Option<TransportSender>,
    pub ipv6_sender: Option<TransportSender>,
    pub l2_sender: Option<Box<dyn DataLinkSender>>,
}

impl WorkerChannels {
    /// Create a new set of transport channels for a single worker
    pub fn new(interface: Option<&NetworkInterface>, dry_run: bool) -> Result<Self> {
        if dry_run {
            return Ok(Self {
                ipv4_sender: None,
                ipv6_sender: None,
                l2_sender: None,
            });
        }

        let ipv4_sender = Self::create_ipv4_channel()?;
        let ipv6_sender = Self::create_ipv6_channel()?;
        let l2_sender = Self::create_l2_channel(interface)?;

        Ok(Self {
            ipv4_sender: Some(ipv4_sender),
            ipv6_sender: Some(ipv6_sender),
            l2_sender,
        })
    }

    /// Create IPv4 transport channel
    fn create_ipv4_channel() -> Result<TransportSender> {
        let (tx, _) = transport_channel(
            TRANSPORT_BUFFER_SIZE,
            TransportChannelType::Layer3(IpNextHeaderProtocols::Ipv4),
        )
        .map_err(|e| NetworkError::ChannelCreation(format!("IPv4 channel: {}", e)))?;
        
        Ok(tx)
    }

    /// Create IPv6 transport channel
    fn create_ipv6_channel() -> Result<TransportSender> {
        let (tx, _) = transport_channel(
            TRANSPORT_BUFFER_SIZE,
            TransportChannelType::Layer3(IpNextHeaderProtocols::Ipv6),
        )
        .map_err(|e| NetworkError::ChannelCreation(format!("IPv6 channel: {}", e)))?;
        
        Ok(tx)
    }

    /// Create Layer 2 channel for ARP packets
    fn create_l2_channel(interface: Option<&NetworkInterface>) -> Result<Option<Box<dyn DataLinkSender>>> {
        if let Some(iface) = interface {
            match channel(iface, Default::default()) {
                Ok(Channel::Ethernet(tx, _)) => Ok(Some(tx)),
                Ok(_) => Err(NetworkError::ChannelCreation("Unknown L2 channel type".to_string()).into()),
                Err(e) => Err(NetworkError::ChannelCreation(format!("L2 channel: {}", e)).into()),
            }
        } else {
            Ok(None)
        }
    }

    /// Send packet through appropriate channel based on target IP type
    pub fn send_packet(&mut self, packet_data: &[u8], target_ip: IpAddr, channel_type: ChannelType) -> Result<()> {
        match channel_type {
            ChannelType::IPv4 => self.send_ipv4_packet(packet_data, target_ip),
            ChannelType::IPv6 => self.send_ipv6_packet(packet_data, target_ip),
            ChannelType::Layer2 => self.send_l2_packet(packet_data),
        }
    }

    /// Send IPv4 packet
    fn send_ipv4_packet(&mut self, packet_data: &[u8], target_ip: IpAddr) -> Result<()> {
        if let Some(ref mut tx) = self.ipv4_sender {
            let packet = pnet::packet::ipv4::Ipv4Packet::new(packet_data)
                .ok_or_else(|| NetworkError::PacketSend("Invalid IPv4 packet data".to_string()))?;
            
            tx.send_to(packet, target_ip)
                .map_err(|e| NetworkError::PacketSend(format!("Failed to send IPv4 packet: {}", e)))?;
        }
        Ok(())
    }

    /// Send IPv6 packet
    fn send_ipv6_packet(&mut self, packet_data: &[u8], target_ip: IpAddr) -> Result<()> {
        if let Some(ref mut tx) = self.ipv6_sender {
            let packet = pnet::packet::ipv6::Ipv6Packet::new(packet_data)
                .ok_or_else(|| NetworkError::PacketSend("Invalid IPv6 packet data".to_string()))?;
            
            tx.send_to(packet, target_ip)
                .map_err(|e| NetworkError::PacketSend(format!("Failed to send IPv6 packet: {}", e)))?;
        }
        Ok(())
    }

    /// Send Layer 2 packet
    fn send_l2_packet(&mut self, packet_data: &[u8]) -> Result<()> {
        if let Some(ref mut tx) = self.l2_sender {
            match tx.send_to(packet_data, None) {
                Some(Ok(())) => {}
                Some(Err(e)) => return Err(NetworkError::PacketSend(format!("Failed to send L2 packet: {}", e)).into()),
                None => return Err(NetworkError::PacketSend("L2 send returned None".to_string()).into()),
            }
        }
        Ok(())
    }
}

/// Factory for creating worker channels in batch
pub struct ChannelFactory;

impl ChannelFactory {
    /// Create channels for all workers
    pub fn create_worker_channels(
        worker_count: usize,
        interface: Option<&NetworkInterface>,
        dry_run: bool,
    ) -> Result<Vec<WorkerChannels>> {
        let mut channels = Vec::with_capacity(worker_count);
        
        for _ in 0..worker_count {
            channels.push(WorkerChannels::new(interface, dry_run)?);
        }
        
        Ok(channels)
    }
}