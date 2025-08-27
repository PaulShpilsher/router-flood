//! IPv6 TCP packet building strategy

use super::PacketStrategy;
use crate::constants::{IPV6_HEADER_SIZE, TCP_HEADER_SIZE};
use crate::error::{PacketError, Result};
use crate::packet::Target;
use crate::rng::BatchedRng;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::packet::MutablePacket;
use std::net::{IpAddr, Ipv6Addr};

pub struct Ipv6TcpStrategy {
    source_ipv6: Ipv6Addr,
    rng: BatchedRng,
}

impl Ipv6TcpStrategy {
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

impl PacketStrategy for Ipv6TcpStrategy {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let target_ip = match target.ip {
            IpAddr::V6(ip) => ip,
            IpAddr::V4(_) => {
                return Err(PacketError::InvalidParameters(
                    "IPv6 TCP strategy requires IPv6 target".to_string()
                ).into());
            }
        };

        let total_len = IPV6_HEADER_SIZE + TCP_HEADER_SIZE;
        
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
                packet_type: "IPv6-TCP".to_string(),
                reason: "Failed to create IPv6 packet".to_string(),
            })?;
        
        ip_packet.set_version(6);
        ip_packet.set_traffic_class(0);
        ip_packet.set_flow_label(self.rng.flow_label());
        ip_packet.set_payload_length(TCP_HEADER_SIZE as u16);
        ip_packet.set_next_header(IpNextHeaderProtocols::Tcp);
        ip_packet.set_hop_limit(self.rng.ttl());
        ip_packet.set_source(self.source_ipv6);
        ip_packet.set_destination(target_ip);

        // Build TCP packet
        let mut tcp_packet = MutableTcpPacket::new(ip_packet.payload_mut())
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "IPv6-TCP".to_string(),
                reason: "Failed to create TCP packet".to_string(),
            })?;
        
        tcp_packet.set_source(self.rng.port());
        tcp_packet.set_destination(target.port);
        tcp_packet.set_sequence(self.rng.sequence());
        tcp_packet.set_acknowledgement(self.rng.sequence());
        tcp_packet.set_data_offset(5);
        tcp_packet.set_flags(TcpFlags::SYN); // Default to SYN for IPv6
        tcp_packet.set_window(self.rng.window_size());
        tcp_packet.set_urgent_ptr(0);
        tcp_packet.set_checksum(pnet::packet::tcp::ipv6_checksum(
            &tcp_packet.to_immutable(),
            &self.source_ipv6,
            &target_ip,
        ));

        Ok(total_len)
    }

    fn protocol_name(&self) -> &'static str {
        "IPv6"
    }

    fn max_packet_size(&self) -> usize {
        IPV6_HEADER_SIZE + TCP_HEADER_SIZE
    }

    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        matches!(target_ip, IpAddr::V6(_))
    }
}