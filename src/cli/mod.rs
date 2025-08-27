//! Command-line interface handling
//!
//! This module handles all CLI argument parsing, validation, help text
//! generation, and enhanced CLI features.

pub mod basic;
pub mod enhanced;

// Re-export basic CLI functions for backward compatibility
pub use basic::{
    parse_arguments, 
    process_cli_config, 
    handle_pre_execution_commands,
    parse_ports,
    parse_positive_number,
    parse_export_format
};

pub use enhanced::EnhancedCli;