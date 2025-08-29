//! IPv6 UDP packet building strategy

use super::PacketStrategy;
use crate::constants::{IPV6_HEADER_SIZE, UDP_HEADER_SIZE};
use crate::error::{PacketError, Result};
use crate::packet::Target;
use crate::utils::rng::BatchedRng;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::MutablePacket;
use std::net::{IpAddr, Ipv6Addr};

pub struct Ipv6UdpStrategy {
    source_ipv6: Ipv6Addr,
    packet_size_range: (usize, usize),
    rng: BatchedRng,
}

impl Ipv6UdpStrategy {
    pub fn new(packet_size_range: (usize, usize), rng: &mut BatchedRng) -> Self {
        let source_ipv6 = Ipv6Addr::new(
            0xfe80,
            0,
            0,
            0,
            rng.identification(),
            rng.identification(),
            rng.identification(),
            rng.identification(),
        );
        
        Self {
            source_ipv6,
            packet_size_range,
            rng: BatchedRng::new(),
        }
    }

    fn random_payload_size(&mut self) -> usize {
        // More realistic payload size distribution
        let min_size = self.packet_size_range.0;
        let max_size = self.packet_size_range.1;
        
        // Ensure we don't create empty ranges
        match self.rng.range(0, 100) {
            0..=40 => {
                // Small packets: use min_size to min(200, max_size)
                let upper = std::cmp::min(200, max_size);
                if min_size <= upper {
                    self.rng.range(min_size, upper + 1)
                } else {
                    min_size
                }
            },
            41..=80 => {
                // Medium packets: use 200 to min(800, max_size)
                let lower = std::cmp::max(200, min_size);
                let upper = std::cmp::min(800, max_size);
                if lower <= upper {
                    self.rng.range(lower, upper + 1)
                } else {
                    // Fallback to valid range
                    self.rng.range(min_size, max_size + 1)
                }
            },
            _ => {
                // Large packets: use max(800, min_size) to max_size
                let lower = std::cmp::max(800, min_size);
                if lower <= max_size {
                    self.rng.range(lower, max_size + 1)
                } else {
                    // Fallback to valid range
                    self.rng.range(min_size, max_size + 1)
                }
            }
        }
    }
}

impl PacketStrategy for Ipv6UdpStrategy {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let target_ip = match target.ip {
            IpAddr::V6(ip) => ip,
            IpAddr::V4(_) => {
                return Err(PacketError::InvalidParameters(
                    "IPv6 UDP strategy requires IPv6 target".to_string()
                ).into());
            }
        };

        let payload_size = self.random_payload_size();
        let total_len = IPV6_HEADER_SIZE + UDP_HEADER_SIZE + payload_size;
        
        if buffer.len() < total_len {
            return Err(PacketError::BufferTooSmall {
                required: total_len,
                available: buffer.len(),
            }.into());
        }

        // Zero out the buffer area we'll use
        buffer[..total_len].fill(0);

        // Build IPv6 header
        let mut ip_packet = MutableIpv6Packet::new(&mut buffer[..total_len])
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "IPv6-UDP".to_string(),
                reason: "Failed to create IPv6 packet".to_string(),
            })?;
        
        ip_packet.set_version(6);
        ip_packet.set_traffic_class(0);
        ip_packet.set_flow_label(self.rng.flow_label());
        ip_packet.set_payload_length((UDP_HEADER_SIZE + payload_size) as u16);
        ip_packet.set_next_header(IpNextHeaderProtocols::Udp);
        ip_packet.set_hop_limit(self.rng.ttl());
        ip_packet.set_source(self.source_ipv6);
        ip_packet.set_destination(target_ip);

        // Build UDP header + payload
        let mut udp_packet = MutableUdpPacket::new(ip_packet.payload_mut())
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "IPv6-UDP".to_string(),
                reason: "Failed to create UDP packet".to_string(),
            })?;
        
        udp_packet.set_source(self.rng.port());
        udp_packet.set_destination(target.port);
        udp_packet.set_length((UDP_HEADER_SIZE + payload_size) as u16);

        let payload = self.rng.payload(payload_size);
        udp_packet.set_payload(&payload);
        udp_packet.set_checksum(pnet::packet::udp::ipv6_checksum(
            &udp_packet.to_immutable(),
            &self.source_ipv6,
            &target_ip,
        ));

        Ok(total_len)
    }

    fn protocol_name(&self) -> &'static str {
        "IPv6"
    }

    fn max_packet_size(&self) -> usize {
        IPV6_HEADER_SIZE + UDP_HEADER_SIZE + self.packet_size_range.1
    }

    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        matches!(target_ip, IpAddr::V6(_))
    }
}