//! Performance-optimized constants and compile-time computations

/// Compile-time computed constants for better performance
pub mod computed {
    /// Pre-computed protocol ratios for faster selection
    pub const PROTOCOL_SELECTION_THRESHOLD_COUNT: usize = 1000;
    
    /// Pre-computed random value batches
    pub const OPTIMAL_BATCH_SIZE: usize = 2048; // Power of 2 for better cache alignment
    
    /// Buffer alignment for SIMD operations
    pub const BUFFER_ALIGNMENT: usize = 64; // Cache line size
    
    /// Optimal worker count based on CPU cores
    pub const fn optimal_worker_count() -> usize {
        // This would ideally use std::thread::available_parallelism() at runtime
        // For now, we use a reasonable default
        8
    }
    
    /// Fast modulo for power-of-2 values
    #[inline(always)]
    pub const fn fast_modulo_pow2(value: usize, modulus_pow2: usize) -> usize {
        value & (modulus_pow2 - 1)
    }
}

/// Performance tuning constants
pub mod tuning {
    /// Threshold for switching between allocation strategies
    pub const LARGE_PACKET_THRESHOLD: usize = 1400;
    
    /// Batch size for atomic operations
    pub const ATOMIC_BATCH_SIZE: usize = 64;
    
    /// Cache-friendly data structure sizes
    pub const CACHE_LINE_SIZE: usize = 64;
    pub const L1_CACHE_SIZE: usize = 32 * 1024;
    pub const L2_CACHE_SIZE: usize = 256 * 1024;
    
    /// Network performance constants
    pub const JUMBO_FRAME_SIZE: usize = 9000;
    pub const STANDARD_MTU: usize = 1500;
    pub const MIN_ETHERNET_FRAME: usize = 64;
}

/// Compile-time protocol mix calculations
pub mod protocol_mix {
    use crate::config::ProtocolMix;
    
    /// Pre-computed cumulative distribution for protocol selection
    pub struct CumulativeDistribution {
        pub udp_threshold: f64,
        pub tcp_syn_threshold: f64,
        pub tcp_ack_threshold: f64,
        pub icmp_threshold: f64,
        pub ipv6_threshold: f64,
        // ARP is the remainder (1.0)
    }
    
    impl CumulativeDistribution {
        #[inline]
        pub const fn from_mix(mix: &ProtocolMix) -> Self {
            Self {
                udp_threshold: mix.udp_ratio,
                tcp_syn_threshold: mix.udp_ratio + mix.tcp_syn_ratio,
                tcp_ack_threshold: mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio,
                icmp_threshold: mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + mix.icmp_ratio,
                ipv6_threshold: mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + mix.icmp_ratio + mix.ipv6_ratio,
            }
        }
        
        /// Fast protocol selection using pre-computed thresholds
        #[inline(always)]
        pub fn select_protocol(&self, random_value: f64) -> crate::packet::PacketType {
            use crate::packet::PacketType;
            
            if random_value < self.udp_threshold {
                PacketType::Udp
            } else if random_value < self.tcp_syn_threshold {
                PacketType::TcpSyn
            } else if random_value < self.tcp_ack_threshold {
                PacketType::TcpAck
            } else if random_value < self.icmp_threshold {
                PacketType::Icmp
            } else if random_value < self.ipv6_threshold {
                // For IPv6, randomly select sub-type
                match ((random_value * 3.0) as usize) % 3 {
                    0 => PacketType::Ipv6Udp,
                    1 => PacketType::Ipv6Tcp,
                    _ => PacketType::Ipv6Icmp,
                }
            } else {
                PacketType::Arp
            }
        }
    }
}