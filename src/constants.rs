//! Application constants and configuration values
//!
//! This module centralizes all magic numbers, buffer sizes, limits,
//! and other constants used throughout the application.

use std::time::Duration;

// Configuration file constants
pub const DEFAULT_CONFIG_FILE: &str = "router_flood_config.yaml";
pub const STATS_EXPORT_DIR: &str = "exports";

// System limits
pub const MAX_THREADS: usize = 100;
pub const MAX_PACKET_RATE: u64 = 10000;
pub const MIN_FILE_DESCRIPTORS: i64 = 1024;

// Packet size constraints
pub const MIN_PAYLOAD_SIZE: usize = 20;
pub const MAX_PAYLOAD_SIZE: usize = 1400;
pub const ETHERNET_HEADER_SIZE: usize = 14;
pub const IPV4_HEADER_SIZE: usize = 20;
pub const IPV6_HEADER_SIZE: usize = 40;
pub const TCP_HEADER_SIZE: usize = 20;
pub const UDP_HEADER_SIZE: usize = 8;
pub const ICMP_HEADER_SIZE: usize = 8;
pub const ARP_PACKET_SIZE: usize = 28;
pub const ARP_FRAME_SIZE: usize = ETHERNET_HEADER_SIZE + ARP_PACKET_SIZE;

// Common frame size constants
pub const IPV4_TCP_HEADER_SIZE: usize = IPV4_HEADER_SIZE + TCP_HEADER_SIZE;
pub const IPV4_UDP_HEADER_SIZE: usize = IPV4_HEADER_SIZE + UDP_HEADER_SIZE;
pub const IPV4_ICMP_HEADER_SIZE: usize = IPV4_HEADER_SIZE + ICMP_HEADER_SIZE;
pub const IPV6_TCP_HEADER_SIZE: usize = IPV6_HEADER_SIZE + TCP_HEADER_SIZE;
pub const IPV6_UDP_HEADER_SIZE: usize = IPV6_HEADER_SIZE + UDP_HEADER_SIZE;
pub const IPV6_ICMP_HEADER_SIZE: usize = IPV6_HEADER_SIZE + ICMP_HEADER_SIZE;

// Network buffer sizes
pub const TRANSPORT_BUFFER_SIZE: usize = 4096;
pub const SMALL_PACKET_THRESHOLD: usize = 200;
pub const MEDIUM_PACKET_THRESHOLD: usize = 800;

// Protocol distribution weights (for random packet type selection)
pub const SMALL_PACKET_PROBABILITY: u8 = 40;
pub const MEDIUM_PACKET_PROBABILITY: u8 = 40;
pub const LARGE_PACKET_PROBABILITY: u8 = 20;

// Timing constants
pub const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
pub const DEFAULT_STATS_INTERVAL: u64 = 5;
pub const DEFAULT_EXPORT_INTERVAL: u64 = 60;
pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(100);

// Network ranges and limits
pub const PRIVATE_IPV4_RANGES: &[(u32, u32)] = &[
    (0xC0A80000, 0xFFFF0000), // 192.168.0.0/16
    (0x0A000000, 0xFF000000), // 10.0.0.0/8
    (0xAC100000, 0xFFF00000), // 172.16.0.0/12
];

// IPv6 private range prefixes
pub const IPV6_LINK_LOCAL_PREFIX: u16 = 0xfe80;
pub const IPV6_LINK_LOCAL_MASK: u16 = 0xffc0;
pub const IPV6_UNIQUE_LOCAL_PREFIX: u16 = 0xfc00;
pub const IPV6_UNIQUE_LOCAL_MASK: u16 = 0xfe00;

// Port ranges
pub const EPHEMERAL_PORT_MIN: u16 = 1024;
pub const EPHEMERAL_PORT_MAX: u16 = 65535;
pub const WELL_KNOWN_PORTS: &[u16] = &[22, 53, 80, 443];

