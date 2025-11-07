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

/// Check if IP is a broadcast address
pub fn is_broadcast(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_broadcast(),
        IpAddr::V6(_) => false,  // IPv6 doesn't have broadcast
    }
}

/// Validate broadcast permission
pub fn validate_broadcast_permission(ip: &IpAddr, allow_broadcast: bool) -> Result<()> {
    if is_broadcast(ip) && !allow_broadcast {
        return Err(ValidationError::new(
            "broadcast",
            "Broadcast addresses are blocked by default. Use --allow-broadcast to enable."
        ).into());
    }

    if is_broadcast(ip) && allow_broadcast {
        warn!("⚠️  BROADCAST MODE ENABLED");
        warn!("⚠️  This will affect ALL devices on the network segment!");
        warn!("⚠️  Ensure you have explicit permission for broadcast testing!");
        info!("Broadcast target validated: {}", ip);
    }

    Ok(())
}

pub fn validate_comprehensive_security(
    ip: &IpAddr,
    ports: &[u16],
    threads: usize,
    rate: u64,
) -> Result<()> {
    // Check if targeting loopback or multicast (but not broadcast - handled separately)
    match ip {
        IpAddr::V4(ipv4) => {
            if ipv4.is_loopback() || ipv4.is_multicast() {
                return Err(ValidationError::new("ip", "Invalid IP range").into());
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() || ipv6.is_multicast() {
                return Err(ValidationError::new("ip", "Invalid IP range").into());
            }
        }
    }

    // Validate private IP (unless broadcast)
    if !is_broadcast(ip) {
        validate_target_ip(ip)?;
    }

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

/// Input validator with security focus (simplified from input_validation.rs)
pub struct InputValidation {
    config: ValidationConfig,
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_string_length: usize,
    pub strict_ip_validation: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_string_length: 1024,
            strict_ip_validation: true,
        }
    }
}

impl InputValidation {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }
    
    pub fn validate_ip(&self, ip: &IpAddr) -> Result<()> {
        if self.config.strict_ip_validation {
            validate_target_ip(ip)
        } else {
            Ok(())
        }
    }
}
