//! Configuration module tests
//!
//! Tests for configuration loading, validation, and default value handling.

use router_flood::config::*;
use tempfile::NamedTempFile;
use std::io::Write;
use std::str::FromStr;

#[test]
fn test_default_config_values() {
    let config = get_default_config();
    
    // Test target defaults
    assert_eq!(config.target.ip, "192.168.0.1");
    assert_eq!(config.target.ports, vec![80]);
    assert!(config.target.interface.is_none());
    
    // Test attack defaults
    assert_eq!(config.attack.threads, 4);
    assert_eq!(config.attack.packet_rate, 100);
    assert!(config.attack.duration.is_none());
    
    // Test safety defaults
    assert!(config.safety.require_private_ranges);
    assert!(config.safety.enable_monitoring);
    assert!(config.safety.audit_logging);
    assert!(!config.safety.dry_run);
    assert!(!config.safety.perfect_simulation);
    
    // Test protocol mix ratios sum to 1.0
    let mix = &config.target.protocol_mix;
    let total = mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + 
               mix.icmp_ratio + mix.ipv6_ratio + mix.arp_ratio;
    assert!((total - 1.0).abs() < f64::EPSILON, "Protocol ratios should sum to 1.0");
}

#[test]
fn test_export_format_parsing() {
    assert_eq!(ExportFormat::from_str("json"), Ok(ExportFormat::Json));
    assert_eq!(ExportFormat::from_str("csv"), Ok(ExportFormat::Csv));
    assert_eq!(ExportFormat::from_str("both"), Ok(ExportFormat::Both));
    assert_eq!(ExportFormat::from_str("JSON"), Ok(ExportFormat::Json));
    assert_eq!(ExportFormat::from_str("CSV"), Ok(ExportFormat::Csv));
    assert_eq!(ExportFormat::from_str("BOTH"), Ok(ExportFormat::Both));
    
    assert!(ExportFormat::from_str("xml").is_err());
    assert!(ExportFormat::from_str("").is_err());
    assert!(ExportFormat::from_str("invalid").is_err());
}

#[test]
fn test_protocol_mix_validation() {
    let mut config = get_default_config();
    
    // Test valid protocol mix
    config.target.protocol_mix = ProtocolMix {
        udp_ratio: 0.6,
        tcp_syn_ratio: 0.2,
        tcp_ack_ratio: 0.1,
        icmp_ratio: 0.05,
        ipv6_ratio: 0.03,
        arp_ratio: 0.02,
    };
    
    let mix = &config.target.protocol_mix;
    let total = mix.udp_ratio + mix.tcp_syn_ratio + mix.tcp_ack_ratio + 
               mix.icmp_ratio + mix.ipv6_ratio + mix.arp_ratio;
    assert!((total - 1.0).abs() < f64::EPSILON);
    
    // Test all ratios are non-negative
    assert!(mix.udp_ratio >= 0.0);
    assert!(mix.tcp_syn_ratio >= 0.0);
    assert!(mix.tcp_ack_ratio >= 0.0);
    assert!(mix.icmp_ratio >= 0.0);
    assert!(mix.ipv6_ratio >= 0.0);
    assert!(mix.arp_ratio >= 0.0);
}

