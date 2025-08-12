use crate::config::{MAX_PACKET_RATE, MAX_THREADS, PRIVATE_RANGES};
use std::net::IpAddr;
use tracing::{error, info, warn};

/// Enhanced safety validation functions
pub fn validate_target_ip(ip: &IpAddr) -> Result<(), String> {
    match ip {
        IpAddr::V4(ipv4) => {
            let ip_u32 = u32::from(*ipv4);

            // Check against defined private ranges using bitwise operations
            let is_private = PRIVATE_RANGES.iter().any(|(network, mask)| {
                (ip_u32 & mask) == *network
            });

            if is_private {
                info!("Target IP {} validated as private range", ip);
                Ok(())
            } else {
                let error_msg = format!("Target IP {} is not in private range. This tool should only target local networks.", ip);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
        IpAddr::V6(ipv6) => {
            // Check for IPv6 private ranges (link-local, unique local)
            if ipv6.is_loopback() {
                return Err("Cannot target IPv6 loopback address".to_string());
            }

            // Link-local (fe80::/10) or unique local (fc00::/7)
            let segments = ipv6.segments();
            if (segments[0] & 0xffc0) == 0xfe80 || (segments[0] & 0xfe00) == 0xfc00 {
                info!("Target IPv6 {} validated as private range", ip);
                Ok(())
            } else {
                let error_msg = format!("Target IPv6 {} is not in private range", ip);
                error!("{}", error_msg);
                Err(error_msg)
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
) -> Result<(), String> {
    // Check if targeting loopback or multicast
    if is_loopback_or_multicast(ip) {
        return Err("Cannot target loopback, multicast, or broadcast addresses".to_string());
    }

    // Validate private IP
    validate_target_ip(ip)?;

    // Check thread limits
    if threads > MAX_THREADS {
        return Err(format!(
            "Thread count {} exceeds maximum: {}",
            threads, MAX_THREADS
        ));
    }

    // Check rate limits
    if rate > MAX_PACKET_RATE {
        return Err(format!(
            "Packet rate {} exceeds maximum: {}",
            rate, MAX_PACKET_RATE
        ));
    }

    // Check for common service ports that shouldn't be flooded
    for &port in ports {
        match port {
            22 => warn!("Targeting SSH port {} - ensure this is intentional", port),
            53 => warn!("Targeting DNS port {} - ensure this is intentional", port),
            443 => warn!("Targeting HTTPS port {} - ensure this is intentional", port),
            _ => {}
        }
    }

    Ok(())
}

pub fn validate_system_requirements(dry_run: bool) -> Result<(), String> {
    // Check if running as root (required for raw sockets, but not for dry-run)
    if !dry_run && unsafe { libc::geteuid() } != 0 {
        return Err(
            "This program requires root privileges for raw socket access. Use --dry-run for testing without root."
                .to_string(),
        );
    }

    if dry_run {
        info!("Dry-run mode: Skipping root privilege check");
    }

    // Check system limits
    let max_files = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    if max_files < 1024 {
        warn!("Low file descriptor limit detected: {}", max_files);
    }

    Ok(())
}
