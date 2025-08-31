//! Buffer pool benchmarks
//!
//! Measures the performance of buffer pool operations under various
//! contention scenarios and usage patterns.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, BenchmarkId};
use router_flood::utils::buffer_pool::BufferPool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Benchmark single-threaded buffer pool operations
fn benchmark_single_thread_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/single_thread");
    
    let pool = BufferPool::new(1500, 100);
    
    group.bench_function("get_buffer", |b| {
        b.iter(|| {
            black_box(pool.buffer())
        })
    });
    
    group.bench_function("get_return_cycle", |b| {
        b.iter(|| {
            let buffer = pool.buffer();
            pool.return_buffer(black_box(buffer));
        })
    });
    
    group.bench_function("batch_operations", |b| {
        b.iter(|| {
            // Get multiple buffers
            let mut buffers = Vec::with_capacity(10);
            for _ in 0..10 {
                buffers.push(pool.buffer());
            }
            
            // Return them all
            for buffer in buffers {
                pool.return_buffer(buffer);
            }
        })
    });
    
    group.finish();
}



/// Benchmark buffer pool under contention
fn benchmark_contended_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/contention");
    
    // Set reasonable timeouts for high-contention benchmarks
    group.measurement_time(Duration::from_secs(8));
    group.warm_up_time(Duration::from_secs(2));
    group.sample_size(30); // Reduce sample size for faster completion
    
    for num_threads in [2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter_batched(
                    || Arc::new(BufferPool::new(1500, 100)),
                    |pool| {
                        let handles: Vec<_> = (0..num_threads)
                            .map(|_| {
                                let pool = Arc::clone(&pool);
                                thread::spawn(move || {
                                    for _ in 0..100 {
                                        let buffer = pool.buffer();
                                        std::hint::black_box(&buffer);
                                        pool.return_buffer(buffer);
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

/// Benchmark different pool sizes impact
fn benchmark_pool_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/pool_size");
    
    for pool_size in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("size", pool_size),
            &pool_size,
            |b, &pool_size| {
                let pool = BufferPool::new(1500, pool_size);
                
                b.iter(|| {
                    let buffer = pool.buffer();
                    pool.return_buffer(black_box(buffer));
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark different buffer sizes
fn benchmark_buffer_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/buffer_size");
    
    for buffer_size in [64, 256, 512, 1500, 4096] {
        group.bench_with_input(
            BenchmarkId::new("bytes", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let pool = BufferPool::new(buffer_size, 100);
                
                b.iter(|| {
                    let buffer = pool.buffer();
                    pool.return_buffer(black_box(buffer));
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark producer-consumer pattern
fn benchmark_producer_consumer(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/producer_consumer");
    
    group.bench_function("balanced_2x2", |b| {
        let pool = Arc::new(BufferPool::new(1500, 100));
        
        b.iter(|| {
            let pool_prod = Arc::clone(&pool);
            let pool_cons = Arc::clone(&pool);
            
            // Start producers
            let producers: Vec<_> = (0..2)
                .map(|_| {
                    let pool = Arc::clone(&pool_prod);
                    thread::spawn(move || {
                        for _ in 0..25 {
                            let buffer = pool.buffer();
                            std::hint::black_box(buffer);
                        }
                    })
                })
                .collect();
            
            // Start consumers
            let consumers: Vec<_> = (0..2)
                .map(|_| {
                    let pool = Arc::clone(&pool_cons);
                    thread::spawn(move || {
                        for _ in 0..25 {
                            let buffer = vec![0u8; 1500];
                            pool.return_buffer(buffer);
                        }
                    })
                })
                .collect();
            
            for handle in producers {
                handle.join().unwrap();
            }
            for handle in consumers {
                handle.join().unwrap();
            }
        })
    });
    
    group.finish();
}

/// Benchmark memory pressure scenarios
fn benchmark_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/memory_pressure");
    
    group.bench_function("high_churn", |b| {
        let pool = BufferPool::new(4096, 20);
        
        b.iter(|| {
            // Rapid allocation and deallocation
            for _ in 0..5 {
                let mut buffers = Vec::new();
                for _ in 0..10 {
                    buffers.push(pool.buffer());
                }
                for buffer in buffers {
                    pool.return_buffer(buffer);
                }
            }
        })
    });
    
    group.bench_function("pool_thrashing", |b| {
        let pool = Arc::new(BufferPool::new(1500, 10));
        
        b.iter(|| {
            // Force constant allocation/deallocation
            let handles: Vec<_> = (0..3)
                .map(|_| {
                    let pool = Arc::clone(&pool);
                    thread::spawn(move || {
                        for _ in 0..20 {
                            let buffer = pool.buffer();
                            std::hint::black_box(&buffer);
                            pool.return_buffer(buffer);
                        }
                    })
                })
                .collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_thread_pool,
    benchmark_contended_pool,
    benchmark_pool_size_impact,
    benchmark_buffer_sizes,
    benchmark_producer_consumer,
    benchmark_memory_pressure
);
criterion_main!(benches);