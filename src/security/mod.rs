//! Security and privilege management
//!
//! This module provides enhanced security features including capability-based
//! security, tamper-proof audit logging, and privilege management.

pub mod capabilities;

pub use capabilities::{
    CapabilityManager, 
    SecurityContext, 
    RequiredCapability,
    TamperProofAuditLog
};