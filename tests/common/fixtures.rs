//! Test data generators and fixtures

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Generate a list of valid private IPv4 addresses for testing
pub fn private_ipv4_addresses() -> Vec<IpAddr> {
    vec![
        // 192.168.0.0/16 range
        IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 255, 254)),
        
        // 10.0.0.0/8 range
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3)),
        IpAddr::V4(Ipv4Addr::new(10, 255, 255, 254)),
        
        // 172.16.0.0/12 range
        IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(172, 20, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(172, 31, 255, 254)),
    ]
}

/// Generate a list of public IPv4 addresses that should be rejected
pub fn public_ipv4_addresses() -> Vec<IpAddr> {
    vec![
        // Public IPs
        IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
        IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(208, 67, 222, 222)),
        
        // Edge cases outside private ranges
        IpAddr::V4(Ipv4Addr::new(172, 15, 255, 255)), // Just before 172.16.0.0
        IpAddr::V4(Ipv4Addr::new(172, 32, 0, 0)),     // Just after 172.31.255.255
        IpAddr::V4(Ipv4Addr::new(192, 167, 255, 255)), // Just before 192.168.0.0
        IpAddr::V4(Ipv4Addr::new(192, 169, 0, 0)),     // Just after 192.168.255.255
        IpAddr::V4(Ipv4Addr::new(9, 255, 255, 255)),   // Just before 10.0.0.0
        IpAddr::V4(Ipv4Addr::new(11, 0, 0, 0)),         // Just after 10.255.255.255
    ]
}

/// Generate special IPv4 addresses that should be rejected
pub fn special_ipv4_addresses() -> Vec<IpAddr> {
    vec![
        // Loopback
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(127, 255, 255, 255)),
        
        // Broadcast
        IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
        
        // Multicast
        IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(239, 255, 255, 255)),
        
        // Reserved
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        IpAddr::V4(Ipv4Addr::new(240, 0, 0, 1)),
    ]
}

/// Generate valid private IPv6 addresses for testing
pub fn private_ipv6_addresses() -> Vec<IpAddr> {
    vec![
        // Link-local (fe80::/10)
        IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
        IpAddr::V6(Ipv6Addr::new(0xfe80, 0x1234, 0x5678, 0, 0, 0, 0, 1)),
        IpAddr::V6(Ipv6Addr::new(0xfebf, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xfffe)),
        
        // Unique local (fc00::/7)
        IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1)),
        IpAddr::V6(Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 1)),
        IpAddr::V6(Ipv6Addr::new(0xfdff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xfffe)),
    ]
}

/// Generate public IPv6 addresses that should be rejected
pub fn public_ipv6_addresses() -> Vec<IpAddr> {
    vec![
        // Public IPv6
        IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)), // Google DNS
        IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)), // Cloudflare DNS
        IpAddr::V6(Ipv6Addr::new(0x2a00, 0x1450, 0x4001, 0x803, 0, 0, 0, 0x200e)),
    ]
}

/// Generate a list of common ports for testing
pub fn common_ports() -> Vec<u16> {
    vec![
        22,    // SSH
        80,    // HTTP
        443,   // HTTPS
        8080,  // HTTP alternate
        3306,  // MySQL
        5432,  // PostgreSQL
        6379,  // Redis
        27017, // MongoDB
    ]
}

/// Generate well-known ports that should trigger warnings
pub fn well_known_ports() -> Vec<u16> {
    vec![
        22,   // SSH
        23,   // Telnet
        25,   // SMTP
        53,   // DNS
        80,   // HTTP
        110,  // POP3
        143,  // IMAP
        443,  // HTTPS
        445,  // SMB
        3389, // RDP
    ]
}

/// Generate test payload data of specified size
pub fn generate_payload(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Generate random test data
pub fn random_bytes(size: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; size];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Test thread counts
pub fn thread_counts() -> Vec<u32> {
    vec![1, 2, 4, 8, 16, 32, 64, 100]
}

/// Test packet rates
pub fn packet_rates() -> Vec<u32> {
    vec![1, 10, 100, 1000, 5000, 10000]
}

/// Test payload sizes
pub fn payload_sizes() -> Vec<usize> {
    vec![20, 64, 128, 256, 512, 1024, 1400]
}