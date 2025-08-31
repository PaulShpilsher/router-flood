use std::net::IpAddr;
use tracing::{info, warn};

use crate::constants::{
    MAX_PACKET_RATE, MAX_THREADS, PRIVATE_IPV4_RANGES,
    IPV6_LINK_LOCAL_PREFIX, IPV6_LINK_LOCAL_MASK,
    IPV6_UNIQUE_LOCAL_PREFIX, IPV6_UNIQUE_LOCAL_MASK,
    validation::ROOT_UID, MIN_FILE_DESCRIPTORS,
    error_messages, WELL_KNOWN_PORTS,
};
use crate::error::{ValidationError, Result};

/// Enhanced safety validation functions
pub fn validate_target_ip(ip: &IpAddr) -> Result<()> {
    match ip {
        IpAddr::V4(ipv4) => {
            let ip_u32 = u32::from(*ipv4);

            // Check against defined private ranges using bitwise operations
            let is_private = PRIVATE_IPV4_RANGES.iter().any(|(network, mask)| {
                (ip_u32 & mask) == *network
            });

            if is_private {
                info!("Target IP {} validated as private range", ip);
                Ok(())
            } else {
                Err(ValidationError::new("ip", "Invalid IP range").into())
            }
        }
        IpAddr::V6(ipv6) => {
            // Check for IPv6 private ranges (link-local, unique local)
            if ipv6.is_loopback() {
                return Err(ValidationError::new("ip", "Invalid IP range").into());
            }

            // Link-local (fe80::/10) or unique local (fc00::/7)
            let segments = ipv6.segments();
            let is_private = (segments[0] & IPV6_LINK_LOCAL_MASK) == IPV6_LINK_LOCAL_PREFIX
                || (segments[0] & IPV6_UNIQUE_LOCAL_MASK) == IPV6_UNIQUE_LOCAL_PREFIX;

            if is_private {
                info!("Target IPv6 {} validated as private range", ip);
                Ok(())
            } else {
                Err(ValidationError::new("ip", "Invalid IP range").into())
            }
        }
    }
}

pub fn is_loopback_or_multicast(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_multicast() || ipv4.is_broadcast(),
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_multicast(),
    }
}

pub fn validate_comprehensive_security(
    ip: &IpAddr,
    ports: &[u16],
    threads: usize,
    rate: u64,
) -> Result<()> {
    // Check if targeting loopback or multicast
    if is_loopback_or_multicast(ip) {
        return Err(ValidationError::new("ip", "Invalid IP range").into());
    }

    // Validate private IP
    validate_target_ip(ip)?;

    // Check thread limits
    if threads > MAX_THREADS {
        return Err(ValidationError::new("limit", "Value exceeds limit").into());
    }

    // Check rate limits
    if rate > MAX_PACKET_RATE {
        return Err(ValidationError::new("limit", "Value exceeds limit").into());
    }

    // Check for common service ports that shouldn't be flooded
    for &port in ports {
        if WELL_KNOWN_PORTS.contains(&port) {
            warn!("Targeting well-known service port {} - ensure this is intentional", port);
        }
    }

    Ok(())
}

pub fn validate_system_requirements(dry_run: bool) -> Result<()> {
    // Check if running as root (required for raw sockets, but not for dry-run)
    if !dry_run && unsafe { libc::geteuid() } != ROOT_UID {
        return Err(ValidationError::new(
            "privileges",
            error_messages::ROOT_REQUIRED
        ).into());
    }

    if dry_run {
        info!("Dry-run mode: Skipping root privilege check");
    }

    // Check system limits
    let max_files = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    if max_files < MIN_FILE_DESCRIPTORS {
        warn!("Low file descriptor limit detected: {} (recommended: {})", 
            max_files, MIN_FILE_DESCRIPTORS);
    }

    Ok(())
}
