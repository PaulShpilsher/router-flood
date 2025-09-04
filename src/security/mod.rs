//! Security and privilege management
//!
//! This module provides enhanced security features including capability-based
//! security, tamper-proof audit logging, threat detection, and input validation.
//!
//! ## Enhanced Security
//!
//! Enhanced security adds security hardening features:
//! - Threat detection and monitoring
//! - Input validation with security focus
//! - Rate limiting and anomaly detection
//! - Comprehensive security logging

pub mod audit;
pub mod capabilities;
pub mod threat_detection;
pub mod validation;

pub use audit::{AuditLogger, EventType};
pub use capabilities::{
    Capabilities, 
    SecurityContext, 
    RequiredCapability,
    AuditLog
};
pub use threat_detection::{
    ThreatDetection,
    ThreatDetectionConfig,
    ThreatEvent,
    ThreatType,
    ThreatSeverity,
    ValidationResult as ThreatValidationResult,
    ThreatSummary,
};
pub use validation::{
    InputValidation,
    ValidationConfig,
};