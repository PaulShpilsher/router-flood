//! Configuration loading and validation integration tests

use router_flood::config::{Config, load_config, validate_config};
use router_flood::security::validation::validate_target_ip;
use std::net::IpAddr;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_default_config_validation() {
    let config = Config::default();
    
    // Default config should be valid
    let validation_result = validate_config(&config);
    assert!(validation_result.is_ok());
    
    // Default IP should be private
    let ip: IpAddr = config.target.ip.parse().unwrap();
    assert!(validate_target_ip(&ip).is_ok());
    
    // Default ports should be set
    assert!(!config.target.ports.is_empty());
    assert_eq!(config.target.ports, vec![80, 443]);
}

#[test]
fn test_config_with_custom_values() {
    let mut config = Config::default();
    
    // Modify configuration
    config.target.ip = "10.0.0.1".to_string();
    config.target.ports = vec![8080, 8081, 8082];
    config.attack.threads = 8;
    config.attack.packet_rate = 500.0;
    config.safety.dry_run = true;
    
    // Validate modified config
    let validation_result = validate_config(&config);
    assert!(validation_result.is_ok());
    
    // Validate IP
    let ip: IpAddr = config.target.ip.parse().unwrap();
    assert!(validate_target_ip(&ip).is_ok());
}

#[test]
fn test_config_validation_boundaries() {
    let mut config = Config::default();
    
    // Test thread boundaries
    config.attack.threads = 100; // Maximum
    assert!(validate_config(&config).is_ok());
    
    config.attack.threads = 101; // Over maximum
    assert!(validate_config(&config).is_err());
    
    config.attack.threads = 0; // Minimum
    assert!(validate_config(&config).is_err());
    
    // Test packet rate boundaries
    config.attack.threads = 4; // Reset to valid
    config.attack.packet_rate = 10000.0; // Maximum
    assert!(validate_config(&config).is_ok());
    
    config.attack.packet_rate = 10001.0; // Over maximum
    assert!(validate_config(&config).is_err());
    
    config.attack.packet_rate = 0.0; // Minimum
    assert!(validate_config(&config).is_err());
}

#[test]
fn test_config_from_yaml() {
    // Create a temporary YAML config file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, r#"target:
  ip: "192.168.1.100"
  ports: [8080, 8081]
  protocol_mix:
    udp_ratio: 0.5
    tcp_syn_ratio: 0.3
    tcp_ack_ratio: 0.1
    icmp_ratio: 0.1
    custom_ratio: 0.0
attack:
  threads: 4
  packet_rate: 1000.0
  payload_size: 100
  duration: 10
  burst_mode: false
safety:
  dry_run: true
  perfect_simulation: false
  rate_limit: true
  max_bandwidth_mbps: 100.0
  allow_localhost: false
  require_confirmation: true
monitoring:
  enabled: true
  interval_ms: 1000
  verbose: false
  show_stats: true
export:
  enabled: false
  format: "Json"
  path: "./stats"
  interval_seconds: 60
  include_system_stats: false
audit:
  enabled: false
  log_file: "/tmp/test_audit.log""#).unwrap();
    
    // Flush the file to ensure data is written
    temp_file.flush().unwrap();
    
    // Load and validate the config
    let config_result = load_config(Some(temp_file.path().to_str().unwrap()));
    if let Err(e) = &config_result {
        eprintln!("Config load error: {:?}", e);
    }
    assert!(config_result.is_ok());
    
    let config = config_result.unwrap();
    assert_eq!(config.target.ip, "192.168.1.100");
    assert_eq!(config.target.ports, vec![8080, 8081]);
    assert_eq!(config.attack.threads, 4);
    assert_eq!(config.attack.packet_rate, 1000.0);
}

#[test]
fn test_invalid_config_rejection() {
    let mut config = Config::default();
    
    // Invalid payload size
    config.attack.payload_size = 0;
    assert!(validate_config(&config).is_err());
    
    config.attack.payload_size = 10000; // Too large
    assert!(validate_config(&config).is_err());
    
    // Reset to valid
    config.attack.payload_size = 100;
    
    // Invalid thread count
    config.attack.threads = 200;
    assert!(validate_config(&config).is_err());
}

#[test]
fn test_config_safety_defaults() {
    let config = Config::default();
    
    // Safety defaults should be secure
    assert!(!config.safety.dry_run); // Default is false
    assert!(config.safety.rate_limit); // Rate limiting enabled by default
    assert!(!config.safety.allow_localhost); // Localhost not allowed by default
    assert!(config.safety.require_confirmation); // Confirmation required by default
}