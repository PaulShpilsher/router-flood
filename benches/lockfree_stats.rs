//! Benchmarks for lock-free statistics performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::stats::lockfree::{LockFreeStats, LockFreeLocalStats, PerCpuStats, ProtocolId};
use router_flood::stats::{Stats, BatchStats};
use std::sync::Arc;

fn benchmark_lockfree_vs_traditional(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats_comparison");
    
    // Benchmark lock-free stats
    let lockfree_stats = Arc::new(LockFreeStats::new());
    
    group.bench_function("lockfree_increment", |b| {
        b.iter(|| {
            lockfree_stats.increment_sent(black_box(100), black_box(ProtocolId::Udp));
            lockfree_stats.increment_failed();
        });
    });
    
    // Benchmark traditional FloodStats
    let flood_stats = Arc::new(Stats::default());
    
    group.bench_function("traditional_increment", |b| {
        b.iter(|| {
            flood_stats.increment_sent(black_box(100), black_box("UDP"));
            flood_stats.increment_failed();
        });
    });
    
    // Benchmark lock-free with local batching
    let lockfree_for_local = Arc::new(LockFreeStats::new());
    let mut local_stats = LockFreeLocalStats::new(lockfree_for_local.clone(), 100);
    
    group.bench_function("lockfree_batched_increment", |b| {
        b.iter(|| {
            local_stats.increment_sent(black_box(100), black_box(ProtocolId::Udp));
            local_stats.increment_failed();
        });
    });
    
    // Benchmark traditional with local batching
    let flood_for_local = Arc::new(Stats::default());
    let mut traditional_local = BatchStats::new(flood_for_local.clone(), 100);
    
    group.bench_function("traditional_batched_increment", |b| {
        b.iter(|| {
            traditional_local.increment_sent(black_box(100), black_box("UDP"));
            traditional_local.increment_failed();
        });
    });
    
    group.finish();
}

fn benchmark_per_cpu_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_cpu_stats");
    
    let per_cpu = Arc::new(PerCpuStats::new());
    
    // Benchmark getting local stats
    group.bench_function("get_local_stats", |b| {
        b.iter(|| {
            black_box(per_cpu.get_local());
        });
    });
    
    // Benchmark operations on local stats
    group.bench_function("local_operations", |b| {
        let local = per_cpu.get_local();
        b.iter(|| {
            local.increment_sent(black_box(100), black_box(ProtocolId::Tcp));
        });
    });
    
    // Benchmark aggregation
    group.bench_function("aggregate_stats", |b| {
        // Pre-populate with some data
        for _ in 0..100 {
            let local = per_cpu.get_local();
            local.increment_sent(100, ProtocolId::Udp);
        }
        
        b.iter(|| {
            black_box(per_cpu.aggregate());
        });
    });
    
    group.finish();
}

fn benchmark_snapshot_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_operations");
    
    let stats = LockFreeStats::new();
    
    // Add some data
    for i in 0..1000 {
        stats.increment_sent(100, match i % 5 {
            0 => ProtocolId::Udp,
            1 => ProtocolId::Tcp,
            2 => ProtocolId::Icmp,
            3 => ProtocolId::Ipv6,
            _ => ProtocolId::Arp,
        });
    }
    
    group.bench_function("create_snapshot", |b| {
        b.iter(|| {
            black_box(stats.snapshot());
        });
    });
    
    let snapshot = stats.snapshot();
    
    group.bench_function("calculate_rates", |b| {
        b.iter(|| {
            black_box(snapshot.packets_per_second());
            black_box(snapshot.megabits_per_second());
        });
    });
    
    group.finish();
}

fn benchmark_protocol_id_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_conversion");
    
    group.bench_function("string_to_protocol_id", |b| {
        b.iter(|| {
            black_box(ProtocolId::from_str("UDP"));
            black_box(ProtocolId::from_str("TCP"));
            black_box(ProtocolId::from_str("ICMP"));
        });
    });
    
    group.bench_function("protocol_id_to_string", |b| {
        b.iter(|| {
            black_box(ProtocolId::Udp.as_str());
            black_box(ProtocolId::Tcp.as_str());
            black_box(ProtocolId::Icmp.as_str());
        });
    });
    
    group.finish();
}

fn benchmark_concurrent_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_updates");
    
    // Test with different thread counts
    for num_threads in [1, 2, 4, 8] {
        let stats = Arc::new(LockFreeStats::new());
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_threads", num_threads)),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let mut handles = vec![];
                    
                    for _ in 0..num_threads {
                        let stats_clone = stats.clone();
                        let handle = std::thread::spawn(move || {
                            for _ in 0..100 {
                                stats_clone.increment_sent(100, ProtocolId::Udp);
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_lockfree_vs_traditional,
    benchmark_per_cpu_stats,
    benchmark_snapshot_operations,
    benchmark_protocol_id_conversion,
    benchmark_concurrent_updates
);
criterion_main!(benches);