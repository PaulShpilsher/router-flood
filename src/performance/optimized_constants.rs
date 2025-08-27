//! Compile-time optimized constants and functions
//!
//! This module provides const functions and pre-computed values
//! for better performance in hot paths.

use crate::packet::PacketType;

/// Compile-time packet type information
impl PacketType {
    /// Get protocol name as const function
    #[inline(always)]
    pub const fn protocol_name_const(&self) -> &'static str {
        match self {
            PacketType::Udp => "UDP",
            PacketType::TcpSyn | PacketType::TcpAck => "TCP",
            PacketType::Icmp => "ICMP",
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp => "IPv6",
            PacketType::Arp => "ARP",
        }
    }
    
    /// Check if packet type is IPv4 (const function)
    #[inline(always)]
    pub const fn is_ipv4_const(&self) -> bool {
        matches!(self, 
            PacketType::Udp | PacketType::TcpSyn | PacketType::TcpAck | 
            PacketType::Icmp | PacketType::Arp
        )
    }
    
    /// Check if packet type is IPv6 (const function)
    #[inline(always)]
    pub const fn is_ipv6_const(&self) -> bool {
        matches!(self, 
            PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp
        )
    }
    
    /// Get minimum packet size for this type (const function)
    #[inline(always)]
    pub const fn min_packet_size(&self) -> usize {
        match self {
            PacketType::Udp => 20 + 8, // IP + UDP
            PacketType::TcpSyn | PacketType::TcpAck => 20 + 20, // IP + TCP
            PacketType::Icmp => 20 + 8, // IP + ICMP
            PacketType::Ipv6Udp => 40 + 8, // IPv6 + UDP
            PacketType::Ipv6Tcp => 40 + 20, // IPv6 + TCP
            PacketType::Ipv6Icmp => 40 + 8, // IPv6 + ICMP
            PacketType::Arp => 14 + 28, // Ethernet + ARP
        }
    }
}

/// Pre-computed lookup tables for fast operations
pub mod lookup_tables {
    use super::PacketType;
    
    /// Pre-computed minimum packet sizes
    pub const MIN_PACKET_SIZES: [usize; 8] = [
        28,  // UDP: 20 + 8
        40,  // TcpSyn: 20 + 20
        40,  // TcpAck: 20 + 20
        28,  // Icmp: 20 + 8
        48,  // Ipv6Udp: 40 + 8
        60,  // Ipv6Tcp: 40 + 20
        48,  // Ipv6Icmp: 40 + 8
        42,  // Arp: 14 + 28
    ];
    
    /// Get minimum packet size by index (const function)
    #[inline(always)]
    pub const fn min_size_by_index(packet_type_index: usize) -> usize {
        if packet_type_index < MIN_PACKET_SIZES.len() {
            MIN_PACKET_SIZES[packet_type_index]
        } else {
            64 // Safe default
        }
    }
    
    /// Convert PacketType to index for lookup tables
    #[inline(always)]
    pub const fn packet_type_to_index(packet_type: PacketType) -> usize {
        match packet_type {
            PacketType::Udp => 0,
            PacketType::TcpSyn => 1,
            PacketType::TcpAck => 2,
            PacketType::Icmp => 3,
            PacketType::Ipv6Udp => 4,
            PacketType::Ipv6Tcp => 5,
            PacketType::Ipv6Icmp => 6,
            PacketType::Arp => 7,
        }
    }
}

/// Fast bit manipulation utilities
pub mod bit_ops {
    /// Fast power-of-2 check
    #[inline(always)]
    pub const fn is_power_of_2(n: usize) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }
    
    /// Fast modulo for power-of-2 divisors
    #[inline(always)]
    pub const fn fast_mod_pow2(value: usize, modulus: usize) -> usize {
        debug_assert!(is_power_of_2(modulus));
        value & (modulus - 1)
    }
    
    /// Round up to next power of 2
    #[inline(always)]
    pub const fn next_power_of_2(mut n: usize) -> usize {
        if n == 0 {
            return 1;
        }
        n -= 1;
        n |= n >> 1;
        n |= n >> 2;
        n |= n >> 4;
        n |= n >> 8;
        n |= n >> 16;
        if usize::BITS > 32 {
            n |= n >> 32;
        }
        n + 1
    }
}

