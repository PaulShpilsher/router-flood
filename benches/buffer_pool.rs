//! Buffer pool benchmarks
//!
//! Measures the performance of buffer pool operations under various
//! contention scenarios and usage patterns.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, BenchmarkId};
use router_flood::utils::buffer_pool::{BufferPool, WorkerBufferPool};
use std::sync::Arc;
use std::thread;

/// Benchmark single-threaded buffer pool operations
fn benchmark_single_thread_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/single_thread");
    
    let pool = BufferPool::new(1500, 100, 1000);
    
    group.bench_function("get_buffer", |b| {
        b.iter(|| {
            black_box(pool.get_buffer())
        })
    });
    
    group.bench_function("get_return_cycle", |b| {
        b.iter(|| {
            let buffer = pool.get_buffer();
            pool.return_buffer(black_box(buffer));
        })
    });
    
    // Benchmark when pool is empty
    group.bench_function("get_buffer_empty_pool", |b| {
        // Drain the pool
        let temp_pool = BufferPool::new(1500, 0, 1000);
        
        b.iter(|| {
            black_box(temp_pool.get_buffer())
        })
    });
    
    // Benchmark when pool is full
    group.bench_function("return_buffer_full_pool", |b| {
        let full_pool = BufferPool::new(1500, 1000, 1000);
        
        b.iter(|| {
            let buffer = vec![0u8; 1500];
            full_pool.return_buffer(black_box(buffer));
        })
    });
    
    group.finish();
}

/// Benchmark per-worker buffer pool (no contention)
fn benchmark_worker_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool/worker_pool");
    
    let mut pool = WorkerBufferPool::new(1500, 100, 1000);
    
    group.bench_function("get_buffer", |b| {
        b.iter(|| {
            black_box(pool.get_buffer())
        })
    });
    
    group.bench_function("get_return_cycle", |b| {
        b.iter(|| {
            let buffer = pool.get_buffer();
            pool.return_buffer(black_box(buffer));
        })
    });
    
    group.bench_function("batch_operations", |b| {
        b.iter(|| {
            // Get multiple buffers
            let mut buffers = Vec::with_capacity(10);
            for _ in 0..10 {
                buffers.push(pool.get_buffer());
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
    
    for num_threads in [2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter_batched(
                    || Arc::new(BufferPool::new(1500, 100, 1000)),
                    |pool| {
                        let handles: Vec<_> = (0..num_threads)
                            .map(|_| {
                                let pool = Arc::clone(&pool);
                                thread::spawn(move || {
                                    for _ in 0..100 {
                                        let buffer = pool.get_buffer();
                                        // Simulate some work
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
    
    for initial_size in [0, 10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("initial", initial_size),
            &initial_size,
            |b, &initial_size| {
                let pool = BufferPool::new(1500, initial_size, 10000);
                
                b.iter(|| {
                    let buffer = pool.get_buffer();
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
    
    for buffer_size in [64, 256, 512, 1500, 4096, 9000] {
        group.bench_with_input(
            BenchmarkId::new("bytes", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let pool = BufferPool::new(buffer_size, 100, 1000);
                
                b.iter(|| {
                    let buffer = pool.get_buffer();
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
        let pool = Arc::new(BufferPool::new(1500, 100, 1000));
        
        b.iter(|| {
            let pool_prod = Arc::clone(&pool);
            let pool_cons = Arc::clone(&pool);
            
            // Start producers
            let producers: Vec<_> = (0..2)
                .map(|_| {
                    let pool = Arc::clone(&pool_prod);
                    thread::spawn(move || {
                        for _ in 0..50 {
                            let buffer = pool.get_buffer();
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
                        for _ in 0..50 {
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
    
    // Simulate high memory usage scenario
    group.bench_function("high_churn", |b| {
        let pool = BufferPool::new(9000, 10, 100);
        
        b.iter(|| {
            // Rapid allocation and deallocation
            for _ in 0..10 {
                let mut buffers = Vec::new();
                for _ in 0..10 {
                    buffers.push(pool.get_buffer());
                }
                for buffer in buffers {
                    pool.return_buffer(buffer);
                }
            }
        })
    });
    
    group.bench_function("pool_thrashing", |b| {
        let pool = BufferPool::new(1500, 1, 10);
        
        b.iter(|| {
            // Force constant allocation/deallocation
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let pool = pool.clone();
                    thread::spawn(move || {
                        for _ in 0..25 {
                            let buffer = pool.get_buffer();
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
    benchmark_worker_pool,
    benchmark_contended_pool,
    benchmark_pool_size_impact,
    benchmark_buffer_sizes,
    benchmark_producer_consumer,
    benchmark_memory_pressure
);
criterion_main!(benches);