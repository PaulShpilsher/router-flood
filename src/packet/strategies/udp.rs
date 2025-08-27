//! UDP packet building strategy

use super::PacketStrategy;
use crate::constants::{IPV4_HEADER_SIZE, UDP_HEADER_SIZE};
use crate::error::{PacketError, Result};
use crate::packet::Target;
use crate::rng::BatchedRng;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::MutablePacket;
use std::net::{IpAddr, Ipv4Addr};

pub struct UdpStrategy {
    source_ip: Ipv4Addr,
    packet_size_range: (usize, usize),
    rng: BatchedRng,
}

impl UdpStrategy {
    pub fn new(packet_size_range: (usize, usize), rng: &mut BatchedRng) -> Self {
        let source_ip = Ipv4Addr::new(192, 168, 1, rng.range(2, 254) as u8);
        
        Self {
            source_ip,
            packet_size_range,
            rng: BatchedRng::new(),
        }
    }

    #[inline]
    fn random_payload_size(&mut self) -> usize {
        // More realistic payload size distribution
        match self.rng.range(0, 100) {
            0..=40 => self.rng.range(self.packet_size_range.0, 200 + 1), // Small packets
            41..=80 => self.rng.range(200, 800 + 1),                     // Medium packets
            _ => self.rng.range(800, self.packet_size_range.1 + 1),      // Large packets
        }
    }

    #[inline]
    fn setup_ip_header(
        &mut self,
        ip_packet: &mut MutableIpv4Packet,
        total_len: usize,
        target_ip: Ipv4Addr,
    ) {
        ip_packet.set_version(4);
        ip_packet.set_header_length(5);
        ip_packet.set_total_length(total_len as u16);
        ip_packet.set_ttl(self.rng.ttl());
        ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Udp);
        ip_packet.set_source(self.source_ip);
        ip_packet.set_destination(target_ip);
        ip_packet.set_identification(self.rng.identification());

        // Occasionally set fragmentation flags
        if self.rng.bool_with_probability(0.1) {
            ip_packet.set_flags(2); // Don't fragment
        }
    }
}

impl PacketStrategy for UdpStrategy {
    #[inline]
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let target_ip = match target.ip {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => {
                return Err(PacketError::InvalidParameters(
                    "UDP strategy requires IPv4 target".to_string()
                ).into());
            }
        };

        let payload_size = self.random_payload_size();
        let total_len = IPV4_HEADER_SIZE + UDP_HEADER_SIZE + payload_size;
        
        if buffer.len() < total_len {
            return Err(PacketError::BufferTooSmall {
                required: total_len,
                available: buffer.len(),
            }.into());
        }

        // Zero out the buffer area we'll use
        buffer[..total_len].fill(0);

        // Build IP header
        let mut ip_packet = MutableIpv4Packet::new(&mut buffer[..total_len])
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "UDP".to_string(),
                reason: "Failed to create IPv4 packet".to_string(),
            })?;
        
        self.setup_ip_header(&mut ip_packet, total_len, target_ip);

        // Build UDP header + payload
        let mut udp_packet = MutableUdpPacket::new(ip_packet.payload_mut())
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "UDP".to_string(),
                reason: "Failed to create UDP packet".to_string(),
            })?;
        
        udp_packet.set_source(self.rng.port());
        udp_packet.set_destination(target.port);
        udp_packet.set_length((UDP_HEADER_SIZE + payload_size) as u16);

        let payload = self.rng.payload(payload_size);
        udp_packet.set_payload(&payload);
        udp_packet.set_checksum(pnet::packet::udp::ipv4_checksum(
            &udp_packet.to_immutable(),
            &self.source_ip,
            &target_ip,
        ));

        // Set IP checksum last
        ip_packet.set_checksum(pnet::packet::ipv4::checksum(&ip_packet.to_immutable()));
        
        Ok(total_len)
    }

    #[inline(always)]
    fn protocol_name(&self) -> &'static str {
        "UDP"
    }

    #[inline(always)]
    fn max_packet_size(&self) -> usize {
        IPV4_HEADER_SIZE + UDP_HEADER_SIZE + self.packet_size_range.1
    }

    #[inline(always)]
    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        matches!(target_ip, IpAddr::V4(_))
    }
}