//! Benchmarks for packet strategy implementations
//!
//! Measures the performance of different packet generation strategies
//! including UDP, TCP, ICMP, IPv6, and ARP.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::packet::strategies::{
    UdpStrategy, TcpStrategy, IcmpStrategy,
    Ipv6UdpStrategy, Ipv6TcpStrategy, Ipv6IcmpStrategy, ArpStrategy
};
use router_flood::packet::{PacketStrategy, PacketTarget};
use router_flood::utils::rng::BatchedRng;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Benchmark UDP packet generation
fn benchmark_udp_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/udp");
    
    let mut rng = BatchedRng::new();
    let packet_size_range = (64, 1400);
    let target = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        80
    );
    
    for buffer_size in [128, 512, 1400] {
        group.bench_with_input(
            BenchmarkId::new("build", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let mut strategy = UdpStrategy::new(packet_size_range, &mut rng);
                let mut buffer = vec![0u8; buffer_size];
                
                b.iter(|| {
                    let size = strategy.build_packet(&target, &mut buffer).unwrap();
                    black_box(size)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark TCP SYN packet generation
fn benchmark_tcp_syn_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/tcp_syn");
    
    let mut rng = BatchedRng::new();
    let target = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        443
    );
    
    for buffer_size in [128, 256, 512] {
        group.bench_with_input(
            BenchmarkId::new("build", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let mut strategy = TcpStrategy::new_syn(&mut rng);
                let mut buffer = vec![0u8; buffer_size];
                
                b.iter(|| {
                    let size = strategy.build_packet(&target, &mut buffer).unwrap();
                    black_box(size)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark TCP ACK packet generation  
fn benchmark_tcp_ack_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/tcp_ack");
    
    let mut rng = BatchedRng::new();
    let target = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        443
    );
    
    for buffer_size in [128, 256, 512] {
        group.bench_with_input(
            BenchmarkId::new("build", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let mut strategy = TcpStrategy::new_ack(&mut rng);
                let mut buffer = vec![0u8; buffer_size];
                
                b.iter(|| {
                    let size = strategy.build_packet(&target, &mut buffer).unwrap();
                    black_box(size)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark ICMP packet generation
fn benchmark_icmp_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/icmp");
    
    let mut rng = BatchedRng::new();
    let target = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
        0 // ICMP doesn't use ports
    );
    
    for buffer_size in [64, 128, 256] {
        group.bench_with_input(
            BenchmarkId::new("build", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let mut strategy = IcmpStrategy::new(&mut rng);
                let mut buffer = vec![0u8; buffer_size];
                
                b.iter(|| {
                    let size = strategy.build_packet(&target, &mut buffer).unwrap();
                    black_box(size)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark IPv6 UDP packet generation
fn benchmark_ipv6_udp_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/ipv6_udp");
    
    let mut rng = BatchedRng::new();
    let packet_size_range = (64, 1400);
    let target = PacketTarget::new(
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
        53
    );
    
    group.bench_function("build", |b| {
        let mut strategy = Ipv6UdpStrategy::new(packet_size_range, &mut rng);
        let mut buffer = vec![0u8; 512];
        
        b.iter(|| {
            let size = strategy.build_packet(&target, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.finish();
}

/// Benchmark IPv6 TCP packet generation
fn benchmark_ipv6_tcp_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/ipv6_tcp");
    
    let mut rng = BatchedRng::new();
    let target = PacketTarget::new(
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2)),
        22
    );
    
    group.bench_function("build", |b| {
        let mut strategy = Ipv6TcpStrategy::new(&mut rng);
        let mut buffer = vec![0u8; 256];
        
        b.iter(|| {
            let size = strategy.build_packet(&target, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.finish();
}

/// Benchmark IPv6 ICMP packet generation
fn benchmark_ipv6_icmp_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/ipv6_icmp");
    
    let mut rng = BatchedRng::new();
    let target = PacketTarget::new(
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 3)),
        0
    );
    
    group.bench_function("build", |b| {
        let mut strategy = Ipv6IcmpStrategy::new(&mut rng);
        let mut buffer = vec![0u8; 256];
        
        b.iter(|| {
            let size = strategy.build_packet(&target, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.finish();
}

/// Benchmark ARP packet generation
fn benchmark_arp_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/arp");
    
    let mut rng = BatchedRng::new();
    let target = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)),
        0
    );
    
    group.bench_function("build", |b| {
        let mut strategy = ArpStrategy::new(&mut rng);
        let mut buffer = vec![0u8; 64];
        
        b.iter(|| {
            let size = strategy.build_packet(&target, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.finish();
}

/// Benchmark strategy comparison
fn benchmark_strategy_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/comparison");
    
    let mut rng = BatchedRng::new();
    let target_v4 = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        80
    );
    let target_v6 = PacketTarget::new(
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
        80
    );
    
    // Compare all strategies with same buffer size
    let buffer_size = 512;
    
    group.bench_function("udp_v4", |b| {
        let mut strategy = UdpStrategy::new((64, 1400), &mut rng);
        let mut buffer = vec![0u8; buffer_size];
        b.iter(|| {
            let size = strategy.build_packet(&target_v4, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.bench_function("tcp_syn_v4", |b| {
        let mut strategy = TcpStrategy::new_syn(&mut rng);
        let mut buffer = vec![0u8; buffer_size];
        b.iter(|| {
            let size = strategy.build_packet(&target_v4, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.bench_function("icmp_v4", |b| {
        let mut strategy = IcmpStrategy::new(&mut rng);
        let mut buffer = vec![0u8; buffer_size];
        b.iter(|| {
            let size = strategy.build_packet(&target_v4, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.bench_function("udp_v6", |b| {
        let mut strategy = Ipv6UdpStrategy::new((64, 1400), &mut rng);
        let mut buffer = vec![0u8; buffer_size];
        b.iter(|| {
            let size = strategy.build_packet(&target_v6, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.bench_function("tcp_v6", |b| {
        let mut strategy = Ipv6TcpStrategy::new(&mut rng);
        let mut buffer = vec![0u8; buffer_size];
        b.iter(|| {
            let size = strategy.build_packet(&target_v6, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.bench_function("arp", |b| {
        let mut strategy = ArpStrategy::new(&mut rng);
        let mut buffer = vec![0u8; 64];
        b.iter(|| {
            let size = strategy.build_packet(&target_v4, &mut buffer).unwrap();
            black_box(size)
        })
    });
    
    group.finish();
}

/// Benchmark buffer size impact
fn benchmark_buffer_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/buffer_size");
    
    let mut rng = BatchedRng::new();
    let target = PacketTarget::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        80
    );
    
    for buffer_size in [64, 128, 256, 512, 1024, 1400] {
        group.bench_with_input(
            BenchmarkId::new("udp", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let mut strategy = UdpStrategy::new((64, buffer_size), &mut rng);
                let mut buffer = vec![0u8; buffer_size];
                
                b.iter(|| {
                    let size = strategy.build_packet(&target, &mut buffer).unwrap();
                    black_box(size)
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_udp_strategy,
    benchmark_tcp_syn_strategy,
    benchmark_tcp_ack_strategy,
    benchmark_icmp_strategy,
    benchmark_ipv6_udp_strategy,
    benchmark_ipv6_tcp_strategy,
    benchmark_ipv6_icmp_strategy,
    benchmark_arp_strategy,
    benchmark_strategy_comparison,
    benchmark_buffer_sizes
);
criterion_main!(benches);