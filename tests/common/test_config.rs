//! Test configuration builders and utilities

use router_flood::config::Config;

/// Create a minimal valid configuration for testing
pub fn minimal_config() -> Config {
    let mut config = Config::default();
    config.target.ip = "192.168.1.1".to_string();
    config.target.ports = vec![8080];
    config.attack.threads = 1;
    config.attack.packet_rate = 10.0;
    config.safety.dry_run = true;
    config
}

/// Create a configuration with specified target IP
pub fn config_with_ip(ip: &str) -> Config {
    let mut config = minimal_config();
    config.target.ip = ip.to_string();
    config
}

/// Create a configuration for performance testing
pub fn performance_config() -> Config {
    let mut config = minimal_config();
    config.attack.threads = 4;
    config.attack.packet_rate = 1000.0;
    config.attack.payload_size = 1400;
    config.attack.duration = Some(10);
    config
}

/// Create a configuration for security testing
pub fn security_test_config() -> Config {
    let mut config = minimal_config();
    config.safety.dry_run = true;
    config.safety.rate_limit = true;
    config.safety.max_bandwidth_mbps = Some(1.0);
    config
}