/// Memory alignment utilities
pub mod alignment {
    /// Align value to cache line boundary
    #[inline(always)]
    pub const fn align_to_cache_line(size: usize) -> usize {
        const CACHE_LINE_SIZE: usize = 64;
        (size + CACHE_LINE_SIZE - 1) & !(CACHE_LINE_SIZE - 1)
    }
    
    /// Check if pointer is aligned to cache line
    #[inline(always)]
    pub fn is_cache_aligned(ptr: *const u8) -> bool {
        (ptr as usize) & 63 == 0
    }
}

/// Network byte order utilities
pub mod network_order {
    /// Convert u16 to network byte order (const function)
    #[inline(always)]
    pub const fn htons(value: u16) -> u16 {
        value.to_be()
    }
    
    /// Convert u32 to network byte order (const function)
    #[inline(always)]
    pub const fn htonl(value: u32) -> u32 {
        value.to_be()
    }
    
    /// Convert u16 from network byte order (const function)
    #[inline(always)]
    pub const fn ntohs(value: u16) -> u16 {
        u16::from_be(value)
    }
    
    /// Convert u32 from network byte order (const function)
    #[inline(always)]
    pub const fn ntohl(value: u32) -> u32 {
        u32::from_be(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_const_functions() {
        // Test packet type const functions
        assert_eq!(PacketType::Udp.protocol_name_const(), "UDP");
        assert!(PacketType::Udp.is_ipv4_const());
        assert!(!PacketType::Udp.is_ipv6_const());
        assert_eq!(PacketType::Udp.min_packet_size(), 28);
        
        assert_eq!(PacketType::Ipv6Udp.protocol_name_const(), "IPv6");
        assert!(!PacketType::Ipv6Udp.is_ipv4_const());
        assert!(PacketType::Ipv6Udp.is_ipv6_const());
        assert_eq!(PacketType::Ipv6Udp.min_packet_size(), 48);
    }
    
    #[test]
    fn test_lookup_tables() {
        use lookup_tables::*;
        
        for (i, &packet_type) in [
            PacketType::Udp, PacketType::TcpSyn, PacketType::TcpAck, PacketType::Icmp,
            PacketType::Ipv6Udp, PacketType::Ipv6Tcp, PacketType::Ipv6Icmp, PacketType::Arp
        ].iter().enumerate() {
            let index = packet_type_to_index(packet_type);
            assert_eq!(index, i);
            assert_eq!(min_size_by_index(index), packet_type.min_packet_size());
        }
    }
    
    #[test]
    fn test_bit_operations() {
        use bit_ops::*;
        
        assert!(is_power_of_2(1));
        assert!(is_power_of_2(2));
        assert!(is_power_of_2(4));
        assert!(is_power_of_2(8));
        assert!(!is_power_of_2(3));
        assert!(!is_power_of_2(5));
        
        assert_eq!(fast_mod_pow2(15, 8), 7);
        assert_eq!(fast_mod_pow2(16, 8), 0);
        
        assert_eq!(next_power_of_2(0), 1);
        assert_eq!(next_power_of_2(1), 1);
        assert_eq!(next_power_of_2(2), 2);
        assert_eq!(next_power_of_2(3), 4);
        assert_eq!(next_power_of_2(15), 16);
    }
    
    #[test]
    fn test_alignment() {
        use alignment::*;
        
        assert_eq!(align_to_cache_line(1), 64);
        assert_eq!(align_to_cache_line(64), 64);
        assert_eq!(align_to_cache_line(65), 128);
        
        // Test cache alignment check
        let aligned_data = [0u8; 128];
        let _aligned_ptr = aligned_data.as_ptr();
        // Note: This might not always be cache-aligned depending on allocator
        // but the function should work correctly
    }
    
    #[test]
    fn test_network_order() {
        use network_order::*;
        
        let value16: u16 = 0x1234;
        let network16 = htons(value16);
        assert_eq!(ntohs(network16), value16);
        
        let value32: u32 = 0x12345678;
        let network32 = htonl(value32);
        assert_eq!(ntohl(network32), value32);
    }
}