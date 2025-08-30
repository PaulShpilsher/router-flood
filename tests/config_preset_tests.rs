//! Tests for preset configuration system moved from inline tests

use router_flood::config::preset::*;

#[test]
fn test_default_config_is_valid() {
    let config = PresetConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_quick_test_config() {
    let config = PresetConfig::quick_test("192.168.1.1");
    assert!(config.validate().is_ok());
    assert_eq!(config.test.intensity, LoadLevel::Low);
    assert!(config.safety.dry_run);
}

#[test]
fn test_standard_test_config() {
    let config = PresetConfig::standard_test("192.168.1.1", vec![80, 443]);
    assert!(config.validate().is_ok());
    assert_eq!(config.test.intensity, LoadLevel::Medium);
    assert_eq!(config.target.ports, vec![80, 443]);
}

#[test]
fn test_load_level_conversion() {
    assert_eq!(LoadLevel::Low.to_thread_rate(), (2, 50));
    assert_eq!(LoadLevel::Medium.to_thread_rate(), (4, 100));
    assert_eq!(LoadLevel::High.to_thread_rate(), (8, 200));
}

#[test]
fn test_protocol_mix_conversion() {
    let protocols = ProtocolConfig {
        udp: true,
        tcp: true,
        icmp: false,
    };
    
    let mix = protocols.to_protocol_mix();
    assert!(mix.udp_ratio > 0.0);
    assert!(mix.tcp_syn_ratio > 0.0);
    assert_eq!(mix.icmp_ratio, 0.0);
}

#[test]
fn test_validation_errors() {
    let mut config = PresetConfig::default();
    
    // Test empty IP
    config.target.ip = "".to_string();
    assert!(config.validate().is_err());
    
    // Test invalid duration
    config.target.ip = "192.168.1.1".to_string();
    config.test.duration = 0;
    assert!(config.validate().is_err());
    
    // Test no protocols enabled
    config.test.duration = 30;
    config.test.protocols = ProtocolConfig {
        udp: false,
        tcp: false,
        icmp: false,
    };
    assert!(config.validate().is_err());
}