//! Benchmarks for configuration validation performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use router_flood::config::ConfigBuilder;

fn benchmark_config_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_validation");
    
    // Benchmark valid configuration building
    group.bench_function("valid_config_build", |b| {
        b.iter(|| {
            black_box(
                ConfigBuilder::new()
                    .target_ip("192.168.1.1")
                    .target_ports(vec![80, 443, 8080])
                    .threads(4)
                    .packet_rate(100)
                    .packet_size_range(64, 1400)
                    .build()
            )
        });
    });
    
    // Benchmark invalid configuration detection
    group.bench_function("invalid_config_detection", |b| {
        b.iter(|| {
            black_box(
                ConfigBuilder::new()
                    .target_ip("8.8.8.8") // Invalid public IP
                    .threads(200) // Exceeds limit
                    .packet_rate(50000) // Exceeds limit
                    .build()
            )
        });
    });
    
    // Benchmark protocol mix validation
    group.bench_function("protocol_mix_validation", |b| {
        let invalid_mix = router_flood::config_original::ProtocolMix {
            udp_ratio: 0.5,
            tcp_syn_ratio: 0.3,
            tcp_ack_ratio: 0.3, // Total > 1.0
            icmp_ratio: 0.1,
            ipv6_ratio: 0.0,
            arp_ratio: 0.0,
        };
        
        b.iter(|| {
            black_box(
                ConfigBuilder::new()
                    .protocol_mix(invalid_mix.clone())
                    .build()
            )
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_config_validation);
criterion_main!(benches);