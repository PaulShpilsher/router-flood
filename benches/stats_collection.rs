//! Statistics collection performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use router_flood::stats::Stats;
use std::sync::Arc;

fn bench_stats_increment(c: &mut Criterion) {
    let stats = Stats::new(None);
    
    c.bench_function("stats_increment_sent", |b| {
        b.iter(|| {
            stats.increment_sent(black_box(64), black_box("UDP"))
        })
    });
    
    c.bench_function("stats_increment_failed", |b| {
        b.iter(|| {
            stats.increment_failed()
        })
    });
}

fn bench_stats_read(c: &mut Criterion) {
    let stats = Stats::new(None);
    
    // Populate some data
    for _ in 0..1000 {
        stats.increment_sent(64, "UDP");
    }
    
    c.bench_function("stats_read_packets_sent", |b| {
        b.iter(|| {
            black_box(stats.packets_sent())
        })
    });
    
    c.bench_function("stats_read_bytes_sent", |b| {
        b.iter(|| {
            black_box(stats.bytes_sent())
        })
    });
}

fn bench_concurrent_stats(c: &mut Criterion) {
    c.bench_function("stats_concurrent_updates", |b| {
        let stats = Arc::new(Stats::new(None));
        
        b.iter(|| {
            let stats_clone = Arc::clone(&stats);
            std::thread::scope(|s| {
                for _ in 0..4 {
                    let stats = Arc::clone(&stats_clone);
                    s.spawn(move || {
                        for _ in 0..100 {
                            stats.increment_sent(64, "UDP");
                        }
                    });
                }
            });
        })
    });
}

fn bench_stats_reset(c: &mut Criterion) {
    let stats = Stats::new(None);
    
    c.bench_function("stats_reset", |b| {
        // Populate before each iteration
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;
            
            for _ in 0..iters {
                // Setup
                for _ in 0..1000 {
                    stats.increment_sent(64, "UDP");
                    stats.increment_failed();
                }
                
                // Measure only the reset
                let start = std::time::Instant::now();
                stats.reset();
                total += start.elapsed();
            }
            
            total
        });
    });
}

criterion_group!(
    benches,
    bench_stats_increment,
    bench_stats_read,
    bench_concurrent_stats,
    bench_stats_reset
);
criterion_main!(benches);