//! Packet building benchmarks

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unit_arg)]
//!
//! These benchmarks measure the performance of different packet building
//! strategies and optimizations to detect performance regressions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::utils::buffer_pool::BufferPool;
use router_flood::utils::rng::BatchedRng;
use std::net::IpAddr;

fn benchmark_packet_building(c: &mut Criterion) {
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Benchmark different packet types
    let packet_types = [
        PacketType::Udp,
        PacketType::TcpSyn,
        PacketType::TcpAck,
        PacketType::Icmp,
    ];
    
    let mut group = c.benchmark_group("packet_building");
    
    for packet_type in packet_types {
        group.bench_with_input(
            BenchmarkId::new("zero_copy", format!("{:?}", packet_type)),
            &packet_type,
            |b, &packet_type| {
                let mut buffer = vec![0u8; 1500];
                b.iter(|| {
                    black_box(packet_builder.build_packet_into_buffer(
                        black_box(&mut buffer),
                        black_box(packet_type),
                        black_box(target_ip),
                        black_box(80),
                    ))
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("allocation", format!("{:?}", packet_type)),
            &packet_type,
            |b, &packet_type| {
                b.iter(|| {
                    black_box(packet_builder.build_packet(
                        black_box(packet_type),
                        black_box(target_ip),
                        black_box(80),
                    ))
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_buffer_pools(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pools");
    
    // Benchmark buffer pool
    let buffer_pool = BufferPool::new(1400, 10, 100);
    
    group.bench_function("buffer_pool_get_return", |b| {
        b.iter(|| {
            let buffer = black_box(buffer_pool.get_buffer());
            // Buffer is automatically returned when dropped
            black_box(buffer);
        });
    });
    
    // Benchmark standard allocation
    group.bench_function("standard_allocation", |b| {
        b.iter(|| {
            let buffer = black_box(vec![0u8; 1400]);
            black_box(buffer);
        });
    });
    
    group.finish();
}

fn benchmark_protocol_selection(c: &mut Criterion) {
    let protocol_mix = router_flood::config::ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.25,
        tcp_ack_ratio: 0.05,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let ipv4_target: IpAddr = "192.168.1.1".parse().unwrap();
    let ipv6_target: IpAddr = "fe80::1".parse().unwrap();
    
    let mut group = c.benchmark_group("protocol_selection");
    
    group.bench_function("ipv4_selection", |b| {
        b.iter(|| {
            black_box(packet_builder.next_packet_type_for_ip(black_box(ipv4_target)))
        });
    });
    
    group.bench_function("ipv6_selection", |b| {
        b.iter(|| {
            black_box(packet_builder.next_packet_type_for_ip(black_box(ipv6_target)))
        });
    });
    
    group.finish();
}

fn benchmark_rng_operations(c: &mut Criterion) {
    let mut rng = BatchedRng::new();
    
    let mut group = c.benchmark_group("rng_operations");
    
    group.bench_function("port_generation", |b| {
        b.iter(|| black_box(rng.port()));
    });
    
    group.bench_function("sequence_generation", |b| {
        b.iter(|| black_box(rng.sequence()));
    });
    
    group.bench_function("payload_generation_small", |b| {
        b.iter(|| black_box(rng.payload(64)));
    });
    
    group.bench_function("payload_generation_large", |b| {
        b.iter(|| black_box(rng.payload(1400)));
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_packet_building,
    benchmark_buffer_pools,
    benchmark_protocol_selection,
    benchmark_rng_operations
);
criterion_main!(benches);