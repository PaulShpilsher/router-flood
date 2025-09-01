//! Network throughput performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ProtocolMix;
use router_flood::stats::stats_aggregator::Stats;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

fn bench_packet_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_throughput");
    group.measurement_time(Duration::from_secs(10));
    
    let protocol_mix = ProtocolMix::default();
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    for packet_count in &[100, 1000, 10000] {
        let mut builder = PacketBuilder::new((64, 1400), protocol_mix.clone());
        
        group.throughput(Throughput::Elements(*packet_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(packet_count),
            packet_count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        if let Ok((packet, _)) = builder.build_packet(
                            black_box(PacketType::Udp),
                            black_box(target_ip),
                            black_box(8080)
                        ) {
                            black_box(packet);
                        }
                    }
                })
            }
        );
    }
    group.finish();
}

fn bench_mixed_protocol_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_protocol_throughput");
    let protocol_mix = ProtocolMix {
        udp_ratio: 0.4,
        tcp_syn_ratio: 0.3,
        tcp_ack_ratio: 0.2,
        icmp_ratio: 0.1,
        custom_ratio: 0.0,
    };
    
    let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    group.throughput(Throughput::Elements(1000));
    group.bench_function("mixed_protocols", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let packet_type = match i % 10 {
                    0..=3 => PacketType::Udp,
                    4..=6 => PacketType::TcpSyn,
                    7..=8 => PacketType::TcpAck,
                    _ => PacketType::Icmp,
                };
                
                if let Ok((packet, _)) = builder.build_packet(
                    black_box(packet_type),
                    black_box(target_ip),
                    black_box(8080)
                ) {
                    black_box(packet);
                }
            }
        })
    });
    
    group.finish();
}

fn bench_stats_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats_collection_overhead");
    let stats = Arc::new(Stats::new(None));
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    group.bench_function("with_stats", |b| {
        b.iter(|| {
            for _ in 0..100 {
                if let Ok((packet, _)) = builder.build_packet(
                    black_box(PacketType::Udp),
                    black_box(target_ip),
                    black_box(8080)
                ) {
                    stats.increment_sent(packet.len() as u64, "udp");
                    black_box(packet);
                }
            }
        })
    });
    
    group.bench_function("without_stats", |b| {
        b.iter(|| {
            for _ in 0..100 {
                if let Ok((packet, _)) = builder.build_packet(
                    black_box(PacketType::Udp),
                    black_box(target_ip),
                    black_box(8080)
                ) {
                    black_box(packet);
                }
            }
        })
    });
    
    group.finish();
}

fn bench_burst_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("burst_patterns");
    let protocol_mix = ProtocolMix::default();
    let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    group.bench_function("steady_rate", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                if let Ok((packet, _)) = builder.build_packet(
                    black_box(PacketType::Udp),
                    black_box(target_ip),
                    black_box(8080)
                ) {
                    black_box(packet);
                }
                // Simulate steady rate with minimal delay
                std::hint::spin_loop();
            }
        })
    });
    
    group.bench_function("burst_pattern", |b| {
        b.iter(|| {
            for burst in 0..10 {
                // Burst of 100 packets
                for _ in 0..100 {
                    if let Ok((packet, _)) = builder.build_packet(
                        black_box(PacketType::Udp),
                        black_box(target_ip),
                        black_box(8080)
                    ) {
                        black_box(packet);
                    }
                }
                // Simulate burst delay
                for _ in 0..1000 {
                    std::hint::spin_loop();
                }
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_packet_throughput,
    bench_mixed_protocol_throughput,
    bench_stats_overhead,
    bench_burst_patterns
);
criterion_main!(benches);