use crate::config::ProtocolMix;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::MutablePacket;
use pnet::util::MacAddr;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Supported packet types for enhanced flood simulation
#[derive(Debug, Clone, Copy)]
pub enum PacketType {
    Udp,
    TcpSyn,
    TcpAck,
    Icmp,
    Ipv6Udp,
    Ipv6Tcp,
    Ipv6Icmp,
    Arp,
}

/// Enhanced packet builder with multiple protocol support and realistic traffic patterns
pub struct PacketBuilder {
    rng: StdRng,
    source_ip: Ipv4Addr,
    source_ipv6: Ipv6Addr,
    source_mac: MacAddr,
    packet_size_range: (usize, usize),
    protocol_mix: ProtocolMix,
}

impl PacketBuilder {
    pub fn new(packet_size_range: (usize, usize), protocol_mix: ProtocolMix) -> Self {
        let mut rng = StdRng::from_entropy();
        let source_ip = Ipv4Addr::new(192, 168, 1, rng.gen_range(2..254));
        let source_ipv6 = Ipv6Addr::new(
            0xfe80,
            0,
            0,
            0,
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
        );
        let source_mac = MacAddr::new(
            0x02,
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
        );

        Self {
            rng,
            source_ip,
            source_ipv6,
            source_mac,
            packet_size_range,
            protocol_mix,
        }
    }

