//! Simulation module tests
//!
//! Tests for the simulation orchestration logic and lifecycle management.

use router_flood::config::default_config;
use router_flood::core::simulation::{Simulation, setup_network_interface};
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_simulation_creation() {
    let config = default_config();
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    let _simulation = Simulation::new(config, target_ip, None);
    // If we get here without panic, the simulation was created successfully
}

#[test]
fn test_simulation_creation_with_interface() {
    let config = default_config();
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    // Test with None interface (should work fine)
    let _simulation = Simulation::new(config.clone(), target_ip, None);
    
    // Note: We can't easily test with a real interface in unit tests
    // as it would require system networking capabilities
}

#[test]
fn test_simulation_configuration_scenarios() {
    // Test with various configuration scenarios
    let mut config = default_config();
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    // Test dry-run configuration
    config.safety.dry_run = true;
    let _simulation = Simulation::new(config.clone(), target_ip, None);
    
    // Test with export enabled
    config.export.enabled = true;
    let _simulation = Simulation::new(config.clone(), target_ip, None);
    
    // Test with monitoring disabled
    config.monitoring.system_monitoring = false;
    let _simulation = Simulation::new(config, target_ip, None);
}

#[test]
fn test_simulation_with_different_ips() {
    let config = default_config();
    
    let test_ips = vec![
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)),
    ];
    
    for ip in test_ips {
        let _simulation = Simulation::new(config.clone(), ip, None);
        // Successful creation implies the simulation accepts the IP
    }
}

#[test]
fn test_network_interface_setup() {
    let mut config = default_config();
    
    // Test with no interface specified (should try to find default)
    config.target.interface = None;
    let result = setup_network_interface(&config);
    // Should either find an interface or return None - both are valid
    assert!(result.is_ok());
    
    // Test with non-existent interface
    config.target.interface = Some("definitely_nonexistent_interface".to_string());
    let result = setup_network_interface(&config);
    assert!(result.is_err()); // Should fail to find the interface
}

#[test]
fn test_configuration_audit_flags() {
    let mut config = default_config();
    config.safety.audit_logging = true;
    config.safety.dry_run = true;
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let _simulation = Simulation::new(config, target_ip, None);
    
    // If we reach here, the simulation was created with audit logging enabled
    // The actual audit logging would be tested in integration tests
}

#[test]
fn test_export_configuration() {
    let mut config = default_config();
    config.export.enabled = true;
    config.export.format = router_flood::config::ExportFormat::Json;
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let _simulation = Simulation::new(config, target_ip, None);
    
    // Simulation should handle export configuration properly
}

#[test]
fn test_monitoring_configuration() {
    let mut config = default_config();
    config.monitoring.system_monitoring = true;
    config.monitoring.stats_interval = 1; // Very frequent for testing
    config.monitoring.export_interval = Some(30);
    
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let _simulation = Simulation::new(config, target_ip, None);
    
    // Should accept monitoring configuration
}