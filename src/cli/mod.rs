//! Command-line interface handling
//!
//! This module handles all CLI argument parsing, validation, help text
//! generation, and enhanced CLI features.
//!
//! ## Guided CLI with Progressive Disclosure
//!
//! The guided CLI provides progressive disclosure:
//! - Quick mode: Minimal options for beginners
//! - Standard mode: Common options for typical use
//! - Advanced mode: Full control for power users

pub mod basic;
pub mod enhanced;
pub mod guided;

// Re-export basic CLI functions for backward compatibility
pub use basic::{
    parse_arguments, 
    process_cli_config, 
    handle_pre_execution_commands,
    parse_ports,
    parse_positive_number,
    parse_export_format
};

pub use enhanced::{InteractiveCli, EnhancedCli}; // EnhancedCli is compatibility alias
// Guided CLI exports
pub use guided::{GuidedCli, GuidanceLevel, CliMode, validate_target_ip}; // CliMode is compatibility alias