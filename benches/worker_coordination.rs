//! Worker coordination benchmarks
//!
//! Measures the performance of worker thread coordination, channel communication,
//! and work distribution mechanisms.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc;
use std::thread;

/// Benchmark channel communication patterns
fn benchmark_channel_communication(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker/channels");
    
    // MPSC channel performance
    group.bench_function("mpsc_single_item", |b| {
        b.iter(|| {
            let (tx, rx) = mpsc::channel();
            tx.send(42).unwrap();
            let value = rx.recv().unwrap();
            black_box(value)
        })
    });
    
    // Batch sending
    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("mpsc_batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let (tx, rx) = mpsc::channel();
                    
                    // Send batch
                    for i in 0..batch_size {
                        tx.send(i).unwrap();
                    }
                    
                    // Receive batch
                    let mut received = Vec::with_capacity(batch_size);
                    for _ in 0..batch_size {
                        received.push(rx.recv().unwrap());
                    }
                    
                    black_box(received)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark work distribution patterns
fn benchmark_work_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker/distribution");
    
    for num_workers in [2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("round_robin", num_workers),
            &num_workers,
            |b, &num_workers| {
                b.iter(|| {
                    let work_items = 1000;
                    let mut worker_loads = vec![0; num_workers];
                    
                    for i in 0..work_items {
                        let worker_id = i % num_workers;
                        worker_loads[worker_id] += 1;
                    }
                    
                    black_box(worker_loads)
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("least_loaded", num_workers),
            &num_workers,
            |b, &num_workers| {
                b.iter(|| {
                    let work_items = 1000;
                    let mut worker_loads = vec![0; num_workers];
                    
                    for _ in 0..work_items {
                        // Find least loaded worker
                        let min_idx = worker_loads
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, &load)| load)
                            .map(|(idx, _)| idx)
                            .unwrap();
                        
                        worker_loads[min_idx] += 1;
                    }
                    
                    black_box(worker_loads)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark barrier synchronization
fn benchmark_barrier_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker/barrier");
    
    for num_threads in [2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let barrier = Arc::new(Barrier::new(num_threads));
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                // Do some work
                                let mut sum = 0;
                                for i in 0..100 {
                                    sum += i;
                                }
                                
                                // Wait at barrier
                                barrier.wait();
                                
                                sum
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

/// Benchmark shared state coordination
fn benchmark_shared_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker/shared_state");
    
    // Mutex contention
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("mutex_contention", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let counter = Arc::new(Mutex::new(0));
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let counter = Arc::clone(&counter);
                            thread::spawn(move || {
                                for _ in 0..100 {
                                    let mut count = counter.lock().unwrap();
                                    *count += 1;
                                }
                            })
                        })
                        .collect();
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    
                    let final_count = *counter.lock().unwrap();
                    black_box(final_count)
                })
            },
        );
    }
    
    // Atomic operations (baseline for comparison)
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("atomic_ops", num_threads),
            &num_threads,
            |b, &num_threads| {
                use std::sync::atomic::{AtomicU64, Ordering};
                
                b.iter(|| {
                    let counter = Arc::new(AtomicU64::new(0));
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let counter = Arc::clone(&counter);
                            thread::spawn(move || {
                                for _ in 0..100 {
                                    counter.fetch_add(1, Ordering::Relaxed);
                                }
                            })
                        })
                        .collect();
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    
                    let final_count = counter.load(Ordering::Relaxed);
                    black_box(final_count)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark task queue patterns
fn benchmark_task_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker/task_queue");
    
    // Simple work stealing queue simulation
    group.bench_function("work_stealing", |b| {
        b.iter(|| {
            let num_workers = 4;
            let tasks_per_worker = 250i32;
            
            // Create per-worker queues
            let mut queues: Vec<Vec<i32>> = (0..num_workers)
                .map(|i| (0..tasks_per_worker).map(|j| (i as i32) * tasks_per_worker + j).collect())
                .collect();
            
            let mut completed = Vec::new();
            let mut iterations = 0;
            
            // Simulate work stealing
            while queues.iter().any(|q| !q.is_empty()) && iterations < 10000 {
                for worker_id in 0..num_workers {
                    if let Some(task) = queues[worker_id as usize].pop() {
                        completed.push(task);
                    } else {
                        // Try to steal from another worker
                        for other_id in 0..num_workers {
                            if other_id != worker_id && !queues[other_id as usize].is_empty() {
                                if let Some(task) = queues[other_id as usize].pop() {
                                    completed.push(task);
                                    break;
                                }
                            }
                        }
                    }
                }
                iterations += 1;
            }
            
            black_box(completed.len())
        })
    });
    
    group.finish();
}

/// Benchmark producer-consumer patterns
fn benchmark_producer_consumer(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker/producer_consumer");
    
    for ratio in [(1, 1), (1, 2), (2, 1), (2, 2)] {
        let (producers, consumers) = ratio;
        
        group.bench_with_input(
            BenchmarkId::new(format!("{}p_{}c", producers, consumers), format!("{}_{}", producers, consumers)),
            &ratio,
            |b, &(producers, consumers)| {
                b.iter(|| {
                    let (tx, rx) = mpsc::channel();
                    let rx = Arc::new(Mutex::new(rx));
                    
                    let mut handles = Vec::new();
                    
                    // Start producers
                    for _ in 0..producers {
                        let tx = tx.clone();
                        handles.push(thread::spawn(move || {
                            for i in 0..100 {
                                tx.send(i).unwrap();
                            }
                            0 // Return same type as consumers
                        }));
                    }
                    
                    // Start consumers
                    let items_per_producer = 100;
                    let total_items = producers * items_per_producer;
                    let items_per_consumer = total_items / consumers;
                    
                    for _ in 0..consumers {
                        let rx = Arc::clone(&rx);
                        handles.push(thread::spawn(move || {
                            let mut sum = 0;
                            for _ in 0..items_per_consumer {
                                let rx = rx.lock().unwrap();
                                if let Ok(val) = rx.recv() {
                                    sum += val;
                                }
                            }
                            sum
                        }));
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark thread pool startup overhead
fn benchmark_thread_pool_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker/thread_pool");
    
    for pool_size in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("startup", pool_size),
            &pool_size,
            |b, &pool_size| {
                b.iter(|| {
                    let handles: Vec<_> = (0..pool_size)
                        .map(|i| {
                            thread::spawn(move || {
                                // Minimal work
                                black_box(i)
                            })
                        })
                        .collect();
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_channel_communication,
    benchmark_work_distribution,
    benchmark_barrier_sync,
    benchmark_shared_state,
    benchmark_task_queue,
    benchmark_producer_consumer,
    benchmark_thread_pool_overhead
);
criterion_main!(benches);