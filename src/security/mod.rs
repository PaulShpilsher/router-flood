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
pub mod input_validation;
pub mod validation;

pub use capabilities::{
    CapabilityManager, 
    SecurityContext, 
    RequiredCapability,
    TamperProofAuditLog
};
pub use threat_detection::{
    ThreatDetector,
    ThreatDetectionConfig,
    ThreatEvent,
    ThreatType,
    ThreatSeverity,
    ValidationResult as ThreatValidationResult,
    ThreatSummary,
};
pub use input_validation::{
    InputValidator,
    ValidationConfig,
    ValidationResult,
    SanitizedString,
    ValidatedIpAddr,
    IpSecurityLevel,
    create_input_validator,
    validate_ip_address,
    validate_port_list,
};