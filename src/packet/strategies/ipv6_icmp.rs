//! IPv6 ICMP packet building strategy

use super::PacketStrategy;
use crate::constants::{IPV6_HEADER_SIZE, ICMP_HEADER_SIZE, icmp};
use crate::error::{PacketError, Result};
use crate::packet::PacketTarget;
use crate::utils::rng::BatchedRng;
use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::MutablePacket;
use std::net::{IpAddr, Ipv6Addr};

pub struct Ipv6IcmpStrategy {
    source_ipv6: Ipv6Addr,
    rng: BatchedRng,
}

impl Ipv6IcmpStrategy {
    pub fn new(rng: &mut BatchedRng) -> Self {
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
            rng: BatchedRng::new(),
        }
    }
}

impl PacketStrategy for Ipv6IcmpStrategy {
    fn build_packet(&mut self, target: &PacketTarget, buffer: &mut [u8]) -> Result<usize> {
        let target_ip = match target.ip {
            IpAddr::V6(ip) => ip,
            IpAddr::V4(_) => {
                return Err(PacketError::InvalidParameters(
                    "IPv6 ICMP strategy requires IPv6 target".to_string()
                ).into());
            }
        };

        let payload_size = self.rng.range(icmp::MIN_PING_SIZE, icmp::MAX_PING_SIZE + 1);
        let total_len = IPV6_HEADER_SIZE + ICMP_HEADER_SIZE + payload_size;
        
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
                packet_type: "IPv6-ICMP".to_string(),
                reason: "Failed to create IPv6 packet".to_string(),
            })?;
        
        ip_packet.set_version(6);
        ip_packet.set_traffic_class(0);
        ip_packet.set_flow_label(self.rng.flow_label());
        ip_packet.set_payload_length((ICMP_HEADER_SIZE + payload_size) as u16);
        ip_packet.set_next_header(IpNextHeaderProtocols::Icmpv6);
        ip_packet.set_hop_limit(self.rng.ttl());
        ip_packet.set_source(self.source_ipv6);
        ip_packet.set_destination(target_ip);

        // Build ICMPv6 packet (simplified - using ICMP structure)
        let mut icmp_packet = MutableIcmpPacket::new(ip_packet.payload_mut())
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "IPv6-ICMP".to_string(),
                reason: "Failed to create ICMP packet".to_string(),
            })?;
        
        icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
        icmp_packet.set_icmp_code(pnet::packet::icmp::IcmpCode(0));
        icmp_packet.set_checksum(0);

        let payload = self.rng.payload(payload_size);
        icmp_packet.set_payload(&payload);

        // ICMPv6 checksum calculation would be more complex in real implementation
        let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
        icmp_packet.set_checksum(checksum);

        Ok(total_len)
    }

    fn protocol_name(&self) -> &'static str {
        "IPv6"
    }

    fn max_packet_size(&self) -> usize {
        IPV6_HEADER_SIZE + ICMP_HEADER_SIZE + icmp::MAX_PING_SIZE
    }

    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        matches!(target_ip, IpAddr::V6(_))
    }
}