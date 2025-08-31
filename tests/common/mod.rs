//! Common test utilities

use router_flood::config::{
    Config, Target, LoadConfig, Safety, 
    Monitoring, Export, ExportFormat, ProtocolMix, BurstPattern
};

/// Create a standard test configuration
pub fn create_test_config() -> Config {
    Config {
        target: Target {
            ip: "192.168.1.1".to_string(),
            ports: vec![80, 443],
            protocol_mix: ProtocolMix {
                udp_ratio: 1.0,
                tcp_syn_ratio: 0.0,
                tcp_ack_ratio: 0.0,
                icmp_ratio: 0.0,
                ipv6_ratio: 0.0,
                arp_ratio: 0.0,
            },
            interface: None,
        },
        attack: LoadConfig {
            threads: 2,
            packet_rate: 100,
            duration: Some(5),
            packet_size_range: (64, 1400),
            burst_pattern: BurstPattern::Sustained { rate: 100 },
            randomize_timing: false,
        },
        safety: Safety {
            max_threads: 10,
            max_packet_rate: 10000,
            require_private_ranges: false,
            enable_monitoring: false,
            audit_logging: false,
            dry_run: true,
            perfect_simulation: true,
        },
        monitoring: Monitoring {
            stats_interval: 5,
            system_monitoring: false,
            export_interval: None,
            performance_tracking: false,
        },
        export: Export {
            enabled: false,
            format: ExportFormat::Json,
            filename_pattern: "test".to_string(),
            include_system_stats: false,
        },
    }
}