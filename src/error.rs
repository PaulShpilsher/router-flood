//! Consolidated error handling for router-flood
//!
//! This module provides all error types and handling for the application.

use std::fmt;
use std::io;
use std::net::AddrParseError;

/// Type alias for Results using our error type
pub type Result<T> = std::result::Result<T, RouterFloodError>;

/// Main error type for router-flood
#[derive(Debug)]
pub enum RouterFloodError {
    /// I/O errors
    Io(io::Error),
    
    /// Network-related errors
    Network(String),
    
    /// Configuration errors
    Config(String),
    
    /// Validation errors
    Validation(String),
    
    /// Packet building errors
    PacketBuild(String),
    
    /// System resource errors
    SystemResource(String),
    
    /// Permission errors
    Permission(String),
    
    /// Statistics errors
    Stats(String),
    
    /// General errors
    General(String),
}

impl fmt::Display for RouterFloodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::PacketBuild(msg) => write!(f, "Packet building error: {}", msg),
            Self::SystemResource(msg) => write!(f, "System resource error: {}", msg),
            Self::Permission(msg) => write!(f, "Permission error: {}", msg),
            Self::Stats(msg) => write!(f, "Statistics error: {}", msg),
            Self::General(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for RouterFloodError {}

impl From<io::Error> for RouterFloodError {
    fn from(error: io::Error) -> Self {
        RouterFloodError::Io(error)
    }
}

impl From<AddrParseError> for RouterFloodError {
    fn from(error: AddrParseError) -> Self {
        RouterFloodError::Network(format!("Invalid IP address: {}", error))
    }
}

/// Configuration error type
#[derive(Debug, Clone)]
pub struct ConfigError {
    pub message: String,
}

impl ConfigError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error: {}", self.message)
    }
}

impl From<ConfigError> for RouterFloodError {
    fn from(error: ConfigError) -> Self {
        RouterFloodError::Config(error.message)
    }
}

/// Validation error type
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub reason: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error for '{}': {}", self.field, self.reason)
    }
}

impl From<ValidationError> for RouterFloodError {
    fn from(error: ValidationError) -> Self {
        RouterFloodError::Validation(format!("{}: {}", error.field, error.reason))
    }
}

/// Packet error type
#[derive(Debug, Clone)]
pub struct PacketError {
    pub packet_type: String,
    pub reason: String,
}

impl PacketError {
    pub fn build_failed(packet_type: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            packet_type: packet_type.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to build {} packet: {}", self.packet_type, self.reason)
    }
}

impl From<PacketError> for RouterFloodError {
    fn from(error: PacketError) -> Self {
        RouterFloodError::PacketBuild(format!("{}: {}", error.packet_type, error.reason))
    }
}

/// System error type
#[derive(Debug, Clone)]
pub struct SystemError {
    pub resource: String,
    pub reason: String,
}

impl SystemError {
    pub fn resource_unavailable(resource: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "System resource '{}' unavailable: {}", self.resource, self.reason)
    }
}

impl From<SystemError> for RouterFloodError {
    fn from(error: SystemError) -> Self {
        RouterFloodError::SystemResource(format!("{}: {}", error.resource, error.reason))
    }
}

/// Statistics error type
#[derive(Debug, Clone)]
pub struct StatsError {
    pub message: String,
}

impl StatsError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Statistics error: {}", self.message)
    }
}

impl From<StatsError> for RouterFloodError {
    fn from(error: StatsError) -> Self {
        RouterFloodError::Stats(error.message)
    }
}

/// Display user-friendly error messages
pub fn display_user_friendly_error(error: &RouterFloodError) {
    eprintln!("\n❌ Error: {}", error);
    
    // Provide helpful suggestions based on error type
    match error {
        RouterFloodError::Permission(_) => {
            eprintln!("\n💡 Tip: Try running with sudo or configure capabilities:");
            eprintln!("   sudo setcap cap_net_raw+ep {}", std::env::current_exe().unwrap_or_default().display());
        }
        RouterFloodError::Network(_) => {
            eprintln!("\n💡 Tip: Check your network configuration and target accessibility");
        }
        RouterFloodError::Config(_) => {
            eprintln!("\n💡 Tip: Verify your configuration file format and values");
        }
        _ => {}
    }
}