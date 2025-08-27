//! ICMP packet building strategy

use super::PacketStrategy;
use crate::constants::{IPV4_HEADER_SIZE, ICMP_HEADER_SIZE, icmp};
use crate::error::{PacketError, Result};
use crate::packet::Target;
use crate::rng::BatchedRng;
use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::MutablePacket;
use std::net::{IpAddr, Ipv4Addr};

pub struct IcmpStrategy {
    source_ip: Ipv4Addr,
    rng: BatchedRng,
}

impl IcmpStrategy {
    pub fn new(rng: &mut BatchedRng) -> Self {
        let source_ip = Ipv4Addr::new(192, 168, 1, rng.range(2, 254) as u8);
        
        Self {
            source_ip,
            rng: BatchedRng::new(),
        }
    }

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
        ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
        ip_packet.set_source(self.source_ip);
        ip_packet.set_destination(target_ip);
        ip_packet.set_identification(self.rng.identification());

        // Occasionally set fragmentation flags
        if self.rng.bool_with_probability(0.1) {
            ip_packet.set_flags(2); // Don't fragment
        }
    }
}

impl PacketStrategy for IcmpStrategy {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let target_ip = match target.ip {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => {
                return Err(PacketError::InvalidParameters(
                    "ICMP strategy requires IPv4 target".to_string()
                ).into());
            }
        };

        let payload_size = self.rng.range(icmp::MIN_PING_SIZE, icmp::MAX_PING_SIZE + 1);
        let total_len = IPV4_HEADER_SIZE + ICMP_HEADER_SIZE + payload_size;
        
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
                packet_type: "ICMP".to_string(),
                reason: "Failed to create IPv4 packet".to_string(),
            })?;
        
        self.setup_ip_header(&mut ip_packet, total_len, target_ip);

        // Build ICMP packet
        let mut icmp_packet = MutableIcmpPacket::new(ip_packet.payload_mut())
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "ICMP".to_string(),
                reason: "Failed to create ICMP packet".to_string(),
            })?;
        
        icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
        icmp_packet.set_icmp_code(pnet::packet::icmp::IcmpCode(0));
        icmp_packet.set_checksum(0);

        // Add payload
        let payload = self.rng.payload(payload_size);
        icmp_packet.set_payload(&payload);

        // Calculate and set ICMP checksum
        let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
        icmp_packet.set_checksum(checksum);

        // Set IP checksum last
        ip_packet.set_checksum(pnet::packet::ipv4::checksum(&ip_packet.to_immutable()));
        
        Ok(total_len)
    }

    fn protocol_name(&self) -> &'static str {
        "ICMP"
    }

    fn max_packet_size(&self) -> usize {
        IPV4_HEADER_SIZE + ICMP_HEADER_SIZE + icmp::MAX_PING_SIZE
    }

    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        matches!(target_ip, IpAddr::V4(_))
    }
}