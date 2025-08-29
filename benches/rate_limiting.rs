//! Rate limiting benchmarks
//!
//! Measures the performance and accuracy of rate limiting mechanisms,
//! including token bucket algorithms and timing precision.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Simple token bucket rate limiter for benchmarking
struct TokenBucket {
    tokens: AtomicU64,
    max_tokens: u64,
    refill_rate: u64, // tokens per second
    last_refill: AtomicU64, // nanoseconds since start
}

impl TokenBucket {
    fn new(max_tokens: u64, refill_rate: u64) -> Self {
        Self {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
            refill_rate,
            last_refill: AtomicU64::new(0),
        }
    }
    
    fn try_consume(&self, tokens: u64, now_nanos: u64) -> bool {
        // Refill tokens based on elapsed time
        let last = self.last_refill.load(Ordering::Relaxed);
        if now_nanos > last {
            let elapsed_secs = (now_nanos - last) as f64 / 1_000_000_000.0;
            let new_tokens = (self.refill_rate as f64 * elapsed_secs) as u64;
            
            if new_tokens > 0 {
                self.last_refill.store(now_nanos, Ordering::Relaxed);
                let current = self.tokens.load(Ordering::Relaxed);
                let updated = (current + new_tokens).min(self.max_tokens);
                self.tokens.store(updated, Ordering::Relaxed);
            }
        }
        
        // Try to consume tokens
        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current < tokens {
                return false;
            }
            
            match self.tokens.compare_exchange_weak(
                current,
                current - tokens,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }
}

/// Benchmark token bucket operations
fn benchmark_token_bucket(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting/token_bucket");
    
    let bucket = TokenBucket::new(1000, 10000);
    let start = Instant::now();
    
    group.bench_function("try_consume_success", |b| {
        b.iter(|| {
            let now_nanos = start.elapsed().as_nanos() as u64;
            bucket.try_consume(black_box(1), black_box(now_nanos))
        })
    });
    
    group.bench_function("try_consume_failure", |b| {
        // Drain the bucket
        let now_nanos = start.elapsed().as_nanos() as u64;
        while bucket.try_consume(100, now_nanos) {}
        
        b.iter(|| {
            let now_nanos = start.elapsed().as_nanos() as u64;
            bucket.try_consume(black_box(100), black_box(now_nanos))
        })
    });
    
    group.finish();
}

/// Benchmark sleep-based rate limiting
fn benchmark_sleep_rate_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting/sleep_based");
    
    for target_rate in [100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("pps", target_rate),
            &target_rate,
            |b, &target_rate| {
                let delay_nanos = 1_000_000_000 / target_rate;
                let _delay = Duration::from_nanos(delay_nanos);
                
                b.iter(|| {
                    // Measure the overhead of setting up the sleep
                    std::thread::sleep(black_box(Duration::from_nanos(1)));
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark spin-wait rate limiting for high precision
fn benchmark_spinwait_rate_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting/spinwait");
    
    for target_rate in [10000, 50000, 100000] {
        group.bench_with_input(
            BenchmarkId::new("pps", target_rate),
            &target_rate,
            |b, &target_rate| {
                let delay_nanos = 1_000_000_000 / target_rate;
                
                b.iter(|| {
                    let start = Instant::now();
                    while start.elapsed().as_nanos() < delay_nanos as u128 {
                        std::hint::spin_loop();
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark hybrid rate limiting (sleep + spinwait)
fn benchmark_hybrid_rate_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting/hybrid");
    
    for target_rate in [1000, 10000, 50000] {
        group.bench_with_input(
            BenchmarkId::new("pps", target_rate),
            &target_rate,
            |b, &target_rate| {
                let delay_nanos = 1_000_000_000 / target_rate;
                
                b.iter(|| {
                    let start = Instant::now();
                    
                    // Sleep for most of the duration
                    if delay_nanos > 10000 {
                        let sleep_nanos = delay_nanos - 1000;
                        std::thread::sleep(Duration::from_nanos(sleep_nanos));
                    }
                    
                    // Spin-wait for precision
                    while start.elapsed().as_nanos() < delay_nanos as u128 {
                        std::hint::spin_loop();
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark rate limiting with jitter
fn benchmark_jittered_rate_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting/jittered");
    
    group.bench_function("1000pps_with_jitter", |b| {
        let base_delay = Duration::from_micros(1000);
        
        b.iter(|| {
            // Simulate jitter between 0.9x and 1.1x
            let jitter = 0.9 + (rand::random::<f64>() * 0.2);
            let _jittered_delay = base_delay.mul_f64(jitter);
            std::thread::sleep(black_box(Duration::from_nanos(1)));
        })
    });
    
    group.finish();
}

/// Benchmark concurrent rate limiting
fn benchmark_concurrent_rate_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting/concurrent");
    
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                let bucket = Arc::new(TokenBucket::new(10000, 100000));
                let start = Instant::now();
                
                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let bucket = Arc::clone(&bucket);
                            let start = start;
                            std::thread::spawn(move || {
                                for _ in 0..100 {
                                    let now_nanos = start.elapsed().as_nanos() as u64;
                                    bucket.try_consume(1, now_nanos);
                                }
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
    benchmark_token_bucket,
    benchmark_sleep_rate_limiting,
    benchmark_spinwait_rate_limiting,
    benchmark_hybrid_rate_limiting,
    benchmark_jittered_rate_limiting,
    benchmark_concurrent_rate_limiting
);
criterion_main!(benches);