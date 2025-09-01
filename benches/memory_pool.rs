//! Memory pool performance benchmarks
//! Note: Since MemoryPool doesn't exist in the codebase, 
//! these benchmarks simulate the expected behavior

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Simulate a simple memory pool for benchmarking
struct SimulatedMemoryPool {
    buffers: Vec<Vec<u8>>,
}

impl SimulatedMemoryPool {
    fn new(size: usize, buffer_size: usize) -> Self {
        let mut buffers = Vec::with_capacity(size);
        for _ in 0..size {
            buffers.push(vec![0u8; buffer_size]);
        }
        Self { buffers }
    }
    
    fn allocate(&mut self) -> Option<Vec<u8>> {
        self.buffers.pop()
    }
    
    fn deallocate(&mut self, buffer: Vec<u8>) {
        self.buffers.push(buffer);
    }
}

fn bench_allocation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pool_allocation");
    
    for pool_size in &[100, 1000, 10000] {
        group.throughput(Throughput::Elements(*pool_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(pool_size),
            pool_size,
            |b, &size| {
                let mut pool = SimulatedMemoryPool::new(size, 1500);
                let mut buffers = Vec::new();
                
                b.iter(|| {
                    // Allocate all
                    for _ in 0..size {
                        if let Some(buf) = pool.allocate() {
                            buffers.push(buf);
                        }
                    }
                    // Return all
                    while let Some(buf) = buffers.pop() {
                        pool.deallocate(buf);
                    }
                })
            }
        );
    }
    group.finish();
}

fn bench_deallocation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pool_deallocation");
    
    for pool_size in &[100, 1000, 10000] {
        group.throughput(Throughput::Elements(*pool_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(pool_size),
            pool_size,
            |b, &size| {
                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;
                    
                    for _ in 0..iters {
                        let mut pool = SimulatedMemoryPool::new(0, 1500);
                        let mut buffers = Vec::new();
                        
                        // Pre-allocate buffers
                        for _ in 0..size {
                            buffers.push(vec![0u8; 1500]);
                        }
                        
                        // Measure only deallocation
                        let start = std::time::Instant::now();
                        while let Some(buf) = buffers.pop() {
                            pool.deallocate(buf);
                        }
                        total += start.elapsed();
                    }
                    
                    total
                })
            }
        );
    }
    group.finish();
}

fn bench_concurrent_access_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_pool_access");
    
    for thread_count in &[1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, &threads| {
                let pool = Arc::new(std::sync::Mutex::new(
                    SimulatedMemoryPool::new(1000, 1500)
                ));
                
                b.iter(|| {
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let pool_clone = Arc::clone(&pool);
                            thread::spawn(move || {
                                for _ in 0..10 {
                                    let mut pool = pool_clone.lock().unwrap();
                                    if let Some(mut buffer) = pool.allocate() {
                                        buffer[0] = 0xFF;
                                        black_box(&buffer);
                                        pool.deallocate(buffer);
                                    }
                                }
                            })
                        })
                        .collect();
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                })
            }
        );
    }
    group.finish();
}

fn bench_pool_vs_heap(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_comparison");
    
    group.bench_function("simulated_pool", |b| {
        let mut pool = SimulatedMemoryPool::new(1000, 1500);
        b.iter(|| {
            if let Some(mut buffer) = pool.allocate() {
                buffer[0] = 0xFF;
                black_box(&buffer);
                pool.deallocate(buffer);
            }
        })
    });
    
    group.bench_function("heap_allocation", |b| {
        b.iter(|| {
            let mut buffer = vec![0u8; 1500];
            buffer[0] = 0xFF;
            black_box(buffer);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_allocation_throughput,
    bench_deallocation_throughput,
    bench_concurrent_access_scaling,
    bench_pool_vs_heap
);
criterion_main!(benches);