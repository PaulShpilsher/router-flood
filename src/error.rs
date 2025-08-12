//! Centralized error handling for router-flood
//!
//! This module provides comprehensive error types and handling utilities
//! to replace unwrap/expect patterns throughout the codebase.

use std::fmt;
use std::io;

/// Main error type for the router-flood application
#[derive(Debug)]
pub enum RouterFloodError {
    /// Configuration-related errors
    Config(ConfigError),
    /// Network-related errors
    Network(NetworkError),
    /// Validation errors
    Validation(ValidationError),
    /// Packet building errors
    Packet(PacketError),
    /// Statistics and export errors
    Stats(StatsError),
    /// System-level errors
    System(SystemError),
    /// Audit logging errors
    Audit(AuditError),
    /// I/O errors
    Io(io::Error),
}

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    InvalidValue { field: String, value: String, reason: String },
    MissingRequired(String),
}

#[derive(Debug)]
pub enum NetworkError {
    InterfaceNotFound(String),
    ChannelCreation(String),
    PacketSend(String),
    InvalidAddress(String),
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidIpRange { ip: String, reason: String },
    ExceedsLimit { field: String, value: u64, limit: u64 },
    SystemRequirement(String),
    PrivilegeRequired(String),
    PermissionDenied(String),
}

#[derive(Debug)]
pub enum PacketError {
    BuildFailed { packet_type: String, reason: String },
    BufferTooSmall { required: usize, available: usize },
    InvalidParameters(String),
}

#[derive(Debug)]
pub enum StatsError {
    ExportFailed(String),
    SerializationError(String),
    FileWriteError(String),
}

#[derive(Debug)]
pub enum SystemError {
    PermissionDenied(String),
    ResourceUnavailable(String),
    LimitExceeded(String),
}

#[derive(Debug)]
pub enum AuditError {
    LogCreationFailed(String),
    WriteError(String),
    FormatError(String),
}

impl fmt::Display for RouterFloodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterFloodError::Config(e) => write!(f, "Configuration error: {}", e),
            RouterFloodError::Network(e) => write!(f, "Network error: {}", e),
            RouterFloodError::Validation(e) => write!(f, "Validation error: {}", e),
            RouterFloodError::Packet(e) => write!(f, "Packet error: {}", e),
            RouterFloodError::Stats(e) => write!(f, "Statistics error: {}", e),
            RouterFloodError::System(e) => write!(f, "System error: {}", e),
            RouterFloodError::Audit(e) => write!(f, "Audit error: {}", e),
            RouterFloodError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "Configuration file not found: {}", path),
            ConfigError::ParseError(msg) => write!(f, "Failed to parse configuration: {}", msg),
            ConfigError::InvalidValue { field, value, reason } => {
                write!(f, "Invalid value '{}' for field '{}': {}", value, field, reason)
            }
            ConfigError::MissingRequired(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::InterfaceNotFound(name) => write!(f, "Network interface not found: {}", name),
            NetworkError::ChannelCreation(msg) => write!(f, "Failed to create network channel: {}", msg),
            NetworkError::PacketSend(msg) => write!(f, "Failed to send packet: {}", msg),
            NetworkError::InvalidAddress(addr) => write!(f, "Invalid network address: {}", addr),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidIpRange { ip, reason } => {
                write!(f, "IP address {} is invalid: {}", ip, reason)
            }
            ValidationError::ExceedsLimit { field, value, limit } => {
                write!(f, "Value {} for {} exceeds limit of {}", value, field, limit)
            }
            ValidationError::SystemRequirement(msg) => write!(f, "System requirement not met: {}", msg),
            ValidationError::PrivilegeRequired(msg) => write!(f, "Privilege required: {}", msg),
            ValidationError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketError::BuildFailed { packet_type, reason } => {
                write!(f, "Failed to build {} packet: {}", packet_type, reason)
            }
            PacketError::BufferTooSmall { required, available } => {
                write!(f, "Buffer too small: required {}, available {}", required, available)
            }
            PacketError::InvalidParameters(msg) => write!(f, "Invalid packet parameters: {}", msg),
        }
    }
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatsError::ExportFailed(msg) => write!(f, "Failed to export statistics: {}", msg),
            StatsError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StatsError::FileWriteError(msg) => write!(f, "File write error: {}", msg),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            SystemError::ResourceUnavailable(msg) => write!(f, "Resource unavailable: {}", msg),
            SystemError::LimitExceeded(msg) => write!(f, "System limit exceeded: {}", msg),
        }
    }
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditError::LogCreationFailed(msg) => write!(f, "Failed to create audit log: {}", msg),
            AuditError::WriteError(msg) => write!(f, "Audit write error: {}", msg),
            AuditError::FormatError(msg) => write!(f, "Audit format error: {}", msg),
        }
    }
}

impl std::error::Error for RouterFloodError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RouterFloodError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl std::error::Error for ConfigError {}
impl std::error::Error for NetworkError {}
impl std::error::Error for ValidationError {}
impl std::error::Error for PacketError {}
impl std::error::Error for StatsError {}
impl std::error::Error for SystemError {}
impl std::error::Error for AuditError {}

// Conversion implementations for easier error handling
impl From<io::Error> for RouterFloodError {
    fn from(error: io::Error) -> Self {
        RouterFloodError::Io(error)
    }
}

impl From<ConfigError> for RouterFloodError {
    fn from(error: ConfigError) -> Self {
        RouterFloodError::Config(error)
    }
}

impl From<NetworkError> for RouterFloodError {
    fn from(error: NetworkError) -> Self {
        RouterFloodError::Network(error)
    }
}

impl From<ValidationError> for RouterFloodError {
    fn from(error: ValidationError) -> Self {
        RouterFloodError::Validation(error)
    }
}

impl From<PacketError> for RouterFloodError {
    fn from(error: PacketError) -> Self {
        RouterFloodError::Packet(error)
    }
}

impl From<StatsError> for RouterFloodError {
    fn from(error: StatsError) -> Self {
        RouterFloodError::Stats(error)
    }
}

impl From<SystemError> for RouterFloodError {
    fn from(error: SystemError) -> Self {
        RouterFloodError::System(error)
    }
}

impl From<AuditError> for RouterFloodError {
    fn from(error: AuditError) -> Self {
        RouterFloodError::Audit(error)
    }
}

/// Type alias for Results used throughout the application
pub type Result<T> = std::result::Result<T, RouterFloodError>;

/// Helper trait for converting string errors to appropriate error types
pub trait MapError<T> {
    fn map_config_error(self, field: &str) -> Result<T>;
    fn map_network_error(self, context: &str) -> Result<T>;
    fn map_validation_error(self, context: &str) -> Result<T>;
}

impl<T, E: fmt::Display> MapError<T> for std::result::Result<T, E> {
    fn map_config_error(self, field: &str) -> Result<T> {
        self.map_err(|e| ConfigError::InvalidValue {
            field: field.to_string(),
            value: "unknown".to_string(),
            reason: e.to_string(),
        }.into())
    }

    fn map_network_error(self, context: &str) -> Result<T> {
        self.map_err(|e| NetworkError::ChannelCreation(format!("{}: {}", context, e)).into())
    }

    fn map_validation_error(self, context: &str) -> Result<T> {
        self.map_err(|e| ValidationError::SystemRequirement(format!("{}: {}", context, e)).into())
    }
}