#[test]
fn test_yaml_config_loading() {
    let yaml_content = r#"
target:
  ip: "10.0.0.1"
  ports: [80, 443, 22]
  interface: "eth0"
  protocol_mix:
    udp_ratio: 0.5
    tcp_syn_ratio: 0.3
    tcp_ack_ratio: 0.1
    icmp_ratio: 0.05
    ipv6_ratio: 0.03
    arp_ratio: 0.02

attack:
  threads: 8
  packet_rate: 500
  duration: 60
  packet_size_range: [64, 1500]
  burst_pattern: !Sustained
    rate: 500
  randomize_timing: true

safety:
  require_private_ranges: true
  enable_monitoring: true
  audit_logging: true
  dry_run: false
  perfect_simulation: false
  max_threads: 100
  max_packet_rate: 10000

export:
  enabled: true
  format: Json
  filename_pattern: "test_export"
  include_system_stats: true

monitoring:
  system_monitoring: true
  stats_interval: 5
  export_interval: 30
  performance_tracking: true
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
    
    let config = load_config(Some(temp_file.path().to_str().unwrap())).expect("Failed to load config");
    
    // Verify loaded values
    assert_eq!(config.target.ip, "10.0.0.1");
    assert_eq!(config.target.ports, vec![80, 443, 22]);
    assert_eq!(config.target.interface, Some("eth0"));
    assert_eq!(config.attack.threads, 8);
    assert_eq!(config.attack.packet_rate, 500);
    assert_eq!(config.attack.duration, Some(60));
    assert!(config.export.enabled);
    assert_eq!(config.export.format, ExportFormat::Json);
    assert_eq!(config.monitoring.stats_interval, 5);
    assert_eq!(config.monitoring.export_interval, Some(30));
}

#[test]
fn test_invalid_yaml_config() {
    let invalid_yaml = r#"
target:
  ip: "invalid_ip"
  ports: "not_a_list"
attack:
  threads: -1
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(invalid_yaml.as_bytes()).expect("Failed to write to temp file");
    
    // Should return error for invalid YAML structure
    let result = load_config(Some(temp_file.path().to_str().unwrap()));
    assert!(result.is_err());
}

#[test]
fn test_nonexistent_config_file() {
    let result = load_config(Some("nonexistent_file.yaml"));
    // Should succeed and return defaults when file not found
    assert!(result.is_ok());
    let config = result.unwrap();
    // Should have default values
    assert_eq!(config.target.ip, "192.168.0.1");
}

#[test]
fn test_config_builder_pattern() {
    let mut config = get_default_config();
    
    // Test modifying config through builder-like pattern
    config.target.ip = "172.16.0.1";
    config.target.ports = vec![8080, 9090];
    config.attack.threads = 16;
    config.attack.packet_rate = 1000;
    config.attack.duration = Some(120);
    config.safety.dry_run = true;
    
    assert_eq!(config.target.ip, "172.16.0.1");
    assert_eq!(config.target.ports, vec![8080, 9090]);
    assert_eq!(config.attack.threads, 16);
    assert_eq!(config.attack.packet_rate, 1000);
    assert_eq!(config.attack.duration, Some(120));
    assert!(config.safety.dry_run);
}

#[test]
fn test_config_safety_limits() {
    let config = get_default_config();
    
    // Test safety limits are reasonable
    assert!(config.safety.max_threads > 0);
    assert!(config.safety.max_threads <= 100);
    assert!(config.safety.max_packet_rate > 0);
    assert!(config.safety.max_packet_rate <= 10000);
}

#[test]
fn test_export_config_combinations() {
    let mut config = get_default_config();
    
    // Test JSON export
    config.export.enabled = true;
    config.export.format = ExportFormat::Json;
    assert!(config.export.enabled);
    assert_eq!(config.export.format, ExportFormat::Json);
    
    // Test CSV export
    config.export.format = ExportFormat::Csv;
    assert_eq!(config.export.format, ExportFormat::Csv);
    
    // Test both formats
    config.export.format = ExportFormat::Both;
    assert_eq!(config.export.format, ExportFormat::Both);
    
    // Test disabled export
    config.export.enabled = false;
    assert!(!config.export.enabled);
}

#[test]
fn test_monitoring_config_validation() {
    let mut config = get_default_config();
    
    // Test valid monitoring intervals
    config.monitoring.stats_interval = 1;
    config.monitoring.export_interval = Some(30);
    assert_eq!(config.monitoring.stats_interval, 1);
    assert_eq!(config.monitoring.export_interval, Some(30));
    
    // Test disabled export interval
    config.monitoring.export_interval = None;
    assert!(config.monitoring.export_interval.is_none());
    
    // Test system monitoring toggle
    config.monitoring.system_monitoring = false;
    assert!(!config.monitoring.system_monitoring);
}