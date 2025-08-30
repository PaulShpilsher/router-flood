//! Error handling tests

#![allow(clippy::uninlined_format_args)]
//!
//! Tests for custom error types, error conversion, and error propagation.

use router_flood::error::*;
use std::io;

#[test]
fn test_config_error_creation() {
    let error = ConfigError::InvalidValue {
        field: "threads".to_string(),
        value: "abc".to_string(),
        reason: "must be a number".to_string(),
    };
    
    assert_eq!(format!("{}", error), "Invalid value 'abc' for field 'threads': must be a number");
}

#[test]
fn test_config_error_file_not_found() {
    let error = ConfigError::FileNotFound("config.yaml".to_string());
    assert_eq!(format!("{}", error), "Configuration file not found: config.yaml");
}

#[test]
fn test_config_error_parse_error() {
    let error = ConfigError::ParseError("invalid YAML syntax".to_string());
    assert_eq!(format!("{}", error), "Failed to parse configuration: invalid YAML syntax");
}

#[test]
fn test_validation_error_creation() {
    let error = ValidationError::InvalidIpRange {
        ip: "8.8.8.8".to_string(),
        reason: "not in private range",
    };
    
    assert_eq!(format!("{}", error), "IP address 8.8.8.8 is invalid: not in private range");
}

#[test]
fn test_validation_error_port_range() {
    let error = ValidationError::ExceedsLimit {
        field: "ports",
        value: 70000,
        limit: 65535,
    };
    
    assert!(format!("{}", error).contains("port"));
}

#[test]
fn test_validation_error_resource_limit() {
    let error = ValidationError::SystemRequirement("Insufficient file descriptors");
    
    assert!(format!("{}", error).contains("file descriptors"));
}

#[test]
fn test_validation_error_permission_denied() {
    let error = ValidationError::PrivilegeRequired("root privileges required");
    assert!(format!("{}", error).contains("root privileges"));
}

#[test]
fn test_network_error_creation() {
    let error = NetworkError::InterfaceNotFound("eth0".to_string());
    assert_eq!(format!("{}", error), "Network interface not found: eth0");
}

#[test]
fn test_network_error_channel_creation() {
    let error = NetworkError::ChannelCreation("failed to create raw socket".to_string());
    assert_eq!(format!("{}", error), "Failed to create network channel: failed to create raw socket");
}

#[test]
fn test_network_error_packet_send() {
    let error = NetworkError::PacketSend("network unreachable".to_string());
    assert_eq!(format!("{}", error), "Failed to send packet: network unreachable");
}

#[test]
fn test_router_flood_error_from_config_error() {
    let config_error = ConfigError::InvalidValue {
        field: "rate".to_string(),
        value: "invalid".to_string(),
        reason: "must be numeric".to_string(),
    };
    
    let router_error: RouterFloodError = config_error.into();
    
    match router_error {
        RouterFloodError::Config(_) => {
            // Expected conversion
        }
        _ => panic!("Expected Config error variant"),
    }
}

#[test]
fn test_router_flood_error_from_validation_error() {
    let validation_error = ValidationError::InvalidIpRange {
        ip: "127.0.0.1".to_string(),
        reason: "loopback not allowed",
    };
    
    let router_error: RouterFloodError = validation_error.into();
    
    match router_error {
        RouterFloodError::Validation(_) => {
            // Expected conversion
        }
        _ => panic!("Expected Validation error variant"),
    }
}

#[test]
fn test_router_flood_error_from_network_error() {
    let network_error = NetworkError::PacketSend("connection timeout".to_string());
    
    let router_error: RouterFloodError = network_error.into();
    
    match router_error {
        RouterFloodError::Network(_) => {
            // Expected conversion
        }
        _ => panic!("Expected Network error variant"),
    }
}

#[test]
fn test_router_flood_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    
    let router_error: RouterFloodError = io_error.into();
    
    match router_error {
        RouterFloodError::Io(_) => {
            // Expected conversion
        }
        _ => panic!("Expected IO error variant"),
    }
}

