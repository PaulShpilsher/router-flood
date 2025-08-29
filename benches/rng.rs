//! Random number generation benchmarks
//!
//! Measures the performance of batched RNG vs standard RNG,
//! different value types, and batch size impacts.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::utils::rng::{BatchedRng, DEFAULT_BATCH_SIZE};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Benchmark batched RNG for different value types
fn benchmark_batched_rng(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng/batched");
    
    let mut rng = BatchedRng::new();
    
    group.bench_function("port", |b| {
        b.iter(|| {
            black_box(rng.port())
        })
    });
    
    group.bench_function("sequence", |b| {
        b.iter(|| {
            black_box(rng.sequence())
        })
    });
    
    group.bench_function("identification", |b| {
        b.iter(|| {
            black_box(rng.identification())
        })
    });
    
    group.bench_function("ttl", |b| {
        b.iter(|| {
            black_box(rng.ttl())
        })
    });
    
    group.bench_function("window", |b| {
        b.iter(|| {
            black_box(rng.window())
        })
    });
    
    group.bench_function("flow_label", |b| {
        b.iter(|| {
            black_box(rng.flow_label())
        })
    });
    
    group.bench_function("random_byte", |b| {
        b.iter(|| {
            black_box(rng.random_byte())
        })
    });
    
    group.bench_function("float_range", |b| {
        b.iter(|| {
            black_box(rng.float_range(0.0, 1.0))
        })
    });
    
    group.bench_function("range", |b| {
        b.iter(|| {
            black_box(rng.range(0, 1000))
        })
    });
    
    group.finish();
}

/// Compare batched RNG with standard RNG
fn benchmark_rng_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng/comparison");
    
    // Batched RNG
    group.bench_function("batched_port", |b| {
        let mut rng = BatchedRng::new();
        b.iter(|| {
            black_box(rng.port())
        })
    });
    
    // Standard RNG
    group.bench_function("standard_port", |b| {
        let mut rng = StdRng::from_entropy();
        b.iter(|| {
            let port = rng.gen_range(1024..=65535);
            black_box(port)
        })
    });
    
    // Batched sequence
    group.bench_function("batched_sequence", |b| {
        let mut rng = BatchedRng::new();
        b.iter(|| {
            black_box(rng.sequence())
        })
    });
    
    // Standard sequence
    group.bench_function("standard_sequence", |b| {
        let mut rng = StdRng::from_entropy();
        b.iter(|| {
            let seq: u32 = rng.gen();
            black_box(seq)
        })
    });
    
    group.finish();
}

/// Benchmark different batch sizes
fn benchmark_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng/batch_size");
    
    for batch_size in [10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("size", batch_size),
            &batch_size,
            |b, &batch_size| {
                let mut rng = BatchedRng::with_batch_size(batch_size);
                b.iter(|| {
                    // Mix of operations
                    black_box(rng.port());
                    black_box(rng.sequence());
                    black_box(rng.ttl());
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark batch replenishment overhead
fn benchmark_batch_replenishment(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng/replenishment");
    
    // Measure the cost when batch is exhausted
    group.bench_function("worst_case_port", |b| {
        b.iter(|| {
            // Create new RNG each time to force replenishment
            let mut rng = BatchedRng::with_batch_size(1);
            black_box(rng.port())
        })
    });
    
    // Measure the cost with pre-populated batch
    group.bench_function("best_case_port", |b| {
        let mut rng = BatchedRng::with_batch_size(10000);
        b.iter(|| {
            black_box(rng.port())
        })
    });
    
    group.finish();
}

/// Benchmark filling random buffers
fn benchmark_buffer_filling(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng/buffer_fill");
    
    for size in [64, 256, 512, 1024, 1400] {
        group.bench_with_input(
            BenchmarkId::new("bytes", size),
            &size,
            |b, &size| {
                let mut rng = BatchedRng::new();
                let mut buffer = vec![0u8; size];
                
                b.iter(|| {
                    rng.fill_buffer(black_box(&mut buffer));
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark random packet field generation
fn benchmark_packet_fields(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng/packet_fields");
    
    group.bench_function("tcp_fields", |b| {
        let mut rng = BatchedRng::new();
        b.iter(|| {
            let src_port = rng.port();
            let dst_port = rng.port();
            let seq = rng.sequence();
            let ack = rng.sequence();
            let window = rng.window();
            black_box((src_port, dst_port, seq, ack, window))
        })
    });
    
    group.bench_function("udp_fields", |b| {
        let mut rng = BatchedRng::new();
        b.iter(|| {
            let src_port = rng.port();
            let dst_port = rng.port();
            let checksum = rng.identification();
            black_box((src_port, dst_port, checksum))
        })
    });
    
    group.bench_function("ip_fields", |b| {
        let mut rng = BatchedRng::new();
        b.iter(|| {
            let id = rng.identification();
            let ttl = rng.ttl();
            let checksum = rng.identification();
            black_box((id, ttl, checksum))
        })
    });
    
    group.finish();
}

/// Benchmark concurrent RNG usage
fn benchmark_concurrent_rng(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng/concurrent");
    
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            std::thread::spawn(|| {
                                let mut rng = BatchedRng::new();
                                let mut values = Vec::with_capacity(1000);
                                for _ in 0..1000 {
                                    values.push(rng.port());
                                }
                                values
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
    benchmark_batched_rng,
    benchmark_rng_comparison,
    benchmark_batch_sizes,
    benchmark_batch_replenishment,
    benchmark_buffer_filling,
    benchmark_packet_fields,
    benchmark_concurrent_rng
);
criterion_main!(benches);