    pub fn rng_gen_bool(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability)
    }

    pub fn rng_gen_range(&mut self, range: std::ops::Range<f64>) -> f64 {
        self.rng.gen_range(range)
    }

    pub fn next_packet_type(&mut self) -> PacketType {
        let rand_val = self.rng.gen::<f64>();
        let mut cumulative = 0.0;

        cumulative += self.protocol_mix.udp_ratio;
        if rand_val < cumulative {
            return PacketType::Udp;
        }

        cumulative += self.protocol_mix.tcp_syn_ratio;
        if rand_val < cumulative {
            return PacketType::TcpSyn;
        }

        cumulative += self.protocol_mix.tcp_ack_ratio;
        if rand_val < cumulative {
            return PacketType::TcpAck;
        }

        cumulative += self.protocol_mix.icmp_ratio;
        if rand_val < cumulative {
            return PacketType::Icmp;
        }

        cumulative += self.protocol_mix.ipv6_ratio;
        if rand_val < cumulative {
            match self.rng.gen_range(0..3) {
                0 => return PacketType::Ipv6Udp,
                1 => return PacketType::Ipv6Tcp,
                _ => return PacketType::Ipv6Icmp,
            }
        }

        PacketType::Arp
    }

    fn random_payload_size(&mut self) -> usize {
        // More realistic payload size distribution
        match self.rng.gen_range(0..100) {
            0..=40 => self.rng.gen_range(self.packet_size_range.0..=200), // Small packets
            41..=80 => self.rng.gen_range(200..=800),                     // Medium packets
            _ => self.rng.gen_range(800..=self.packet_size_range.1),      // Large packets
        }
    }

    pub fn build_packet(
        &mut self,
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
    ) -> Result<(Vec<u8>, &'static str), String> {
        match packet_type {
            PacketType::Udp => {
                if let IpAddr::V4(ipv4) = target_ip {
                    Ok((self.build_udp_packet(ipv4, target_port)?, "UDP"))
                } else {
                    Err("UDP packet requires IPv4 target".to_string())
                }
            }
            PacketType::TcpSyn => {
                if let IpAddr::V4(ipv4) = target_ip {
                    Ok((
                        self.build_tcp_packet(ipv4, target_port, TcpFlags::SYN)?,
                        "TCP",
                    ))
                } else {
                    Err("TCP SYN packet requires IPv4 target".to_string())
                }
            }
            PacketType::TcpAck => {
                if let IpAddr::V4(ipv4) = target_ip {
                    Ok((
                        self.build_tcp_packet(ipv4, target_port, TcpFlags::ACK)?,
                        "TCP",
                    ))
                } else {
                    Err("TCP ACK packet requires IPv4 target".to_string())
                }
            }
            PacketType::Icmp => {
                if let IpAddr::V4(ipv4) = target_ip {
                    Ok((self.build_icmp_packet(ipv4)?, "ICMP"))
                } else {
                    Err("ICMP packet requires IPv4 target".to_string())
                }
            }
            PacketType::Ipv6Udp => {
                if let IpAddr::V6(ipv6) = target_ip {
                    Ok((self.build_ipv6_udp_packet(ipv6, target_port)?, "IPv6"))
                } else {
                    Err("IPv6 UDP packet requires IPv6 target".to_string())
                }
            }
            PacketType::Ipv6Tcp => {
                if let IpAddr::V6(ipv6) = target_ip {
                    Ok((self.build_ipv6_tcp_packet(ipv6, target_port)?, "IPv6"))
                } else {
                    Err("IPv6 TCP packet requires IPv6 target".to_string())
                }
            }
            PacketType::Ipv6Icmp => {
                if let IpAddr::V6(ipv6) = target_ip {
                    Ok((self.build_ipv6_icmp_packet(ipv6)?, "IPv6"))
                } else {
                    Err("IPv6 ICMP packet requires IPv6 target".to_string())
                }
            }
            PacketType::Arp => {
                if let IpAddr::V4(ipv4) = target_ip {
                    Ok((self.build_arp_packet(ipv4)?, "ARP"))
                } else {
                    Err("ARP packet requires IPv4 target".to_string())
                }
            }
        }
    }

    fn build_udp_packet(
        &mut self,
        target_ip: Ipv4Addr,
        target_port: u16,
    ) -> Result<Vec<u8>, String> {
        let payload_size = self.random_payload_size();
        let total_len = 20 + 8 + payload_size; // IP + UDP + payload
        let mut packet_buf = vec![0u8; total_len];

        // Build IP header
        let mut ip_packet = MutableIpv4Packet::new(&mut packet_buf).unwrap();
        self.setup_ip_header(&mut ip_packet, total_len, IpNextHeaderProtocols::Udp, target_ip);

        // Build UDP header + payload
        let mut udp_packet = MutableUdpPacket::new(ip_packet.payload_mut()).unwrap();
        udp_packet.set_source(self.rng.gen_range(1024..65535));
        udp_packet.set_destination(target_port);
        udp_packet.set_length((8 + payload_size) as u16);

        let payload: Vec<u8> = (0..payload_size).map(|_| self.rng.gen()).collect();
        udp_packet.set_payload(&payload);
        udp_packet.set_checksum(pnet::packet::udp::ipv4_checksum(
            &udp_packet.to_immutable(),
            &self.source_ip,
            &target_ip,
        ));

        // Set IP checksum last
        ip_packet.set_checksum(pnet::packet::ipv4::checksum(&ip_packet.to_immutable()));
        Ok(packet_buf)
    }

    fn build_tcp_packet(
        &mut self,
        target_ip: Ipv4Addr,
        target_port: u16,
        flags: u8,
    ) -> Result<Vec<u8>, String> {
        let total_len = 20 + 20; // IP + TCP (no payload for SYN/ACK)
        let mut packet_buf = vec![0u8; total_len];

        // Build IP header
        let mut ip_packet = MutableIpv4Packet::new(&mut packet_buf).unwrap();
        self.setup_ip_header(&mut ip_packet, total_len, IpNextHeaderProtocols::Tcp, target_ip);

        // Build TCP packet
        let mut tcp_packet = MutableTcpPacket::new(ip_packet.payload_mut()).unwrap();
        tcp_packet.set_source(self.rng.gen_range(1024..65535));
        tcp_packet.set_destination(target_port);
        tcp_packet.set_sequence(self.rng.gen());
        tcp_packet.set_acknowledgement(if flags == TcpFlags::ACK {
            self.rng.gen()
        } else {
            0
        });
        tcp_packet.set_data_offset(5);
        tcp_packet.set_flags(TcpFlags::SYN);
        tcp_packet.set_window(self.rng.gen_range(1024..65535));
        tcp_packet.set_urgent_ptr(0);
        tcp_packet.set_checksum(pnet::packet::tcp::ipv4_checksum(
            &tcp_packet.to_immutable(),
            &self.source_ip,
            &target_ip,
        ));

        // Set IP checksum last
        ip_packet.set_checksum(pnet::packet::ipv4::checksum(&ip_packet.to_immutable()));
        Ok(packet_buf)
    }

    fn build_icmp_packet(&mut self, target_ip: Ipv4Addr) -> Result<Vec<u8>, String> {
        let payload_size = self.rng.gen_range(8..=56); // Standard ping sizes
        let total_len = 20 + 8 + payload_size; // IP + ICMP + payload
        let mut packet_buf = vec![0u8; total_len];

        // Build IP header
        let mut ip_packet = MutableIpv4Packet::new(&mut packet_buf).unwrap();
        self.setup_ip_header(&mut ip_packet, total_len, IpNextHeaderProtocols::Icmp, target_ip);

        // Build ICMP packet
        let mut icmp_packet = MutableIcmpPacket::new(ip_packet.payload_mut()).unwrap();
        icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
        icmp_packet.set_icmp_code(pnet::packet::icmp::IcmpCode(0));
        icmp_packet.set_checksum(0);

        // Add payload
        let payload: Vec<u8> = (0..payload_size).map(|_| self.rng.gen()).collect();
        icmp_packet.set_payload(&payload);

        // Calculate and set ICMP checksum
        let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
        icmp_packet.set_checksum(checksum);

        // Set IP checksum last
        ip_packet.set_checksum(pnet::packet::ipv4::checksum(&ip_packet.to_immutable()));
        Ok(packet_buf)
    }

    fn build_ipv6_udp_packet(
        &mut self,
        target_ip: Ipv6Addr,
        target_port: u16,
    ) -> Result<Vec<u8>, String> {
        let payload_size = self.random_payload_size();
        let total_len = 40 + 8 + payload_size; // IPv6 + UDP + payload
        let mut packet_buf = vec![0u8; total_len];

        // Build IPv6 header
        let mut ip_packet = MutableIpv6Packet::new(&mut packet_buf).unwrap();
        ip_packet.set_version(6);
        ip_packet.set_traffic_class(0);
        ip_packet.set_flow_label(self.rng.gen::<u32>() & 0xFFFFF);
        ip_packet.set_payload_length((8 + payload_size) as u16);
        ip_packet.set_next_header(IpNextHeaderProtocols::Udp);
        ip_packet.set_hop_limit(self.rng.gen_range(32..128));
        ip_packet.set_source(self.source_ipv6);
        ip_packet.set_destination(target_ip);

        // Build UDP header + payload
        let mut udp_packet = MutableUdpPacket::new(ip_packet.payload_mut()).unwrap();
        udp_packet.set_source(self.rng.gen_range(1024..65535));
        udp_packet.set_destination(target_port);
        udp_packet.set_length((8 + payload_size) as u16);

        let payload: Vec<u8> = (0..payload_size).map(|_| self.rng.gen()).collect();
        udp_packet.set_payload(&payload);
        udp_packet.set_checksum(pnet::packet::udp::ipv6_checksum(
            &udp_packet.to_immutable(),
            &self.source_ipv6,
            &target_ip,
        ));

        Ok(packet_buf)
    }

    fn build_ipv6_tcp_packet(
        &mut self,
        target_ip: Ipv6Addr,
        target_port: u16,
    ) -> Result<Vec<u8>, String> {
        let total_len = 40 + 20; // IPv6 + TCP
        let mut packet_buf = vec![0u8; total_len];

        // Build IPv6 header
        let mut ip_packet = MutableIpv6Packet::new(&mut packet_buf).unwrap();
        ip_packet.set_version(6);
        ip_packet.set_traffic_class(0);
        ip_packet.set_flow_label(self.rng.gen::<u32>() & 0xFFFFF);
        ip_packet.set_payload_length(20);
        ip_packet.set_next_header(IpNextHeaderProtocols::Tcp);
        ip_packet.set_hop_limit(self.rng.gen_range(32..128));
        ip_packet.set_source(self.source_ipv6);
        ip_packet.set_destination(target_ip);

        // Build TCP packet
        let mut tcp_packet = MutableTcpPacket::new(ip_packet.payload_mut()).unwrap();
        tcp_packet.set_source(self.rng.gen_range(1024..65535));
        tcp_packet.set_destination(target_port);
        tcp_packet.set_sequence(self.rng.gen());
        tcp_packet.set_acknowledgement(0);
        tcp_packet.set_data_offset(5);
        tcp_packet.set_flags(TcpFlags::SYN);
        tcp_packet.set_window(self.rng.gen_range(1024..65535));
        tcp_packet.set_urgent_ptr(0);
        tcp_packet.set_checksum(pnet::packet::tcp::ipv6_checksum(
            &tcp_packet.to_immutable(),
            &self.source_ipv6,
            &target_ip,
        ));

        Ok(packet_buf)
    }

    fn build_ipv6_icmp_packet(&mut self, target_ip: Ipv6Addr) -> Result<Vec<u8>, String> {
        let payload_size = self.rng.gen_range(8..=56);
        let total_len = 40 + 8 + payload_size; // IPv6 + ICMPv6 + payload
        let mut packet_buf = vec![0u8; total_len];

        // Build IPv6 header
        let mut ip_packet = MutableIpv6Packet::new(&mut packet_buf).unwrap();
        ip_packet.set_version(6);
        ip_packet.set_traffic_class(0);
        ip_packet.set_flow_label(self.rng.gen::<u32>() & 0xFFFFF);
        ip_packet.set_payload_length((8 + payload_size) as u16);
        ip_packet.set_next_header(IpNextHeaderProtocols::Icmpv6);
        ip_packet.set_hop_limit(self.rng.gen_range(32..128));
        ip_packet.set_source(self.source_ipv6);
        ip_packet.set_destination(target_ip);

        // Build ICMPv6 packet (simplified - using ICMP structure)
        let mut icmp_packet = MutableIcmpPacket::new(ip_packet.payload_mut()).unwrap();
        icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
        icmp_packet.set_icmp_code(pnet::packet::icmp::IcmpCode(0));
        icmp_packet.set_checksum(0);

        let payload: Vec<u8> = (0..payload_size).map(|_| self.rng.gen()).collect();
        icmp_packet.set_payload(&payload);

        // ICMPv6 checksum calculation would be more complex in real implementation
        let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
        icmp_packet.set_checksum(checksum);

        Ok(packet_buf)
    }

    fn build_arp_packet(&mut self, target_ip: Ipv4Addr) -> Result<Vec<u8>, String> {
        let total_len = 14 + 28; // Ethernet + ARP
        let mut packet_buf = vec![0u8; total_len];

        // Build Ethernet header
        let mut ethernet_packet = MutableEthernetPacket::new(&mut packet_buf).unwrap();
        ethernet_packet.set_destination(MacAddr::broadcast());
        ethernet_packet.set_source(self.source_mac);
        ethernet_packet.set_ethertype(EtherTypes::Arp);

        // Build ARP packet
        let mut arp_packet = MutableArpPacket::new(ethernet_packet.payload_mut()).unwrap();
        arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_packet.set_protocol_type(EtherTypes::Ipv4);
        arp_packet.set_hw_addr_len(6);
        arp_packet.set_proto_addr_len(4);
        arp_packet.set_operation(ArpOperations::Request);
        arp_packet.set_sender_hw_addr(self.source_mac);
        arp_packet.set_sender_proto_addr(self.source_ip);
        arp_packet.set_target_hw_addr(MacAddr::zero());
        arp_packet.set_target_proto_addr(target_ip);

        Ok(packet_buf)
    }

    fn setup_ip_header(
        &mut self,
        ip_packet: &mut MutableIpv4Packet,
        total_len: usize,
        protocol: pnet::packet::ip::IpNextHeaderProtocol,
        target_ip: Ipv4Addr,
    ) {
        ip_packet.set_version(4);
        ip_packet.set_header_length(5);
        ip_packet.set_total_length(total_len as u16);
        ip_packet.set_ttl(self.rng.gen_range(32..128));
        ip_packet.set_next_level_protocol(protocol);
        ip_packet.set_source(self.source_ip);
        ip_packet.set_destination(target_ip);
        ip_packet.set_identification(self.rng.gen());

        // Occasionally set fragmentation flags
        if self.rng.gen_bool(0.1) {
            ip_packet.set_flags(2); // Don't fragment
        }
    }
}
