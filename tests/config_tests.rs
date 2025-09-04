//! Configuration validation unit tests

use router_flood::config::{Config, ProtocolMix};

#[test]
fn test_config_default() {
    let config = Config::default();
    assert!(!config.target.ip.is_empty());
    assert!(!config.target.ports.is_empty());
}

#[test]
fn test_protocol_mix_default() {
    let mix = ProtocolMix::default();
    assert_eq!(mix.udp_ratio, 0.25);
    assert_eq!(mix.tcp_syn_ratio, 0.25);
    assert_eq!(mix.tcp_ack_ratio, 0.15);
    assert_eq!(mix.tcp_fin_ratio, 0.15);
    assert_eq!(mix.tcp_rst_ratio, 0.10);
    assert_eq!(mix.icmp_ratio, 0.10);
    assert_eq!(mix.custom_ratio, 0.0);
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();
    
    // Test valid configuration
    config.target.ip = "192.168.1.1".to_string();
    config.target.ports = vec![80, 443];
    config.attack.threads = 4;
    config.attack.packet_rate = 100.0;
    
    assert!(config.attack.threads > 0);
    assert!(config.attack.packet_rate > 0.0);
}

#[test]
fn test_safety_config() {
    let config = Config::default();
    assert!(!config.safety.dry_run);  // Default is false
    assert!(!config.safety.allow_localhost);
    assert!(config.safety.rate_limit);  // Default is true
    assert!(config.safety.require_confirmation);  // Default is true
}

#[test]
fn test_monitoring_config() {
    let config = Config::default();
    assert!(config.monitoring.enabled);  // Default is true
    assert!(config.monitoring.interval_ms > 0);
    assert!(!config.monitoring.verbose);  // Default is false
    assert!(config.monitoring.show_stats);  // Default is true
}