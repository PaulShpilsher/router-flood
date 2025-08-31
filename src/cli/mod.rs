//! Command-line interface handling
//!
//! This module handles all CLI argument parsing, validation, and help text generation.

pub mod basic;

// Re-export CLI functions
pub use basic::{
    parse_arguments, 
    process_cli_config, 
    handle_pre_execution_commands,
    parse_ports,
    parse_positive_number,
    parse_export_format
};