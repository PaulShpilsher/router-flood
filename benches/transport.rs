//! Transport layer benchmarks
//!
//! Measures the performance of packet sending operations, batch sending,
//! and transport layer overhead.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, BenchmarkId};
use router_flood::transport::{MockTransport, TransportLayer, ChannelType};
use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ProtocolMix;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread;

/// Benchmark single packet sending
fn benchmark_single_packet_send(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport/single_packet");
    
    let transport = Arc::new(MockTransport::new());
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Create a sample packet
    let protocol_mix = ProtocolMix {
        udp_ratio: 1.0,
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let mut buffer = vec![0u8; 1500];
    let (packet_len, _) = packet_builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        target_ip,
        80,
    ).unwrap();
    buffer.truncate(packet_len);
    
    // Benchmark IPv4 send
    group.bench_function("ipv4", |b| {
        b.iter(|| {
            transport.send_packet(
                black_box(&buffer),
                black_box(target_ip),
                black_box(ChannelType::IPv4),
            )
        })
    });
    
    // Benchmark IPv6 send
    let ipv6_target: IpAddr = "::1".parse().unwrap();
    group.bench_function("ipv6", |b| {
        b.iter(|| {
            transport.send_packet(
                black_box(&buffer),
                black_box(ipv6_target),
                black_box(ChannelType::IPv6),
            )
        })
    });
    
    group.finish();
}

/// Benchmark batch packet sending
fn benchmark_batch_send(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport/batch_send");
    
    let transport = Arc::new(MockTransport::new());
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Create sample packets
    let protocol_mix = ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.3,
        tcp_ack_ratio: 0.1,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    
    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("packets", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter_batched(
                    || {
                        // Setup: create batch of packets
                        let mut packets = Vec::with_capacity(batch_size);
                        for _ in 0..batch_size {
                            let mut buffer = vec![0u8; 1500];
                            // Use a fixed packet type for benchmarking
                            let packet_type = if rand::random::<f64>() < 0.6 {
                                PacketType::Udp
                            } else if rand::random::<f64>() < 0.3 {
                                PacketType::TcpSyn
                            } else {
                                PacketType::TcpAck
                            };
                            let (len, _) = packet_builder.build_packet_into_buffer(
                                &mut buffer,
                                packet_type,
                                target_ip,
                                80,
                            ).unwrap();
                            buffer.truncate(len);
                            packets.push(buffer);
                        }
                        packets
                    },
                    |packets| {
                        // Benchmark: send all packets
                        for packet in packets {
                            let _ = transport.send_packet(
                                &packet,
                                target_ip,
                                ChannelType::IPv4,
                            );
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent packet sending
fn benchmark_concurrent_send(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport/concurrent_send");
    
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter_batched(
                    || {
                        // Setup: create shared transport and packets
                        let transport = Arc::new(MockTransport::new());
                        let protocol_mix = ProtocolMix {
                            udp_ratio: 1.0,
                            tcp_syn_ratio: 0.0,
                            tcp_ack_ratio: 0.0,
                            icmp_ratio: 0.0,
                            ipv6_ratio: 0.0,
                            arp_ratio: 0.0,
                        };
                        
                        let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
                        let mut buffer = vec![0u8; 1500];
                        let (len, _) = packet_builder.build_packet_into_buffer(
                            &mut buffer,
                            PacketType::Udp,
                            target_ip,
                            80,
                        ).unwrap();
                        buffer.truncate(len);
                        
                        (transport, buffer)
                    },
                    |(transport, packet)| {
                        // Benchmark: concurrent sends
                        let handles: Vec<_> = (0..num_threads)
                            .map(|_| {
                                let transport = Arc::clone(&transport);
                                let packet = packet.clone();
                                thread::spawn(move || {
                                    for _ in 0..100 {
                                        let _ = transport.send_packet(
                                            &packet,
                                            target_ip,
                                            ChannelType::IPv4,
                                        );
                                    }
                                })
                            })
                            .collect();
                        
                        for handle in handles {
                            handle.join().unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

/// Benchmark different packet sizes
fn benchmark_packet_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport/packet_size");
    
    let transport = Arc::new(MockTransport::new());
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    for size in [64, 256, 512, 1024, 1400] {
        group.bench_with_input(
            BenchmarkId::new("bytes", size),
            &size,
            |b, &size| {
                let packet = vec![0u8; size];
                b.iter(|| {
                    transport.send_packet(
                        black_box(&packet),
                        black_box(target_ip),
                        black_box(ChannelType::IPv4),
                    )
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_packet_send,
    benchmark_batch_send,
    benchmark_concurrent_send,
    benchmark_packet_size_impact
);
criterion_main!(benches);