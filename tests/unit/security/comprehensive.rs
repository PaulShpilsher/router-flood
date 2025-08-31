//! Comprehensive security validation tests

#[path = "../../common/mod.rs"]
mod common;

use common::*;
use router_flood::security::validation::{validate_comprehensive_security};
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

#[cfg(test)]
mod comprehensive_validation_tests {
    use super::*;

    #[test]
    fn test_thread_safe_validation() {
        // Test that validation functions are thread-safe
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080];
        let threads = 4;
        let rate = 1000;

        test_concurrent(10, 100, move || {
            let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
            assert_ok(result);
        });
    }

    #[test]
    fn test_validation_consistency() {
        // Ensure validation produces consistent results
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let ports = vec![8080, 8081, 8082];
        let threads = 8;
        let rate = 5000;

        assert_consistent(100, || {
            validate_comprehensive_security(&ip, &ports, threads, rate as u64).is_ok()
        });
    }

    #[test]
    fn test_multiple_private_ranges_simultaneously() {
        // Test validation across different private ranges
        let test_cases = vec![
            ("192.168.1.1", vec![80], 4, 1000),
            ("10.0.0.1", vec![443], 8, 2000),
            ("172.16.0.1", vec![8080], 16, 3000),
            ("172.31.255.254", vec![3000], 32, 4000),
        ];

        for (ip_str, ports, threads, rate) in test_cases {
            let ip: IpAddr = ip_str.parse().unwrap();
            let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
            assert_ok(result);
        }
    }

    #[test]
    fn test_validation_with_mixed_valid_invalid() {
        // Test that any invalid parameter causes overall failure
        struct TestCase {
            ip: &'static str,
            ports: Vec<u16>,
            threads: usize,
            rate: u64,
            should_succeed: bool,
        }

        let test_cases = vec![
            TestCase {
                ip: "192.168.1.1",
                ports: vec![80],
                threads: 4,
                rate: 1000,
                should_succeed: true,
            },
            TestCase {
                ip: "8.8.8.8", // Invalid: public IP
                ports: vec![80],
                threads: 4,
                rate: 1000,
                should_succeed: false,
            },
            TestCase {
                ip: "192.168.1.1",
                ports: vec![], // Empty ports are actually allowed
                threads: 4,
                rate: 1000,
                should_succeed: true,
            },
            TestCase {
                ip: "192.168.1.1",
                ports: vec![80],
                threads: 0, // Zero threads are actually allowed (not checked)
                rate: 1000,
                should_succeed: true,
            },
            TestCase {
                ip: "192.168.1.1",
                ports: vec![80],
                threads: 101, // Invalid: too many threads
                rate: 1000,
                should_succeed: false,
            },
            TestCase {
                ip: "192.168.1.1",
                ports: vec![80],
                threads: 4,
                rate: 10001, // Invalid: rate too high
                should_succeed: false,
            },
        ];

        for (i, tc) in test_cases.iter().enumerate() {
            let ip: IpAddr = tc.ip.parse().unwrap();
            let result = validate_comprehensive_security(&ip, &tc.ports, tc.threads, tc.rate);
            
            if tc.should_succeed {
                assert_ok(result);
            } else {
                assert_is_validation_error(result);
            }
        }
    }

    #[test]
    fn test_ipv6_comprehensive_validation() {
        // Test comprehensive validation with IPv6 addresses
        let valid_ipv6_cases = vec![
            "fe80::1",                              // Link-local
            "fc00::1",                              // Unique local
            "fd00::1",                              // Unique local
            "febf:ffff:ffff:ffff:ffff:ffff:ffff:fffe", // Link-local edge
        ];

        for ip_str in valid_ipv6_cases {
            let ip: IpAddr = ip_str.parse().unwrap();
            let result = validate_comprehensive_security(&ip, &vec![8080], 4, 1000);
            assert_ok(result);
        }

        let invalid_ipv6_cases = vec![
            "2001:4860:4860::8888", // Google DNS (public)
            "::1",                  // Loopback
            "ff02::1",              // Multicast
        ];

        for ip_str in invalid_ipv6_cases {
            let ip: IpAddr = ip_str.parse().unwrap();
            let result = validate_comprehensive_security(&ip, &vec![8080], 4, 1000);
            assert_is_validation_error(result);
        }
    }

    #[test]
    fn test_boundary_combinations() {
        // Test various combinations of boundary values
        let boundary_tests = vec![
            // Min values
            ("192.168.1.1", vec![1], 1, 0),       // All minimums
            ("192.168.1.1", vec![65535], 100, 10000), // All maximums
            
            // Mixed boundaries
            ("10.0.0.0", vec![1], 100, 0),        // IP edge, port min, thread max, rate min
            ("10.255.255.255", vec![65535], 1, 10000), // IP edge, port max, thread min, rate max
            
            // Multiple ports at boundaries
            ("172.16.0.0", vec![1, 32768, 65535], 50, 5000),
            ("172.31.255.255", vec![22, 80, 443, 8080], 50, 5000),
        ];

        for (ip_str, ports, threads, rate) in boundary_tests {
            let ip: IpAddr = ip_str.parse().unwrap();
            let result = validate_comprehensive_security(&ip, &ports, threads, rate as u64);
            assert_ok(result);
        }
    }

    #[test]
    fn test_concurrent_validation_stress() {
        // Stress test with many concurrent validations
        let success_count = Arc::new(AtomicU32::new(0));
        let failure_count = Arc::new(AtomicU32::new(0));
        
        let mut handles = vec![];
        
        for thread_id in 0..10 {
            let success = success_count.clone();
            let failure = failure_count.clone();
            
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    // Mix of valid and invalid configurations
                    let (ip_str, should_succeed) = if (thread_id + i) % 3 == 0 {
                        ("8.8.8.8", false) // Invalid
                    } else {
                        ("192.168.1.1", true) // Valid
                    };
                    
                    let ip: IpAddr = ip_str.parse().unwrap();
                    let result = validate_comprehensive_security(
                        &ip,
                        &vec![8080],
                        4,
                        1000
                    );
                    
                    if result.is_ok() {
                        success.fetch_add(1, Ordering::Relaxed);
                        assert!(should_succeed);
                    } else {
                        failure.fetch_add(1, Ordering::Relaxed);
                        assert!(!should_succeed);
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        let total = success_count.load(Ordering::Relaxed) + failure_count.load(Ordering::Relaxed);
        assert_eq!(total, 1000, "Expected 1000 total validations");
    }
}