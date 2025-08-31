//! Unit tests for security validation functions

#[path = "../../common/mod.rs"]
mod common;

use common::*;
use router_flood::security::validation::{
    validate_target_ip, validate_system_requirements, validate_comprehensive_security,
};
use std::net::IpAddr;

#[cfg(test)]
mod validate_target_ip_tests {
    use super::*;

    #[test]
    fn test_accepts_private_ipv4_ranges() {
        for ip in private_ipv4_addresses() {
            let result = validate_target_ip(&ip);
            assert_ok(result);
        }
    }

    #[test]
    fn test_rejects_public_ipv4_addresses() {
        for ip in public_ipv4_addresses() {
            let result = validate_target_ip(&ip);
            assert_is_validation_error(result);
        }
    }

    #[test]
    fn test_rejects_special_ipv4_addresses() {
        for ip in special_ipv4_addresses() {
            let result = validate_target_ip(&ip);
            assert_is_validation_error(result);
        }
    }

    #[test]
    fn test_accepts_private_ipv6_ranges() {
        for ip in private_ipv6_addresses() {
            let result = validate_target_ip(&ip);
            assert_ok(result);
        }
    }

    #[test]
    fn test_rejects_public_ipv6_addresses() {
        for ip in public_ipv6_addresses() {
            let result = validate_target_ip(&ip);
            assert_is_validation_error(result);
        }
    }

    #[test]
    fn test_rejects_ipv6_loopback() {
        let loopback = "::1".parse::<IpAddr>().unwrap();
        let result = validate_target_ip(&loopback);
        assert_is_validation_error(result);
    }

    #[test]
    fn test_private_range_edge_cases() {
        // Test edge addresses of private ranges
        let edge_cases = vec![
            "192.168.0.0",   // Start of 192.168.0.0/16
            "192.168.255.255", // End of 192.168.0.0/16
            "10.0.0.0",       // Start of 10.0.0.0/8
            "10.255.255.255", // End of 10.0.0.0/8
            "172.16.0.0",     // Start of 172.16.0.0/12
            "172.31.255.255", // End of 172.16.0.0/12
        ];

        for ip_str in edge_cases {
            let ip: IpAddr = ip_str.parse().unwrap();
            let result = validate_target_ip(&ip);
            assert_ok(result);
        }
    }

    #[test]
    fn test_just_outside_private_ranges() {
        // Test addresses just outside private ranges
        let outside_ranges = vec![
            "192.167.255.255", // Just before 192.168.0.0
            "192.169.0.0",     // Just after 192.168.255.255
            "9.255.255.255",   // Just before 10.0.0.0
            "11.0.0.0",        // Just after 10.255.255.255
            "172.15.255.255",  // Just before 172.16.0.0
            "172.32.0.0",      // Just after 172.31.255.255
        ];

        for ip_str in outside_ranges {
            let ip: IpAddr = ip_str.parse().unwrap();
            let result = validate_target_ip(&ip);
            assert_is_validation_error(result);
        }
    }
}


#[cfg(test)]
mod validate_system_requirements_tests {
    use super::*;

    #[test]
    fn test_dry_run_bypasses_root_check() {
        // In dry run mode, should always succeed regardless of privileges
        let result = validate_system_requirements(true);
        assert_ok(result);
    }

    #[test]
    fn test_non_dry_run_checks_privileges() {
        // This test behavior depends on whether running as root
        // We can't assert a specific outcome, but we can verify it doesn't panic
        let _ = validate_system_requirements(false);
    }
}

#[cfg(test)]
mod validate_comprehensive_security_tests {
    use super::*;

    #[test]
    fn test_accepts_valid_configuration() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080, 8081];
        let threads = 4;
        let rate = 1000;

        let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
        assert_ok(result);
    }

    #[test]
    fn test_rejects_public_ip_in_comprehensive() {
        let ip: IpAddr = "8.8.8.8".parse().unwrap();
        let ports = vec![8080];
        let threads = 4;
        let rate = 1000;

        let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
        assert_is_validation_error(result);
    }

    #[test]
    fn test_rejects_invalid_threads_in_comprehensive() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080];
        let threads = 200; // Too many
        let rate = 1000;

        let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
        assert_is_validation_error(result);
    }

    #[test]
    fn test_rejects_excessive_rate_in_comprehensive() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080];
        let threads = 4;
        let rate = 20000; // Too high

        let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
        assert_is_validation_error(result);
    }

    #[test]
    fn test_rejects_empty_ports_in_comprehensive() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![];
        let threads = 4;
        let rate = 1000;

        let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
        assert_is_validation_error(result);
    }

    #[test]
    fn test_comprehensive_with_edge_values() {
        // Test with all edge values that should be valid
        let ip: IpAddr = "10.0.0.1".parse().unwrap();
        let ports = vec![1, 65535]; // Min and max ports
        let threads = 100; // Max threads
        let rate = 10000; // Max rate

        let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
        assert_ok(result);
    }
}