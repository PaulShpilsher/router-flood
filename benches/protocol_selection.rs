//! Protocol selection benchmarks
//!
//! Measures the performance of protocol selection algorithms,
//! weighted random selection, and packet type distribution.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::packet::{PacketBuilder, PacketType};
use router_flood::config::ProtocolMix;
use router_flood::utils::rng::BatchedRng;
use std::net::IpAddr;
use std::collections::HashMap;

/// Benchmark basic protocol selection
fn benchmark_protocol_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_selection/basic");
    
    // Test different protocol mixes
    let mixes = vec![
        ("balanced", ProtocolMix {
            udp_ratio: 0.3,
            tcp_syn_ratio: 0.3,
            tcp_ack_ratio: 0.2,
            icmp_ratio: 0.1,
            ipv6_ratio: 0.05,
            arp_ratio: 0.05,
        }),
        ("udp_heavy", ProtocolMix {
            udp_ratio: 0.8,
            tcp_syn_ratio: 0.1,
            tcp_ack_ratio: 0.05,
            icmp_ratio: 0.03,
            ipv6_ratio: 0.01,
            arp_ratio: 0.01,
        }),
        ("tcp_heavy", ProtocolMix {
            udp_ratio: 0.1,
            tcp_syn_ratio: 0.5,
            tcp_ack_ratio: 0.3,
            icmp_ratio: 0.05,
            ipv6_ratio: 0.03,
            arp_ratio: 0.02,
        }),
    ];
    
    for (name, mix) in mixes {
        group.bench_function(name, |b| {
            b.iter(|| {
                // Simulate the protocol selection logic
                let mut rng = BatchedRng::new();
                let rand_val = rng.float_range(0.0, 1.0);
                let mut cumulative = 0.0;
                
                cumulative += mix.udp_ratio;
                if rand_val < cumulative {
                    return black_box(PacketType::Udp);
                }
                
                cumulative += mix.tcp_syn_ratio;
                if rand_val < cumulative {
                    return black_box(PacketType::TcpSyn);
                }
                
                cumulative += mix.tcp_ack_ratio;
                if rand_val < cumulative {
                    return black_box(PacketType::TcpAck);
                }
                
                cumulative += mix.icmp_ratio;
                if rand_val < cumulative {
                    return black_box(PacketType::Icmp);
                }
                
                black_box(PacketType::Udp) // Default
            })
        });
    }
    
    group.finish();
}

/// Benchmark weighted random selection algorithms
fn benchmark_weighted_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_selection/weighted");
    
    // Simple cumulative distribution selection
    group.bench_function("cumulative_distribution", |b| {
        let weights = vec![0.3, 0.3, 0.2, 0.1, 0.05, 0.05];
        let mut rng = BatchedRng::new();
        
        b.iter(|| {
            let rand_val = rng.float_range(0.0, 1.0);
            let mut cumulative = 0.0;
            let mut selected = 0;
            
            for (i, &weight) in weights.iter().enumerate() {
                cumulative += weight;
                if rand_val < cumulative {
                    selected = i;
                    break;
                }
            }
            
            black_box(selected)
        })
    });
    
    // Alias method (O(1) selection after O(n) setup)
    group.bench_function("alias_method", |b| {
        // Pre-computed alias table for fast selection
        let probabilities = vec![0.3, 0.3, 0.2, 0.1, 0.05, 0.05];
        let n = probabilities.len();
        let mut prob = vec![0.0; n];
        let mut alias = vec![0; n];
        
        // Build alias table (Walker's method)
        let avg = 1.0 / n as f64;
        let mut small = Vec::new();
        let mut large = Vec::new();
        
        for (i, &p) in probabilities.iter().enumerate() {
            let scaled = p / avg;
            if scaled < 1.0 {
                small.push(i);
            } else {
                large.push(i);
            }
            prob[i] = scaled;
        }
        
        while !small.is_empty() && !large.is_empty() {
            let l = small.pop().unwrap();
            let g = large.pop().unwrap();
            
            alias[l] = g;
            prob[g] = prob[g] + prob[l] - 1.0;
            
            if prob[g] < 1.0 {
                small.push(g);
            } else {
                large.push(g);
            }
        }
        
        let mut rng = BatchedRng::new();
        
        b.iter(|| {
            let i = rng.range(0, n);
            let r = rng.float_range(0.0, 1.0);
            
            let selected = if r < prob[i] {
                i
            } else {
                alias[i]
            };
            
            black_box(selected)
        })
    });
    
    group.finish();
}

