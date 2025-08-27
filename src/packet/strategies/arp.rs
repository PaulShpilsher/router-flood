//! ARP packet building strategy

use super::PacketStrategy;
use crate::constants::{ETHERNET_HEADER_SIZE, ARP_PACKET_SIZE};
use crate::error::{PacketError, Result};
use crate::packet::Target;
use crate::rng::BatchedRng;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
use pnet::packet::MutablePacket;
use pnet::util::MacAddr;
use std::net::{IpAddr, Ipv4Addr};

pub struct ArpStrategy {
    source_ip: Ipv4Addr,
    source_mac: MacAddr,
    rng: BatchedRng,
}

impl ArpStrategy {
    pub fn new(rng: &mut BatchedRng) -> Self {
        let source_ip = Ipv4Addr::new(192, 168, 1, rng.range(2, 254) as u8);
        let source_mac = MacAddr::new(
            0x02,
            rng.byte(),
            rng.byte(),
            rng.byte(),
            rng.byte(),
            rng.byte(),
        );
        
        Self {
            source_ip,
            source_mac,
            rng: BatchedRng::new(),
        }
    }
}

impl PacketStrategy for ArpStrategy {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let target_ip = match target.ip {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => {
                return Err(PacketError::InvalidParameters(
                    "ARP strategy requires IPv4 target".to_string()
                ).into());
            }
        };

        let total_len = ETHERNET_HEADER_SIZE + ARP_PACKET_SIZE;
        
        if buffer.len() < total_len {
            return Err(PacketError::BufferTooSmall {
                required: total_len,
                available: buffer.len(),
            }.into());
        }

        // Zero out the buffer area we'll use
        buffer[..total_len].fill(0);

        // Build Ethernet header
        let mut ethernet_packet = MutableEthernetPacket::new(&mut buffer[..total_len])
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "ARP".to_string(),
                reason: "Failed to create Ethernet packet".to_string(),
            })?;
        
        ethernet_packet.set_destination(MacAddr::broadcast());
        ethernet_packet.set_source(self.source_mac);
        ethernet_packet.set_ethertype(EtherTypes::Arp);

        // Build ARP packet
        let mut arp_packet = MutableArpPacket::new(ethernet_packet.payload_mut())
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "ARP".to_string(),
                reason: "Failed to create ARP packet".to_string(),
            })?;
        
        arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_packet.set_protocol_type(EtherTypes::Ipv4);
        arp_packet.set_hw_addr_len(6);
        arp_packet.set_proto_addr_len(4);
        arp_packet.set_operation(ArpOperations::Request);
        arp_packet.set_sender_hw_addr(self.source_mac);
        arp_packet.set_sender_proto_addr(self.source_ip);
        arp_packet.set_target_hw_addr(MacAddr::zero());
        arp_packet.set_target_proto_addr(target_ip);

        Ok(total_len)
    }

    fn protocol_name(&self) -> &'static str {
        "ARP"
    }

    fn max_packet_size(&self) -> usize {
        ETHERNET_HEADER_SIZE + ARP_PACKET_SIZE
    }

    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        matches!(target_ip, IpAddr::V4(_))
    }
}