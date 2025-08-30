//! Protocol handling utilities
//!
//! This module provides efficient utilities for working with protocol names
//! and conversions, optimized for performance in hot paths.

use crate::constants::protocols;
use crate::packet::PacketType;
use crate::stats::protocol_breakdown::ProtocolIndex;

/// Fast protocol name lookup using const functions
pub struct ProtocolUtils;

impl ProtocolUtils {
    /// Convert PacketType to protocol name (const function for performance)
    #[inline(always)]
    pub const fn packet_type_to_protocol_name(packet_type: PacketType) -> &'static str {
        match packet_type {
            PacketType::Udp => protocols::UDP,
            PacketType::TcpSyn | PacketType::TcpAck => protocols::TCP,
            PacketType::Icmp => protocols::ICMP,
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp => protocols::IPV6,
            PacketType::Arp => protocols::ARP,
        }
    }
    
    /// Convert PacketType to ProtocolIndex for efficient stats tracking
    #[inline(always)]
    pub const fn packet_type_to_protocol_index(packet_type: PacketType) -> ProtocolIndex {
        match packet_type {
            PacketType::Udp => ProtocolIndex::Udp,
            PacketType::TcpSyn | PacketType::TcpAck => ProtocolIndex::Tcp,
            PacketType::Icmp => ProtocolIndex::Icmp,
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp => ProtocolIndex::Ipv6,
            PacketType::Arp => ProtocolIndex::Arp,
        }
    }
    
    /// Check if protocol name is valid
    #[inline(always)]
    pub fn is_valid_protocol(protocol: &str) -> bool {
        matches!(protocol, 
            protocols::UDP | protocols::TCP | protocols::ICMP | 
            protocols::IPV6 | protocols::ARP
        )
    }
    
    /// Get all supported protocol names
    #[inline(always)]
    pub const fn all_protocol_names() -> &'static [&'static str] {
        protocols::ALL_PROTOCOLS
    }
    
    /// Fast protocol name comparison (avoids string allocation)
    #[inline(always)]
    pub fn protocol_equals(protocol: &str, expected: &'static str) -> bool {
        protocol == expected
    }
}

/// Extension trait for PacketType to add protocol utilities
pub trait PacketTypeExt {
    /// Get protocol name using optimized lookup
    fn protocol_name_fast(&self) -> &'static str;
    
    /// Get protocol index for efficient stats tracking
    fn protocol_index(&self) -> ProtocolIndex;
    
    /// Check if this packet type uses IPv4
    fn is_ipv4(&self) -> bool;
    
    /// Check if this packet type uses IPv6
    fn is_ipv6(&self) -> bool;
}

impl PacketTypeExt for PacketType {
    #[inline(always)]
    fn protocol_name_fast(&self) -> &'static str {
        ProtocolUtils::packet_type_to_protocol_name(*self)
    }
    
    #[inline(always)]
    fn protocol_index(&self) -> ProtocolIndex {
        ProtocolUtils::packet_type_to_protocol_index(*self)
    }
    
    #[inline(always)]
    fn is_ipv4(&self) -> bool {
        matches!(self, 
            PacketType::Udp | PacketType::TcpSyn | PacketType::TcpAck | 
            PacketType::Icmp | PacketType::Arp
        )
    }
    
    #[inline(always)]
    fn is_ipv6(&self) -> bool {
        matches!(self, 
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp
        )
    }
}

/// Protocol validation function
pub fn validate_protocol(protocol: &str) -> bool {
    ProtocolUtils::is_valid_protocol(protocol)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_packet_type_to_protocol_name() {
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::Udp), "UDP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::TcpSyn), "TCP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::TcpAck), "TCP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::Icmp), "ICMP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::Ipv6Udp), "IPv6");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::Arp), "ARP");
    }
    
    #[test]
    fn test_packet_type_to_protocol_index() {
        assert_eq!(ProtocolUtils::packet_type_to_protocol_index(PacketType::Udp), ProtocolIndex::Udp);
        assert_eq!(ProtocolUtils::packet_type_to_protocol_index(PacketType::TcpSyn), ProtocolIndex::Tcp);
        assert_eq!(ProtocolUtils::packet_type_to_protocol_index(PacketType::Ipv6Tcp), ProtocolIndex::Ipv6);
    }
    
    #[test]
    fn test_protocol_validation() {
        assert!(ProtocolUtils::is_valid_protocol("UDP"));
        assert!(ProtocolUtils::is_valid_protocol("TCP"));
        assert!(!ProtocolUtils::is_valid_protocol("INVALID"));
    }
    
    #[test]
    fn test_packet_type_ext() {
        assert_eq!(PacketType::Udp.protocol_name_fast(), "UDP");
        assert_eq!(PacketType::Udp.protocol_index(), ProtocolIndex::Udp);
        assert!(PacketType::Udp.is_ipv4());
        assert!(!PacketType::Udp.is_ipv6());
        
        assert!(PacketType::Ipv6Udp.is_ipv6());
        assert!(!PacketType::Ipv6Udp.is_ipv4());
    }
    
    #[test]
    fn test_protocol_validation_function() {
        assert!(validate_protocol("UDP"));
        assert!(validate_protocol("TCP"));
        assert!(!validate_protocol("INVALID"));
    }
}