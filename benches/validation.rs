//! Validation benchmarks
//!
//! Measures the performance of IP validation, security checks,
//! and other validation operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::security::validation::{validate_target_ip, validate_comprehensive_security};
use router_flood::config::{LoadConfig, BurstPattern};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Benchmark IP validation for different address types
fn benchmark_ip_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation/ip");
    
    // Test various IP addresses
    let test_ips = vec![
        ("private_ipv4_192", "192.168.1.1".parse::<IpAddr>().unwrap()),
        ("private_ipv4_10", "10.0.0.1".parse::<IpAddr>().unwrap()),
        ("private_ipv4_172", "172.16.0.1".parse::<IpAddr>().unwrap()),
        ("public_ipv4", "8.8.8.8".parse::<IpAddr>().unwrap()),
        ("ipv6_link_local", "fe80::1".parse::<IpAddr>().unwrap()),
        ("ipv6_unique_local", "fc00::1".parse::<IpAddr>().unwrap()),
        ("ipv6_public", "2001:4860:4860::8888".parse::<IpAddr>().unwrap()),
    ];
    
    for (name, ip) in test_ips {
        group.bench_function(name, |b| {
            b.iter(|| {
                let _ = validate_target_ip(black_box(&ip));
            })
        });
    }
    
    group.finish();
}

/// Benchmark batch IP validation
fn benchmark_batch_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation/batch");
    
    for count in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("ips", count),
            &count,
            |b, &count| {
                // Generate a mix of IPs
                let ips: Vec<IpAddr> = (0..count)
                    .map(|i| {
                        let octet = (i % 256) as u8;
                        if i % 3 == 0 {
                            IpAddr::V4(Ipv4Addr::new(192, 168, 1, octet))
                        } else if i % 3 == 1 {
                            IpAddr::V4(Ipv4Addr::new(10, 0, 0, octet))
                        } else {
                            IpAddr::V4(Ipv4Addr::new(172, 16, 0, octet))
                        }
                    })
                    .collect();
                
                b.iter(|| {
                    for ip in &ips {
                        let _ = validate_target_ip(black_box(ip));
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark comprehensive security validation
fn benchmark_security_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation/security");
    
    let config = LoadConfig {
        threads: 4,
        packet_rate: 1000,
        duration: Some(60),
        packet_size_range: (64, 1400),
        burst_pattern: BurstPattern::Sustained { rate: 1000 },
        randomize_timing: false,
    };
    
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    let ports = vec![80, 443, 8080];
    
    group.bench_function("comprehensive", |b| {
        b.iter(|| {
            let _ = validate_comprehensive_security(
                black_box(&target_ip),
                black_box(&ports),
                black_box(config.threads),
                black_box(config.packet_rate),
            );
        })
    });
    
    // Test with different thread counts
    for threads in [1, 10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("threads", threads),
            &threads,
            |b, &threads| {
                let mut test_config = config.clone();
                test_config.threads = threads;
                
                b.iter(|| {
                    let _ = validate_comprehensive_security(
                        black_box(&target_ip),
                        black_box(&ports),
                        black_box(test_config.threads),
                        black_box(test_config.packet_rate),
                    );
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark port validation
fn benchmark_port_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation/ports");
    
    // Well-known ports vs random ports
    group.bench_function("well_known", |b| {
        let ports = vec![22, 80, 443, 3306, 5432];
        b.iter(|| {
            for port in &ports {
                // Simulate port validation logic
                let is_well_known = *port < 1024;
                black_box(is_well_known);
            }
        })
    });
    
    group.bench_function("random_ports", |b| {
        let ports: Vec<u16> = (8000..9000).step_by(10).collect();
        b.iter(|| {
            for port in &ports {
                let is_well_known = *port < 1024;
                black_box(is_well_known);
            }
        })
    });
    
    group.finish();
}

/// Benchmark IPv4 vs IPv6 validation performance
fn benchmark_ip_version_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation/ip_version");
    
    let ipv4_addrs: Vec<IpAddr> = (0..100)
        .map(|i| IpAddr::V4(Ipv4Addr::new(192, 168, 1, (i % 256) as u8)))
        .collect();
    
    let ipv6_addrs: Vec<IpAddr> = (0..100)
        .map(|i| {
            IpAddr::V6(Ipv6Addr::new(
                0xfe80, 0, 0, 0, 
                0, 0, 0, i as u16
            ))
        })
        .collect();
    
    group.bench_function("ipv4_validation", |b| {
        b.iter(|| {
            for ip in &ipv4_addrs {
                let _ = validate_target_ip(black_box(ip));
            }
        })
    });
    
    group.bench_function("ipv6_validation", |b| {
        b.iter(|| {
            for ip in &ipv6_addrs {
                let _ = validate_target_ip(black_box(ip));
            }
        })
    });
    
    group.finish();
}

/// Benchmark parameter range validation
fn benchmark_parameter_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation/parameters");
    
    group.bench_function("packet_rate", |b| {
        let rates = vec![10, 100, 1000, 10000, 100000];
        b.iter(|| {
            for rate in &rates {
                let is_valid = *rate <= 100000; // MAX_PACKET_RATE
                black_box(is_valid);
            }
        })
    });
    
    group.bench_function("thread_count", |b| {
        let thread_counts = vec![1, 2, 4, 8, 16, 32, 64, 128];
        b.iter(|| {
            for count in &thread_counts {
                let is_valid = *count <= 100; // MAX_THREADS
                black_box(is_valid);
            }
        })
    });
    
    group.bench_function("packet_size", |b| {
        let sizes = vec![64, 128, 256, 512, 1024, 1400, 1500, 9000];
        b.iter(|| {
            for size in &sizes {
                let is_valid = *size >= 64 && *size <= 9000;
                black_box(is_valid);
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_ip_validation,
    benchmark_batch_validation,
    benchmark_security_validation,
    benchmark_port_validation,
    benchmark_ip_version_comparison,
    benchmark_parameter_validation
);
criterion_main!(benches);