// Protocol names (centralized for consistency)
pub mod protocols {
    pub const UDP: &str = "UDP";
    pub const TCP: &str = "TCP";
    pub const ICMP: &str = "ICMP";
    pub const IPV6: &str = "IPv6";
    pub const ARP: &str = "ARP";
    
    pub const ALL_PROTOCOLS: &[&str] = &[UDP, TCP, ICMP, IPV6, ARP];
}

// Default configuration values
pub mod defaults {
    use super::*;
    
    pub const TARGET_IP: &str = "192.168.0.1";
    pub const TARGET_PORT: u16 = 80;
    pub const DEFAULT_THREADS: usize = 4;
    pub const THREAD_COUNT: usize = 4;
    pub const DEFAULT_PACKET_RATE: f64 = 100.0;
    pub const PACKET_RATE: u64 = 100;
    pub const DEFAULT_PAYLOAD_SIZE: usize = 64;
    pub const DEFAULT_DURATION_SECONDS: u64 = 60;
    pub const DEFAULT_MAX_BANDWIDTH_MBPS: f64 = 100.0;
    pub const DEFAULT_STATS_INTERVAL_MS: u64 = 1000;
    pub const STATS_INTERVAL: u64 = DEFAULT_STATS_INTERVAL;
    
    // Protocol mix ratios (must sum to 1.0)
    pub const UDP_RATIO: f64 = 0.6;
    pub const TCP_SYN_RATIO: f64 = 0.25;
    pub const TCP_ACK_RATIO: f64 = 0.05;
    pub const ICMP_RATIO: f64 = 0.05;
    pub const IPV6_RATIO: f64 = 0.03;
    pub const ARP_RATIO: f64 = 0.02;
    
    pub const FILENAME_PATTERN: &str = "router_flood";
}

// Timing jitter ranges
pub mod timing {
    pub const JITTER_MIN: f64 = 0.8;
    pub const JITTER_MAX: f64 = 1.2;
    pub const TTL_MIN: u8 = 32;
    pub const TTL_MAX: u8 = 128;
    pub const HOP_LIMIT_MIN: u8 = 32;
    pub const HOP_LIMIT_MAX: u8 = 128;
}

// Statistics and monitoring
pub mod stats {
    pub const SUCCESS_RATE_SIMULATION: f64 = 0.98; // 98% success rate in dry-run
    pub const LOG_FREQUENCY: u64 = 1000; // Log every 1000 packets
    pub const MEGABITS_DIVISOR: f64 = 1_000_000.0;
    pub const BYTES_TO_MB_DIVISOR: u64 = 1024 * 1024;
}

// System validation constants
pub mod validation {
    pub const ROOT_UID: u32 = 0;
    pub const FRAGMENTATION_PROBABILITY: f64 = 0.1; // 10% chance of DF flag
}

// Source address generation
pub mod source_generation {
    pub const IPV4_PRIVATE_BASE: [u8; 4] = [192, 168, 1, 0];
    pub const IPV4_HOST_MIN: u8 = 2;
    pub const IPV4_HOST_MAX: u8 = 254;
    
    pub const IPV6_LINK_LOCAL_BASE: [u16; 8] = [0xfe80, 0, 0, 0, 0, 0, 0, 0];
    pub const MAC_LOCALLY_ADMINISTERED: u8 = 0x02; // Locally administered MAC prefix
}

// ICMP constants
pub mod icmp {
    pub const MIN_PING_SIZE: usize = 8;
    pub const MAX_PING_SIZE: usize = 56;
    pub const DEFAULT_PING_SIZE: usize = 32;
}

// Error message constants
pub mod error_messages {
    pub const ROOT_REQUIRED: &str = "This program requires root privileges for raw socket access. Use --dry-run for testing without root.";
    pub const INTERFACE_NOT_FOUND: &str = "Network interface not found";
    pub const INVALID_IP_FORMAT: &str = "Invalid IP address format";
    pub const PRIVATE_RANGE_REQUIRED: &str = "Target IP must be in private range for safety";
    pub const LOOPBACK_PROHIBITED: &str = "Cannot target loopback, multicast, or broadcast addresses";
}