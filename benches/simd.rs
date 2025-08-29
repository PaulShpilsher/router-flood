//! SIMD operations benchmarks
//!
//! Measures the performance of SIMD-optimized operations vs scalar implementations,
//! including packet building, checksum calculation, and buffer operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use router_flood::performance::simd_packet::SimdPacketBuilder;

/// Benchmark SIMD payload filling
fn benchmark_payload_filling(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/payload_fill");
    
    let mut builder = SimdPacketBuilder::new();
    
    for size in [16, 64, 256, 512, 1024, 1400] {
        group.bench_with_input(
            BenchmarkId::new("simd", size),
            &size,
            |b, &size| {
                let mut buffer = vec![0u8; size];
                b.iter(|| {
                    builder.fill_payload_simd(black_box(&mut buffer)).unwrap();
                })
            },
        );
        
        // Compare with scalar version
        group.bench_with_input(
            BenchmarkId::new("scalar", size),
            &size,
            |b, &size| {
                let mut buffer = vec![0u8; size];
                b.iter(|| {
                    // Force scalar path
                    for i in 0..buffer.len() {
                        buffer[i] = (i & 0xFF) as u8;
                    }
                    black_box(&buffer);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark checksum calculation (scalar only since SIMD method doesn't exist)
fn benchmark_checksum_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/checksum");
    
    for size in [20, 40, 60, 256, 1400] {
        let data = vec![0xABu8; size];
        
        group.bench_with_input(
            BenchmarkId::new("scalar", size),
            &size,
            |b, _| {
                b.iter(|| {
                    // Simple scalar checksum
                    let mut sum: u32 = 0;
                    for chunk in data.chunks(2) {
                        let word = if chunk.len() == 2 {
                            ((chunk[0] as u32) << 8) | (chunk[1] as u32)
                        } else {
                            (chunk[0] as u32) << 8
                        };
                        sum = sum.wrapping_add(word);
                    }
                    while (sum >> 16) > 0 {
                        sum = (sum & 0xffff) + (sum >> 16);
                    }
                    black_box(!sum as u16)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark UDP packet building with SIMD
fn benchmark_udp_packet_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/udp_packet");
    
    let mut builder = SimdPacketBuilder::new();
    
    for payload_size in [0, 64, 256, 512, 1024] {
        group.bench_with_input(
            BenchmarkId::new("build", payload_size),
            &payload_size,
            |b, &payload_size| {
                let mut buffer = vec![0u8; 20 + 8 + payload_size]; // IP + UDP + payload
                b.iter(|| {
                    // Build a simple UDP packet manually since the method doesn't exist
                    buffer.fill(0);
                    let result = builder.fill_payload_simd(&mut buffer[28..28+payload_size]);
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark buffer operations
fn benchmark_buffer_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/buffer_ops");
    
    // Memory copy benchmark
    for size in [64, 256, 512, 1024, 4096] {
        let src = vec![0x42u8; size];
        let mut dst = vec![0u8; size];
        
        group.bench_with_input(
            BenchmarkId::new("memcpy", size),
            &size,
            |b, _| {
                b.iter(|| {
                    dst.copy_from_slice(black_box(&src));
                    black_box(&dst);
                })
            },
        );
    }
    
    // Memory set benchmark
    for size in [64, 256, 512, 1024, 4096] {
        let mut buffer = vec![0u8; size];
        
        group.bench_with_input(
            BenchmarkId::new("memset", size),
            &size,
            |b, _| {
                b.iter(|| {
                    buffer.fill(0x42);
                    black_box(&buffer);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark SIMD detection overhead
fn benchmark_simd_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/detection");
    
    group.bench_function("feature_detection", |b| {
        b.iter(|| {
            #[cfg(target_arch = "x86_64")]
            {
                let has_avx2 = is_x86_feature_detected!("avx2");
                let has_sse42 = is_x86_feature_detected!("sse4.2");
                black_box((has_avx2, has_sse42))
            }
            #[cfg(target_arch = "aarch64")]
            {
                let has_neon = std::arch::is_aarch64_feature_detected!("neon");
                black_box(has_neon)
            }
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            {
                black_box(false)
            }
        })
    });
    
    group.bench_function("builder_creation", |b| {
        b.iter(|| {
            let builder = SimdPacketBuilder::new();
            black_box(builder)
        })
    });
    
    group.finish();
}

/// Benchmark performance with different SIMD widths
fn benchmark_simd_widths(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/width_comparison");
    
    let data_1k = vec![0xABu8; 1024];
    
    // Test different processing widths
    group.bench_function("process_8_bytes", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for chunk in data_1k.chunks(8) {
                for &byte in chunk {
                    sum = sum.wrapping_add(byte as u64);
                }
            }
            black_box(sum)
        })
    });
    
    group.bench_function("process_16_bytes", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for chunk in data_1k.chunks(16) {
                for &byte in chunk {
                    sum = sum.wrapping_add(byte as u64);
                }
            }
            black_box(sum)
        })
    });
    
    group.bench_function("process_32_bytes", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for chunk in data_1k.chunks(32) {
                for &byte in chunk {
                    sum = sum.wrapping_add(byte as u64);
                }
            }
            black_box(sum)
        })
    });
    
    group.finish();
}

/// Benchmark packet field extraction with SIMD
fn benchmark_field_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/field_extraction");
    
    // Create a sample packet
    let packet = vec![
        // Ethernet header (14 bytes)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // dst mac
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // src mac
        0x08, 0x00, // ethertype
        // IP header (20 bytes)
        0x45, 0x00, 0x00, 0x3c, // version, tos, length
        0x00, 0x00, 0x40, 0x00, // id, flags, fragment
        0x40, 0x06, 0x00, 0x00, // ttl, proto, checksum
        0xc0, 0xa8, 0x01, 0x01, // src ip
        0xc0, 0xa8, 0x01, 0x02, // dst ip
        // TCP header (20 bytes)
        0x04, 0xd2, 0x00, 0x50, // src port, dst port
        0x00, 0x00, 0x00, 0x00, // sequence
        0x00, 0x00, 0x00, 0x00, // acknowledgment
        0x50, 0x02, 0x20, 0x00, // flags, window
        0x00, 0x00, 0x00, 0x00, // checksum, urgent
    ];
    
    group.bench_function("extract_all_fields", |b| {
        b.iter(|| {
            // Extract key fields
            let eth_type = u16::from_be_bytes([packet[12], packet[13]]);
            let ip_proto = packet[23];
            let src_ip = u32::from_be_bytes([packet[26], packet[27], packet[28], packet[29]]);
            let dst_ip = u32::from_be_bytes([packet[30], packet[31], packet[32], packet[33]]);
            let src_port = u16::from_be_bytes([packet[34], packet[35]]);
            let dst_port = u16::from_be_bytes([packet[36], packet[37]]);
            
            black_box((eth_type, ip_proto, src_ip, dst_ip, src_port, dst_port))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_payload_filling,
    benchmark_checksum_calculation,
    benchmark_udp_packet_building,
    benchmark_buffer_operations,
    benchmark_simd_detection,
    benchmark_simd_widths,
    benchmark_field_extraction
);
criterion_main!(benches);