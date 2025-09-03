//! Packet generation performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::packet::{PacketBuilder, PacketType, PacketSizeRange};
use router_flood::config::ProtocolMix;
use std::net::{IpAddr, Ipv4Addr};

fn bench_udp_packet_generation(c: &mut Criterion) {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 1400), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    c.bench_function("udp_packet_generation", |b| {
        b.iter(|| {
            builder.build_packet(
                black_box(PacketType::Udp),
                black_box(target_ip),
                black_box(8080)
            )
        })
    });
}

fn bench_tcp_packet_generation(c: &mut Criterion) {
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 1400), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    c.bench_function("tcp_syn_packet_generation", |b| {
        b.iter(|| {
            builder.build_packet(
                black_box(PacketType::TcpSyn),
                black_box(target_ip),
                black_box(443)
            )
        })
    });
}

fn bench_packet_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_sizes");
    let protocol_mix = ProtocolMix::default();
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    for size in &[64, 256, 512, 1024, 1400] {
        let mut builder = PacketBuilder::new(PacketSizeRange::new(20, *size), protocol_mix.clone());
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    builder.build_packet(
                        black_box(PacketType::Udp),
                        black_box(target_ip),
                        black_box(8080)
                    )
                })
            }
        );
    }
    group.finish();
}

fn bench_zero_copy_vs_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_building_methods");
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new(PacketSizeRange::new(64, 1400), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let mut buffer = vec![0u8; 1500];
    
    group.bench_function("with_allocation", |b| {
        b.iter(|| {
            builder.build_packet(
                black_box(PacketType::Udp),
                black_box(target_ip),
                black_box(8080)
            )
        })
    });
    
    group.bench_function("zero_copy", |b| {
        b.iter(|| {
            builder.build_packet_into_buffer(
                black_box(&mut buffer),
                black_box(PacketType::Udp),
                black_box(target_ip),
                black_box(8080)
            )
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_udp_packet_generation,
    bench_tcp_packet_generation,
    bench_packet_sizes,
    bench_zero_copy_vs_allocation
);
criterion_main!(benches);