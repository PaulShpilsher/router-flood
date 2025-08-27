//! Main packet builder implementation using strategy pattern

use super::{PacketStrategy, PacketType, Target};
use crate::config::ProtocolMix;
use crate::error::{PacketError, Result};
use crate::rng::BatchedRng;
use std::collections::HashMap;
use std::net::IpAddr;

/// Main packet builder that coordinates different packet strategies
pub struct PacketBuilder {
    strategies: HashMap<PacketType, Box<dyn PacketStrategy>>,
    protocol_selector: ProtocolSelector,
    rng: BatchedRng,
}

impl PacketBuilder {
    /// Create a new packet builder with the given configuration
    pub fn new(packet_size_range: (usize, usize), protocol_mix: ProtocolMix) -> Self {
        let mut strategies: HashMap<PacketType, Box<dyn PacketStrategy>> = HashMap::new();
        let mut rng = BatchedRng::new();
        
        // Initialize strategies for each packet type
        strategies.insert(
            PacketType::Udp,
            Box::new(super::strategies::UdpStrategy::new(packet_size_range, &mut rng)),
        );
        strategies.insert(
            PacketType::TcpSyn,
            Box::new(super::strategies::TcpStrategy::new_syn(&mut rng)),
        );
        strategies.insert(
            PacketType::TcpAck,
            Box::new(super::strategies::TcpStrategy::new_ack(&mut rng)),
        );
        strategies.insert(
            PacketType::Icmp,
            Box::new(super::strategies::IcmpStrategy::new(&mut rng)),
        );
        strategies.insert(
            PacketType::Ipv6Udp,
            Box::new(super::strategies::Ipv6UdpStrategy::new(packet_size_range, &mut rng)),
        );
        strategies.insert(
            PacketType::Ipv6Tcp,
            Box::new(super::strategies::Ipv6TcpStrategy::new(&mut rng)),
        );
        strategies.insert(
            PacketType::Ipv6Icmp,
            Box::new(super::strategies::Ipv6IcmpStrategy::new(&mut rng)),
        );
        strategies.insert(
            PacketType::Arp,
            Box::new(super::strategies::ArpStrategy::new(&mut rng)),
        );

        Self {
            strategies,
            protocol_selector: ProtocolSelector::new(protocol_mix),
            rng,
        }
    }

    /// Build a packet using the zero-copy approach
    #[inline]
    pub fn build_packet_into_buffer(
        &mut self,
        buffer: &mut [u8],
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
    ) -> Result<(usize, &'static str)> {
        let target = Target::new(target_ip, target_port);
        
        let strategy = self.strategies.get_mut(&packet_type)
            .ok_or_else(|| PacketError::InvalidParameters(
                format!("No strategy available for packet type: {}", packet_type)
            ))?;

        if !strategy.is_compatible_with(target_ip) {
            return Err(PacketError::InvalidParameters(
                format!("Packet type {} is not compatible with target IP {}", packet_type, target_ip)
            ).into());
        }

        let packet_size = strategy.build_packet(&target, buffer)?;
        let protocol_name = strategy.protocol_name();
        
        Ok((packet_size, protocol_name))
    }

    /// Build a packet with allocation (fallback method)
    #[inline]
    pub fn build_packet(
        &mut self,
        packet_type: PacketType,
        target_ip: IpAddr,
        target_port: u16,
    ) -> Result<(Vec<u8>, &'static str)> {
        let strategy = self.strategies.get(&packet_type)
            .ok_or_else(|| PacketError::InvalidParameters(
                format!("No strategy available for packet type: {}", packet_type)
            ))?;

        let max_size = strategy.max_packet_size();
        let mut buffer = vec![0u8; max_size];
        
        let (actual_size, protocol_name) = self.build_packet_into_buffer(
            &mut buffer, packet_type, target_ip, target_port
        )?;
        
        buffer.truncate(actual_size);
        Ok((buffer, protocol_name))
    }

    /// Select the next packet type based on protocol mix and target IP compatibility
    #[inline]
    pub fn next_packet_type_for_ip(&mut self, target_ip: IpAddr) -> PacketType {
        self.protocol_selector.select_packet_type(target_ip, &mut self.rng)
    }

    /// Generate a random boolean with the given probability
    #[inline(always)]
    pub fn rng_gen_bool(&mut self, probability: f64) -> bool {
        self.rng.bool_with_probability(probability)
    }

    /// Generate a random float in the given range
    #[inline(always)]
    pub fn rng_gen_range(&mut self, range: std::ops::Range<f64>) -> f64 {
        self.rng.float_range(range.start, range.end)
    }
}

/// Protocol selector that chooses packet types based on configured ratios
struct ProtocolSelector {
    protocol_mix: ProtocolMix,
}

impl ProtocolSelector {
    fn new(protocol_mix: ProtocolMix) -> Self {
        Self { protocol_mix }
    }

    fn select_packet_type(&self, target_ip: IpAddr, rng: &mut BatchedRng) -> PacketType {
        let rand_val = rng.float_range(0.0, 1.0);
        let mut cumulative = 0.0;

        match target_ip {
            IpAddr::V4(_) => {
                // IPv4 protocols only
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

                // ARP for IPv4 (fallback)
                PacketType::Arp
            }
            IpAddr::V6(_) => {
                // IPv6 protocols only - evenly distribute across IPv6 types
                let norm_val = rand_val * 3.0; // 3 IPv6 packet types
                
                if norm_val < 1.0 {
                    PacketType::Ipv6Udp
                } else if norm_val < 2.0 {
                    PacketType::Ipv6Tcp
                } else {
                    PacketType::Ipv6Icmp
                }
            }
        }
    }
}