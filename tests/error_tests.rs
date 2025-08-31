//! Error handling unit tests

use router_flood::error::{RouterFloodError, ValidationError, PacketError};

#[test]
fn test_validation_error_creation() {
    let error = ValidationError::new("field", "Invalid value");
    let router_error: RouterFloodError = error.into();
    
    match router_error {
        RouterFloodError::Validation(msg) => {
            assert!(msg.contains("Invalid value"));
        }
        _ => panic!("Expected validation error"),
    }
}

#[test]
fn test_packet_error_creation() {
    let error = PacketError::build_failed("UDP", "Invalid size");
    let router_error: RouterFloodError = error.into();
    
    match router_error {
        RouterFloodError::PacketBuild(msg) => {
            assert!(msg.contains("UDP"));
            assert!(msg.contains("Invalid size"));
        }
        _ => panic!("Expected packet error"),
    }
}

#[test]
fn test_error_display() {
    let error = ValidationError::new("ip", "Must be private IP");
    let router_error: RouterFloodError = error.into();
    
    let error_string = format!("{}", router_error);
    assert!(!error_string.is_empty());
}