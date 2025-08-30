//! Centralized error handling for router-flood
//!
//! This module provides comprehensive error types and handling utilities
//! to replace unwrap/expect patterns throughout the codebase.
//!
//! ## Interactive User Experience
//!
//! The interactive error system provides:
//! - Actionable error messages with specific guidance
//! - Examples showing how to fix common issues
//! - Progressive help based on error context
//! - Beginner-friendly explanations

pub mod user_friendly;
pub mod user_friendly_enhanced;

pub use user_friendly::{UserFriendlyError, display_user_friendly_error, ErrorContext};
// Enhanced error handling exports
pub use user_friendly_enhanced::{
    EnhancedUserError, ErrorSeverity, display_enhanced_user_error, show_quick_help
};

use std::fmt;
use std::io;

/// Common validation error messages as constants to avoid string allocations
pub mod messages {
    // Thread validation messages
    pub const THREAD_COUNT_ZERO: &str = "Thread count must be greater than 0";
    pub const THREAD_COUNT_EXCEEDS_LIMIT: &str = "threads";
    
    // Packet rate validation messages
    pub const PACKET_RATE_ZERO: &str = "Packet rate must be greater than 0";
    pub const PACKET_RATE_EXCEEDS_LIMIT: &str = "packet_rate";
    
    // Packet size validation messages
    pub const MIN_PACKET_SIZE_TOO_LARGE: &str = "Minimum packet size cannot be greater than maximum";
    pub const MIN_PACKET_SIZE_FIELD: &str = "min_packet_size";
    pub const MAX_PACKET_SIZE_FIELD: &str = "max_packet_size";
    
    // Port validation messages
    pub const NO_TARGET_PORTS: &str = "At least one target port must be specified";
    pub const TARGET_PORTS_FIELD: &str = "target_ports";
    
    // Protocol validation messages
    pub const PROTOCOL_RATIOS_SUM: &str = "Protocol ratios must sum to 1.0";
    pub const PROTOCOL_RATIOS_RANGE: &str = "All protocol ratios must be between 0.0 and 1.0";
    
    // Burst pattern validation messages
    pub const SUSTAINED_RATE_ZERO: &str = "Sustained rate must be greater than 0";
    pub const SUSTAINED_RATE_FIELD: &str = "sustained_rate";
    pub const BURST_SIZE_ZERO: &str = "Burst size must be greater than 0";
    pub const BURST_INTERVAL_ZERO: &str = "Burst interval must be greater than 0";
    pub const RAMP_RATES_ZERO: &str = "Ramp start and end rates must be greater than 0";
    pub const RAMP_DURATION_ZERO: &str = "Ramp duration must be greater than 0";
    
    // Duration validation messages
    pub const DURATION_ZERO: &str = "Duration must be greater than 0 seconds";
    
    // Stats validation messages
    pub const STATS_INTERVAL_ZERO: &str = "Stats interval must be greater than 0";
    pub const STATS_INTERVAL_FIELD: &str = "stats_interval";
    pub const EXPORT_INTERVAL_ZERO: &str = "Export interval must be greater than 0";
    pub const EXPORT_INTERVAL_FIELD: &str = "export_interval";
    
    // IP validation messages
    pub const INVALID_IP_FORMAT: &str = "Invalid IP address format";
    
    // Safety validation messages
    pub const MAX_THREADS_ZERO: &str = "Maximum threads must be greater than 0";
    pub const MAX_THREADS_FIELD: &str = "max_threads";
    pub const MAX_PACKET_RATE_ZERO: &str = "Maximum packet rate must be greater than 0";
    pub const MAX_PACKET_RATE_FIELD: &str = "max_packet_rate";
    
    // Export validation messages
    pub const FILENAME_PATTERN_EMPTY: &str = "Filename pattern cannot be empty";
    pub const FILENAME_PATTERN_FIELD: &str = "filename_pattern";
}

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
    InvalidIpRange { ip: String, reason: &'static str },
    ExceedsLimit { field: &'static str, value: u64, limit: u64 },
    SystemRequirement(&'static str),
    PrivilegeRequired(&'static str),
    PermissionDenied(&'static str),
}

#[derive(Debug)]
pub enum PacketError {
    BuildFailed { packet_type: String, reason: String },
    BufferTooSmall { required: usize, available: usize },
    InvalidParameters(String),
    PluginError(String),
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
            PacketError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
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

    fn map_validation_error(self, _context: &str) -> Result<T> {
        // For now, we'll use a generic message since we need &'static str
        // In a real implementation, you'd want to create specific constants for each context
        self.map_err(|_| ValidationError::SystemRequirement("System requirement not met").into())
    }
}