/// Benchmark protocol selection with different packet builders
fn benchmark_packet_builder_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_selection/packet_builder");
    
    let protocol_mix = ProtocolMix {
        udp_ratio: 0.3,
        tcp_syn_ratio: 0.3,
        tcp_ack_ratio: 0.2,
        icmp_ratio: 0.1,
        ipv6_ratio: 0.05,
        arp_ratio: 0.05,
    };
    
    let mut builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
    
    // Benchmark full packet building (includes protocol selection)
    group.bench_function("full_build_with_selection", |b| {
        b.iter(|| {
            let mut buffer = vec![0u8; 1500];
            // Use next_packet_type_for_ip which performs protocol selection
            let packet_type = builder.next_packet_type_for_ip(target_ip);
            let result = builder.build_packet_into_buffer(
                &mut buffer,
                packet_type,
                target_ip,
                80,
            );
            black_box(result)
        })
    });
    
    // Benchmark just the selection logic
    group.bench_function("selection_only", |b| {
        b.iter(|| {
            // Simulate just the selection part
            let mut rng = BatchedRng::new();
            let rand_val = rng.float_range(0.0, 1.0);
            
            let packet_type = if rand_val < 0.3 {
                PacketType::Udp
            } else if rand_val < 0.6 {
                PacketType::TcpSyn
            } else if rand_val < 0.8 {
                PacketType::TcpAck
            } else if rand_val < 0.9 {
                PacketType::Icmp
            } else {
                PacketType::Udp
            };
            
            black_box(packet_type)
        })
    });
    
    group.finish();
}

/// Benchmark batch protocol selection
fn benchmark_batch_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_selection/batch");
    
    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("size", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut selections = Vec::with_capacity(batch_size);
                    let mut rng = BatchedRng::new();
                    
                    for _ in 0..batch_size {
                        let rand_val = rng.float_range(0.0, 1.0);
                        let packet_type = if rand_val < 0.4 {
                            PacketType::Udp
                        } else if rand_val < 0.7 {
                            PacketType::TcpSyn
                        } else if rand_val < 0.9 {
                            PacketType::TcpAck
                        } else {
                            PacketType::Icmp
                        };
                        selections.push(packet_type);
                    }
                    black_box(selections)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark protocol distribution accuracy
fn benchmark_distribution_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_selection/distribution");
    
    group.bench_function("10k_selections", |b| {
        b.iter(|| {
            let mut counts = HashMap::new();
            let mut rng = BatchedRng::new();
            
            for _ in 0..10000 {
                let rand_val = rng.float_range(0.0, 1.0);
                let packet_type = if rand_val < 0.4 {
                    PacketType::Udp
                } else if rand_val < 0.7 {
                    PacketType::TcpSyn
                } else if rand_val < 0.9 {
                    PacketType::TcpAck
                } else {
                    PacketType::Icmp
                };
                *counts.entry(packet_type).or_insert(0) += 1;
            }
            
            black_box(counts)
        })
    });
    
    group.finish();
}

/// Benchmark protocol selection caching
fn benchmark_cached_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_selection/cached");
    
    // Pre-generate selections for caching
    group.bench_function("pregenerated_cache", |b| {
        // Pre-generate 1000 selections
        let mut cache = Vec::with_capacity(1000);
        let mut rng = BatchedRng::new();
        
        for _ in 0..1000 {
            let rand_val = rng.float_range(0.0, 1.0);
            let packet_type = if rand_val < 0.4 {
                PacketType::Udp
            } else if rand_val < 0.7 {
                PacketType::TcpSyn
            } else if rand_val < 0.9 {
                PacketType::TcpAck
            } else {
                PacketType::Icmp
            };
            cache.push(packet_type);
        }
        
        let mut index = 0;
        
        b.iter(|| {
            let selection = cache[index % cache.len()];
            index += 1;
            black_box(selection)
        })
    });
    
    // Compare with non-cached
    group.bench_function("no_cache", |b| {
        let mut rng = BatchedRng::new();
        
        b.iter(|| {
            let rand_val = rng.float_range(0.0, 1.0);
            let packet_type = if rand_val < 0.4 {
                PacketType::Udp
            } else if rand_val < 0.7 {
                PacketType::TcpSyn
            } else if rand_val < 0.9 {
                PacketType::TcpAck
            } else {
                PacketType::Icmp
            };
            black_box(packet_type)
        })
    });
    
    group.finish();
}

/// Benchmark concurrent protocol selection
fn benchmark_concurrent_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_selection/concurrent");
    
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            std::thread::spawn(|| {
                                let mut selections = Vec::with_capacity(1000);
                                let mut rng = BatchedRng::new();
                                
                                for _ in 0..1000 {
                                    let rand_val = rng.float_range(0.0, 1.0);
                                    let packet_type = if rand_val < 0.4 {
                                        PacketType::Udp
                                    } else if rand_val < 0.7 {
                                        PacketType::TcpSyn
                                    } else if rand_val < 0.9 {
                                        PacketType::TcpAck
                                    } else {
                                        PacketType::Icmp
                                    };
                                    selections.push(packet_type);
                                }
                                
                                selections
                            })
                        })
                        .collect();
                    
                    let results: Vec<_> = handles.into_iter()
                        .map(|h| h.join().unwrap())
                        .collect();
                    
                    black_box(results)
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_protocol_selection,
    benchmark_weighted_selection,
    benchmark_packet_builder_selection,
    benchmark_batch_selection,
    benchmark_distribution_accuracy,
    benchmark_cached_selection,
    benchmark_concurrent_selection
);
criterion_main!(benches);