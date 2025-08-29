//! Benchmarks for abstraction layer performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use router_flood::abstractions::{NetworkProvider, SystemProvider};
use router_flood::abstractions::network::PnetProvider;
use router_flood::abstractions::system::DefaultSystemProvider;

fn benchmark_network_provider(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_provider");
    // Reduce sample size for expensive network operations
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let provider = PnetProvider;
    
    // Benchmark interface listing
    group.bench_function("list_interfaces", |b| {
        b.iter(|| {
            black_box(provider.interfaces());
        });
    });
    
    // Benchmark interface lookup by name
    group.bench_function("find_by_name", |b| {
        b.iter(|| {
            black_box(provider.find_by_name("lo"));
        });
    });
    
    // Benchmark default interface selection
    group.bench_function("default_interface", |b| {
        b.iter(|| {
            black_box(provider.default_interface());
        });
    });
    
    group.finish();
}

fn benchmark_system_provider(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_provider");
    
    let provider = DefaultSystemProvider;
    
    // Benchmark privilege checking
    group.bench_function("is_root", |b| {
        b.iter(|| {
            black_box(provider.is_root());
        });
    });
    
    // Benchmark UID retrieval
    group.bench_function("effective_uid", |b| {
        b.iter(|| {
            black_box(provider.effective_uid());
        });
    });
    
    // Benchmark TTY check
    group.bench_function("is_tty", |b| {
        b.iter(|| {
            black_box(provider.is_tty());
        });
    });
    
    // Benchmark CPU count
    group.bench_function("cpu_count", |b| {
        b.iter(|| {
            black_box(provider.cpu_count());
        });
    });
    
    group.finish();
}

fn benchmark_abstraction_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("abstraction_overhead");
    // Reduce sample size for operations that make system calls
    group.sample_size(50);
    
    // Direct system call
    group.bench_function("direct_geteuid", |b| {
        b.iter(|| {
            unsafe { black_box(libc::geteuid()) }
        });
    });
    
    // Through abstraction
    let provider = DefaultSystemProvider;
    group.bench_function("abstracted_uid", |b| {
        b.iter(|| {
            black_box(provider.effective_uid());
        });
    });
    
    // Direct pnet call
    group.bench_function("direct_pnet_interfaces", |b| {
        b.iter(|| {
            black_box(pnet::datalink::interfaces());
        });
    });
    
    // Through abstraction
    let net_provider = PnetProvider;
    group.bench_function("abstracted_interfaces", |b| {
        b.iter(|| {
            black_box(net_provider.interfaces());
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_network_provider,
    benchmark_system_provider,
    benchmark_abstraction_overhead
);
criterion_main!(benches);