#[test]
fn test_error_display_implementation() {
    let errors = vec![
        RouterFloodError::Config(ConfigError::FileNotFound("test.yaml".to_string())),
        RouterFloodError::Validation(ValidationError::PermissionDenied("need root")),
        RouterFloodError::Network(NetworkError::InterfaceNotFound("wlan0".to_string())),
        RouterFloodError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found")),
    ];
    
    for error in errors {
        let display_string = format!("{}", error);
        assert!(!display_string.is_empty(), "Error display should not be empty");
        assert!(display_string.len() > 5, "Error display should be descriptive");
    }
}

#[test]
fn test_error_debug_implementation() {
    let error = RouterFloodError::Config(ConfigError::InvalidValue {
        field: "threads".to_string(),
        value: "abc".to_string(),
        reason: "must be numeric".to_string(),
    });
    
    let debug_string = format!("{:?}", error);
    assert!(!debug_string.is_empty());
    assert!(debug_string.contains("Config"));
    assert!(debug_string.contains("InvalidValue"));
}

#[test]
fn test_error_chain_propagation() {
    // Test that errors can be chained and propagated correctly
    fn inner_function() -> Result<()> {
        Err(ConfigError::FileNotFound("inner.yaml".to_string()).into())
    }
    
    fn middle_function() -> Result<()> {
        inner_function()?;
        Ok(())
    }
    
    fn outer_function() -> Result<()> {
        middle_function()?;
        Ok(())
    }
    
    let result = outer_function();
    assert!(result.is_err());
    
    match result {
        Err(RouterFloodError::Config(ConfigError::FileNotFound(filename))) => {
            assert_eq!(filename, "inner.yaml");
        }
        _ => panic!("Error was not propagated correctly"),
    }
}

#[test]
fn test_error_context_preservation() {
    // Test that error context is preserved through conversions
    let original_message = "original error message";
    let config_error = ConfigError::ParseError(original_message.to_string());
    let router_error: RouterFloodError = config_error.into();
    
    let error_string = format!("{}", router_error);
    assert!(error_string.contains(original_message), 
           "Original error message should be preserved: '{}'", error_string);
}

#[test]
fn test_result_type_alias() {
    // Test that our Result type alias works correctly
    fn test_function() -> Result<i32> {
        Ok(42)
    }
    
    let result = test_function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    fn test_error_function() -> Result<i32> {
        Err(ConfigError::FileNotFound("test".to_string()).into())
    }
    
    let error_result = test_error_function();
    assert!(error_result.is_err());
}

#[test]
fn test_error_equality() {
    // Test error equality where applicable
    let error1 = ConfigError::FileNotFound("test.yaml".to_string());
    let error2 = ConfigError::FileNotFound("test.yaml".to_string());
    let error3 = ConfigError::FileNotFound("other.yaml".to_string());
    
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_error_categorization() {
    // Test that we can categorize errors appropriately
    let config_err = RouterFloodError::Config(ConfigError::FileNotFound("test".to_string()));
    let validation_err = RouterFloodError::Validation(ValidationError::PermissionDenied("test"));
    let network_err = RouterFloodError::Network(NetworkError::InterfaceNotFound("test".to_string()));
    let io_err = RouterFloodError::Io(io::Error::new(io::ErrorKind::NotFound, "test".to_string()));
    
    // Test pattern matching
    match config_err {
        RouterFloodError::Config(_) => { /* expected */ }
        _ => panic!("Should be Config error"),
    }
    
    match validation_err {
        RouterFloodError::Validation(_) => { /* expected */ }
        _ => panic!("Should be Validation error"),
    }
    
    match network_err {
        RouterFloodError::Network(_) => { /* expected */ }
        _ => panic!("Should be Network error"),
    }
    
    match io_err {
        RouterFloodError::Io(_) => { /* expected */ }
        _ => panic!("Should be IO error"),
    }
}