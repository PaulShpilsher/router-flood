//! Comprehensive error handling tests

use router_flood::error::{RouterFloodError, ValidationError, PacketError, ConfigError};
use std::io;
use std::net::AddrParseError;

#[test]
fn test_error_display_formatting() {
    let io_err = RouterFloodError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
    assert!(format!("{}", io_err).contains("I/O error"));
    
    let network_err = RouterFloodError::Network("connection failed".to_string());
    assert!(format!("{}", network_err).contains("Network error"));
    
    let config_err = RouterFloodError::Config("invalid config".to_string());
    assert!(format!("{}", config_err).contains("Configuration error"));
    
    let validation_err = RouterFloodError::Validation("invalid input".to_string());
    assert!(format!("{}", validation_err).contains("Validation error"));
    
    let packet_err = RouterFloodError::PacketBuild("packet failed".to_string());
    assert!(format!("{}", packet_err).contains("Packet building error"));
    
    let resource_err = RouterFloodError::SystemResource("out of memory".to_string());
    assert!(format!("{}", resource_err).contains("System resource error"));
    
    let permission_err = RouterFloodError::Permission("access denied".to_string());
    assert!(format!("{}", permission_err).contains("Permission error"));
    
    let stats_err = RouterFloodError::Stats("stats failed".to_string());
    assert!(format!("{}", stats_err).contains("Statistics error"));
    
    let general_err = RouterFloodError::General("something went wrong".to_string());
    assert!(format!("{}", general_err).contains("Error"));
}

#[test]
fn test_error_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let router_err: RouterFloodError = io_err.into();
    
    match router_err {
        RouterFloodError::Io(_) => assert!(true),
        _ => panic!("Expected Io variant"),
    }
}

#[test]
fn test_error_from_addr_parse_error() {
    let parse_result: Result<std::net::IpAddr, AddrParseError> = "invalid".parse();
    if let Err(parse_err) = parse_result {
        let router_err: RouterFloodError = parse_err.into();
        
        match router_err {
            RouterFloodError::Network(msg) => {
                assert!(msg.contains("Invalid IP address"));
            }
            _ => panic!("Expected Network variant"),
        }
    }
}

#[test]
fn test_validation_error_conversion() {
    let val_err = ValidationError::new("field", "message");
    let router_err: RouterFloodError = val_err.into();
    
    match router_err {
        RouterFloodError::Validation(msg) => {
            assert!(msg.contains("field"));
            assert!(msg.contains("message"));
        }
        _ => panic!("Expected Validation variant"),
    }
}

#[test]
fn test_config_error_conversion() {
    let cfg_err = ConfigError::new("bad config");
    let router_err: RouterFloodError = cfg_err.into();
    
    match router_err {
        RouterFloodError::Config(msg) => {
            assert_eq!(msg, "bad config");
        }
        _ => panic!("Expected Config variant"),
    }
}

#[test]
fn test_packet_error_conversion() {
    let pkt_err = PacketError::build_failed("UDP", "too large");
    let router_err: RouterFloodError = pkt_err.into();
    
    match router_err {
        RouterFloodError::PacketBuild(msg) => {
            assert!(msg.contains("UDP"));
            assert!(msg.contains("too large"));
        }
        _ => panic!("Expected PacketBuild variant"),
    }
}

#[test]
fn test_error_result_type() {
    fn returns_error() -> router_flood::error::Result<()> {
        Err(RouterFloodError::General("test".to_string()))
    }
    
    let result = returns_error();
    assert!(result.is_err());
}

#[test]
fn test_error_chain() {
    // Test that errors can be chained and propagated
    fn inner() -> router_flood::error::Result<String> {
        Err(ValidationError::new("input", "invalid").into())
    }
    
    fn outer() -> router_flood::error::Result<String> {
        inner()?;
        Ok("success".to_string())
    }
    
    let result = outer();
    assert!(result.is_err());
    
    if let Err(e) = result {
        match e {
            RouterFloodError::Validation(_) => assert!(true),
            _ => panic!("Expected Validation error"),
        }
    }
}

#[test]
fn test_error_debug_format() {
    let err = RouterFloodError::Network("test".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("Network"));
}

#[test]
fn test_multiple_error_types() {
    let errors: Vec<RouterFloodError> = vec![
        io::Error::new(io::ErrorKind::Other, "io").into(),
        ValidationError::new("f", "m").into(),
        ConfigError::new("cfg").into(),
        PacketError::build_failed("P", "m").into(),
    ];
    
    assert_eq!(errors.len(), 4);
    
    // Each error should have different variant
    let mut io_count = 0;
    let mut val_count = 0;
    let mut cfg_count = 0;
    let mut pkt_count = 0;
    
    for err in errors {
        match err {
            RouterFloodError::Io(_) => io_count += 1,
            RouterFloodError::Validation(_) => val_count += 1,
            RouterFloodError::Config(_) => cfg_count += 1,
            RouterFloodError::PacketBuild(_) => pkt_count += 1,
            _ => {}
        }
    }
    
    assert_eq!(io_count, 1);
    assert_eq!(val_count, 1);
    assert_eq!(cfg_count, 1);
    assert_eq!(pkt_count, 1);
}