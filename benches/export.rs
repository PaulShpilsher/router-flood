//! Data export and serialization benchmarks
//!
//! Measures the performance of statistics export, JSON/CSV serialization,
//! and file I/O operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::stats::collector::{SessionStats, SystemStats};
use router_flood::stats::lockfree::LockFreeStats;
use serde_json;
use csv::Writer;
use std::collections::HashMap;
use chrono::Utc;

/// Create sample statistics for benchmarking
fn create_sample_stats(size: usize) -> SessionStats {
    let mut protocol_breakdown = HashMap::new();
    protocol_breakdown.insert("UDP".to_string(), (size as u64) * 600);
    protocol_breakdown.insert("TCP_SYN".to_string(), (size as u64) * 250);
    protocol_breakdown.insert("TCP_ACK".to_string(), (size as u64) * 100);
    protocol_breakdown.insert("ICMP".to_string(), (size as u64) * 50);
    
    SessionStats {
        session_id: format!("session_{}", size),
        timestamp: Utc::now(),
        packets_sent: (size as u64).saturating_mul(1000),
        packets_failed: (size as u64).saturating_mul(10),
        bytes_sent: (size as u64).saturating_mul(1500),
        duration_secs: 60.0,
        packets_per_second: 1000.0,
        megabits_per_second: 12.0,
        protocol_breakdown,
        system_stats: Some(SystemStats {
            cpu_usage: 45.5,
            memory_usage: 67200000,
            memory_total: 134400000,
            network_sent: 1000000,
            network_received: 500000,
        }),
    }
}

/// Benchmark JSON serialization
fn benchmark_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("export/json");
    
    for size in [1, 10, 100, 1000] {
        let stats = create_sample_stats(size);
        
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let json = serde_json::to_string(&stats).unwrap();
                    black_box(json)
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("serialize_pretty", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let json = serde_json::to_string_pretty(&stats).unwrap();
                    black_box(json)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark CSV export
fn benchmark_csv_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("export/csv");
    
    for rows in [10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("write_rows", rows),
            &rows,
            |b, &rows| {
                b.iter(|| {
                    let mut wtr = Writer::from_writer(Vec::new());
                    
                    // Write header
                    wtr.write_record(&["timestamp", "packets_sent", "packets_failed", "bytes_sent", "pps"]).unwrap();
                    
                    // Write data rows
                    for i in 0..rows {
                        wtr.write_record(&[
                            &format!("2024-01-01T00:00:{:02}Z", i % 60),
                            &((i % 1000) * 1000).to_string(),
                            &((i % 1000) * 10).to_string(),
                            &((i % 1000) * 1500).to_string(),
                            "1000.0",
                        ]).unwrap();
                    }
                    
                    let data = wtr.into_inner().unwrap();
                    black_box(data)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark statistics collection
fn benchmark_stats_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("export/collection");
    
    group.bench_function("snapshot_creation", |b| {
        let stats = LockFreeStats::new();
        
        // Populate with some data
        use router_flood::stats::lockfree::ProtocolId;
        for _ in 0..1000 {
            stats.increment_sent(1500, ProtocolId::Udp);
        }
        for _ in 0..10 {
            stats.increment_failed();
        }
        
        b.iter(|| {
            let snapshot = stats.snapshot();
            black_box(snapshot)
        })
    });
    
    group.bench_function("rate_calculation", |b| {
        let stats = LockFreeStats::new();
        use router_flood::stats::lockfree::ProtocolId;
        for _ in 0..10000 {
            stats.increment_sent(1500, ProtocolId::Udp);
        }
        
        b.iter(|| {
            let snapshot = stats.snapshot();
            let rate = snapshot.packets_per_second();
            let mbps = snapshot.megabits_per_second();
            black_box((rate, mbps))
        })
    });
    
    group.finish();
}

/// Benchmark string formatting for export
fn benchmark_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("export/formatting");
    
    let stats = create_sample_stats(100);
    
    group.bench_function("format_display", |b| {
        b.iter(|| {
            let formatted = format!("{:?}", stats);
            black_box(formatted)
        })
    });
    
    group.bench_function("format_custom", |b| {
        b.iter(|| {
            let formatted = format!(
                "Packets: {} | Failed: {} | Rate: {:.2} pps | Mbps: {:.1}",
                stats.packets_sent,
                stats.packets_failed,
                stats.packets_per_second,
                stats.megabits_per_second
            );
            black_box(formatted)
        })
    });
    
    group.bench_function("format_timestamp", |b| {
        use chrono::Utc;
        b.iter(|| {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
            black_box(timestamp)
        })
    });
    
    group.finish();
}

/// Benchmark large data export
fn benchmark_large_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("export/large_data");
    
    // Create a large stats object with history
    let mut large_stats = Vec::new();
    for i in 0..1000 {
        large_stats.push(create_sample_stats(i));
    }
    
    group.bench_function("json_1k_records", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&large_stats).unwrap();
            black_box(json.len())
        })
    });
    
    group.bench_function("json_compressed", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(&large_stats).unwrap();
            // Simulate compression (just measure serialization to bytes)
            black_box(json.len())
        })
    });
    
    group.finish();
}

/// Benchmark concurrent export operations
fn benchmark_concurrent_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("export/concurrent");
    
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|i| {
                            std::thread::spawn(move || {
                                let stats = create_sample_stats(i * 100);
                                let json = serde_json::to_string(&stats).unwrap();
                                json.len()
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

/// Benchmark different export formats
fn benchmark_export_formats(c: &mut Criterion) {
    let mut group = c.benchmark_group("export/formats");
    
    let stats = create_sample_stats(100);
    
    // JSON
    group.bench_function("json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&stats).unwrap();
            black_box(json.len())
        })
    });
    
    // CSV (single row)
    group.bench_function("csv_row", |b| {
        b.iter(|| {
            let mut wtr = Writer::from_writer(Vec::new());
            wtr.write_record(&[
                stats.packets_sent.to_string(),
                stats.packets_failed.to_string(),
                stats.bytes_sent.to_string(),
                stats.packets_per_second.to_string(),
            ]).unwrap();
            let data = wtr.into_inner().unwrap();
            black_box(data.len())
        })
    });
    
    // Binary (using bincode would be ideal, but using serde_json::to_vec as proxy)
    group.bench_function("binary", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(&stats).unwrap();
            black_box(bytes.len())
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_json_serialization,
    benchmark_csv_export,
    benchmark_stats_collection,
    benchmark_formatting,
    benchmark_large_export,
    benchmark_concurrent_export,
    benchmark_export_formats
);
criterion_main!(benches);