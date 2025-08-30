//! Security and privilege management
//!
//! This module provides enhanced security features including capability-based
//! security, tamper-proof audit logging, threat detection, and input validation.
//!
//! ## Phase 5 Enhancements
//!
//! Phase 5 adds advanced security hardening features:
//! - Threat detection and monitoring
//! - Enhanced input validation with security focus
//! - Rate limiting and anomaly detection
//! - Comprehensive security logging

pub mod capabilities;
pub mod threat_detection;
pub mod input_validation;

pub use capabilities::{
    CapabilityManager, 
    SecurityContext, 
    RequiredCapability,
    TamperProofAuditLog
};
// Temporarily commented for compilation
// pub use threat_detection::{
//     ThreatDetector,
//     ThreatDetectionConfig,
//     ThreatEvent,
//     ThreatType,
//     ThreatSeverity,
//     ValidationResult as ThreatValidationResult,
//     ThreatSummary,
// };
// pub use input_validation::{
//     SecurityInputValidator,
//     ValidationConfig,
//     ValidationResult,
//     SanitizedString,
//     ValidatedIpAddr,
//     IpSecurityLevel,
//     create_strict_validator,
//     validate_ip_strict,
//     validate_ports_strict,
// };