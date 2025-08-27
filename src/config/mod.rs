//! Configuration management with builder pattern and validation
//!
//! This module provides enhanced configuration management with centralized
//! validation and a fluent builder API.

pub mod builder;
pub mod schema;
pub mod validation;

pub use builder::ConfigBuilder;
pub use schema::{ConfigSchema, ConfigTemplates};
pub use validation::ConfigValidator;

// Re-export existing config types for backward compatibility
pub use crate::config_original::{
    Config, TargetConfig, ProtocolMix, AttackConfig, SafetyConfig,
    MonitoringConfig, ExportConfig, ExportFormat, BurstPattern,
    load_config, get_default_config,
};