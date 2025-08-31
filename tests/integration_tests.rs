//! Integration tests for major workflows

use router_flood::security::validation::{validate_target_ip, validate_comprehensive_security};
use router_flood::config::Config;
use router_flood::stats::Stats;
use std::net::IpAddr;

#[test]
fn test_security_validation_workflow() {
    // Test complete security validation pipeline
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    let ports = vec![8080, 8081];
    let threads = 4;
    let rate = 1000;
    
    // First validate IP
    let ip_result = validate_target_ip(&ip);
    assert!(ip_result.is_ok());
    
    // Then comprehensive validation
    let comprehensive_result = validate_comprehensive_security(&ip, &ports, threads, rate);
    assert!(comprehensive_result.is_ok());
}

#[test]
fn test_config_with_validation() {
    let mut config = Config::default();
    config.target.ip = "10.0.0.1".to_string();
    config.target.ports = vec![80];
    config.attack.threads = 2;
    config.safety.dry_run = true;
    
    let ip: IpAddr = config.target.ip.parse().unwrap();
    let result = validate_target_ip(&ip);
    assert!(result.is_ok());
}

#[test]
fn test_stats_workflow() {
    let stats = Stats::new(None);
    
    // Simulate packet sending
    for _ in 0..100 {
        stats.increment_sent(64, "UDP");
    }
    
    for _ in 0..10 {
        stats.increment_failed();
    }
    
    assert_eq!(stats.packets_sent(), 100);
    assert_eq!(stats.packets_failed(), 10);
    assert_eq!(stats.bytes_sent(), 6400);
    
    // Test reset
    stats.reset();
    assert_eq!(stats.packets_sent(), 0);
    assert_eq!(stats.packets_failed(), 0);
    assert_eq!(stats.bytes_sent(), 0);
}