//! Simplified CLI runner
//!
//! This module provides a simplified CLI runner after removing guided mode.

use clap::ArgMatches;
use tracing::info;

use crate::config::Config;
use crate::error::{Result, RouterFloodError};

/// Simplified CLI application runner
pub struct CliRunner {
    config: Config,
}

impl CliRunner {
    /// Create a new CLI runner
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the CLI application
    pub fn run(&self) -> Result<()> {
        info!("Starting router-flood with simplified CLI");
        // Implementation will be added during consolidation phase
        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// Create a CLI runner from command line arguments
pub fn from_args(_matches: &ArgMatches) -> Result<CliRunner> {
    // For now, use a basic config - will be properly implemented in consolidation phase
    use crate::config::default_config;
    let config = default_config();
    Ok(CliRunner::new(config))
}