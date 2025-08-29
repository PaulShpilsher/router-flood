//! Benchmarks for RAII guard performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use router_flood::utils::raii::ChannelGuard;
use router_flood::transport::WorkerChannels;

fn benchmark_guard_creation_drop(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_lifecycle");
    
    // Benchmark ChannelGuard creation and drop
    group.bench_function("channel_guard_lifecycle", |b| {
        b.iter(|| {
            let channels = WorkerChannels {
                ipv4_sender: None,
                ipv6_sender: None,
                l2_sender: None,
            };
            let guard = ChannelGuard::new(channels, "bench");
            black_box(guard); // Forces guard to be created and then dropped
        });
    });
    
    // Skip StatsGuard benchmark as it requires tokio runtime
    // which would add overhead to the benchmark
    
    // Benchmark manual vs RAII cleanup
    group.bench_function("manual_cleanup", |b| {
        b.iter(|| {
            let channels = WorkerChannels {
                ipv4_sender: None,
                ipv6_sender: None,
                l2_sender: None,
            };
            // Manual drop
            drop(channels);
        });
    });
    
    group.bench_function("raii_cleanup", |b| {
        b.iter(|| {
            let channels = WorkerChannels {
                ipv4_sender: None,
                ipv6_sender: None,
                l2_sender: None,
            };
            let _guard = ChannelGuard::new(channels, "bench");
            // RAII drop happens automatically
        });
    });
    
    group.finish();
}

fn benchmark_guard_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_operations");
    
    // Benchmark accessing resources through guards
    let channels = WorkerChannels {
        ipv4_sender: None,
        ipv6_sender: None,
        l2_sender: None,
    };
    let mut channel_guard = ChannelGuard::new(channels, "bench");
    
    group.bench_function("channel_guard_access", |b| {
        b.iter(|| {
            black_box(channel_guard.channels_mut());
        });
    });
    
    // Skip stats guard access benchmark (requires tokio)
    
    group.finish();
}

fn benchmark_nested_guards(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_guards");
    
    // Benchmark nested channel guard creation/destruction
    group.bench_function("nested_guard_lifecycle", |b| {
        b.iter(|| {
            let channels1 = WorkerChannels {
                ipv4_sender: None,
                ipv6_sender: None,
                l2_sender: None,
            };
            let guard1 = ChannelGuard::new(channels1, "outer");
            {
                let channels2 = WorkerChannels {
                    ipv4_sender: None,
                    ipv6_sender: None,
                    l2_sender: None,
                };
                let guard2 = ChannelGuard::new(channels2, "inner");
                black_box(&guard2);
            }
            black_box(&guard1);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_guard_creation_drop,
    benchmark_guard_operations,
    benchmark_nested_guards
);
criterion_main!(benches);