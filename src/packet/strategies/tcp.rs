//! TCP packet building strategy

use super::PacketStrategy;
use crate::constants::{IPV4_HEADER_SIZE, TCP_HEADER_SIZE};
use crate::error::{PacketError, Result};
use crate::packet::PacketTarget;
use crate::utils::rng::BatchedRng;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::packet::MutablePacket;
use std::net::{IpAddr, Ipv4Addr};

pub struct TcpStrategy {
    source_ip: Ipv4Addr,
    tcp_flags: u8,
    rng: BatchedRng,
}

impl TcpStrategy {
    pub fn new_syn(rng: &mut BatchedRng) -> Self {
        let source_ip = Ipv4Addr::new(192, 168, 1, rng.range(2, 254) as u8);
        
        Self {
            source_ip,
            tcp_flags: TcpFlags::SYN,
            rng: BatchedRng::new(),
        }
    }

    pub fn new_ack(rng: &mut BatchedRng) -> Self {
        let source_ip = Ipv4Addr::new(192, 168, 1, rng.range(2, 254) as u8);
        
        Self {
            source_ip,
            tcp_flags: TcpFlags::ACK,
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
        ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
        ip_packet.set_source(self.source_ip);
        ip_packet.set_destination(target_ip);
        ip_packet.set_identification(self.rng.identification());

        // Occasionally set fragmentation flags
        if self.rng.bool_with_probability(0.1) {
            ip_packet.set_flags(2); // Don't fragment
        }
    }
}

impl PacketStrategy for TcpStrategy {
    fn build_packet(&mut self, target: &PacketTarget, buffer: &mut [u8]) -> Result<usize> {
        let target_ip = match target.ip {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => {
                return Err(PacketError::InvalidParameters(
                    "TCP strategy requires IPv4 target".to_string()
                ).into());
            }
        };

        let total_len = IPV4_HEADER_SIZE + TCP_HEADER_SIZE; // No payload for SYN/ACK
        
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
                packet_type: "TCP".to_string(),
                reason: "Failed to create IPv4 packet".to_string(),
            })?;
        
        self.setup_ip_header(&mut ip_packet, total_len, target_ip);

        // Build TCP packet
        let mut tcp_packet = MutableTcpPacket::new(ip_packet.payload_mut())
            .ok_or_else(|| PacketError::BuildFailed {
                packet_type: "TCP".to_string(),
                reason: "Failed to create TCP packet".to_string(),
            })?;
        
        tcp_packet.set_source(self.rng.port());
        tcp_packet.set_destination(target.port);
        tcp_packet.set_sequence(self.rng.sequence());
        tcp_packet.set_acknowledgement(if self.tcp_flags == TcpFlags::ACK {
            self.rng.sequence()
        } else {
            0
        });
        tcp_packet.set_data_offset(5);
        tcp_packet.set_flags(self.tcp_flags);
        tcp_packet.set_window(self.rng.window_size());
        tcp_packet.set_urgent_ptr(0);
        tcp_packet.set_checksum(pnet::packet::tcp::ipv4_checksum(
            &tcp_packet.to_immutable(),
            &self.source_ip,
            &target_ip,
        ));

        // Set IP checksum last
        ip_packet.set_checksum(pnet::packet::ipv4::checksum(&ip_packet.to_immutable()));
        
        Ok(total_len)
    }

    fn protocol_name(&self) -> &'static str {
        "TCP"
    }

    fn max_packet_size(&self) -> usize {
        IPV4_HEADER_SIZE + TCP_HEADER_SIZE
    }

    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        matches!(target_ip, IpAddr::V4(_))
    }
}