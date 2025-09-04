//! Packet type definitions and utilities

use std::fmt;

/// Supported packet types for network simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketType {
    Udp,
    TcpSyn,
    TcpAck,
    TcpFin,
    TcpRst,
    Icmp,
    Ipv6Udp,
    Ipv6Tcp,
    Ipv6Icmp,
    Arp,
}

impl PacketType {
    /// Get all available packet types
    pub const fn all() -> &'static [PacketType] {
        &[
            PacketType::Udp,
            PacketType::TcpSyn,
            PacketType::TcpAck,
            PacketType::TcpFin,
            PacketType::TcpRst,
            PacketType::Icmp,
            PacketType::Ipv6Udp,
            PacketType::Ipv6Tcp,
            PacketType::Ipv6Icmp,
            PacketType::Arp,
        ]
    }
    
    /// Check if this packet type is IPv6-based
    pub const fn is_ipv6(&self) -> bool {
        matches!(self, PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp)
    }
    
    /// Check if this packet type is IPv4-based
    pub const fn is_ipv4(&self) -> bool {
        matches!(self, PacketType::Udp | PacketType::TcpSyn | PacketType::TcpAck | PacketType::TcpFin | PacketType::TcpRst | PacketType::Icmp | PacketType::Arp)
    }
    
    /// Get the protocol name for statistics
    pub const fn protocol_name(&self) -> &'static str {
        match self {
            PacketType::Udp => "UDP",
            PacketType::TcpSyn | PacketType::TcpAck | PacketType::TcpFin | PacketType::TcpRst => "TCP",
            PacketType::Icmp => "ICMP",
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp => "IPv6",
            PacketType::Arp => "ARP",
        }
    }
}

impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PacketType::Udp => "UDP",
            PacketType::TcpSyn => "TCP-SYN",
            PacketType::TcpAck => "TCP-ACK",
            PacketType::TcpFin => "TCP-FIN",
            PacketType::TcpRst => "TCP-RST",
            PacketType::Icmp => "ICMP",
            PacketType::Ipv6Udp => "IPv6-UDP",
            PacketType::Ipv6Tcp => "IPv6-TCP",
            PacketType::Ipv6Icmp => "IPv6-ICMP",
            PacketType::Arp => "ARP",
        };
        write!(f, "{}", name)
    }
}