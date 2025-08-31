//! Interactive CLI with advanced features
//!
//! This module provides an interactive command-line interface with features like
//! configuration templates, interactive mode, and advanced validation.
//!
//! This module now serves as a facade that delegates to specialized modules
//! following the Single Responsibility Principle.

use crate::error::Result;
use crate::security::Capabilities;
use clap::{ArgMatches, Command};

// Re-export the specialized modules for backward compatibility
pub use self::parser::Parser;
pub use self::commands::Commands;
pub use self::interactive::{InteractiveMode, InteractiveConfig};
pub use self::prompts::Prompts;

// Internal modules - these are submodules of enhanced
#[path = "parser.rs"]
mod parser;
#[path = "commands.rs"]
mod commands;
#[path = "interactive.rs"]
mod interactive;
#[path = "prompts.rs"]
mod prompts;

/// Interactive CLI manager with advanced features
/// 
/// This struct maintains backward compatibility while delegating
/// to specialized modules internally.
pub struct Interactive {
    command_executor: Commands,
    capability_manager: Capabilities,
}

impl Interactive {
    /// Create a new interactive CLI manager
    pub fn new() -> Result<Self> {
        let capability_manager = Capabilities::new()?;
        let command_executor = Commands::new()?;
        
        Ok(Self {
            command_executor,
            capability_manager,
        })
    }

    /// Build the interactive command structure
    /// 
    /// Delegates to the parser module for command building
    pub fn build_command() -> Command {
        Parser::build_command()
    }

    /// Handle configuration subcommands
    /// 
    /// Delegates to the command executor
    pub async fn handle_config_command(&self, matches: &ArgMatches) -> Result<()> {
        self.command_executor.handle_config_command(matches).await
    }

    /// Handle system subcommands
    /// 
    /// Delegates to the command executor
    pub async fn handle_system_command(&self, matches: &ArgMatches) -> Result<()> {
        self.command_executor.handle_system_command(matches).await
    }

    /// Handle interactive mode
    /// 
    /// Creates an interactive mode handler and runs it
    pub async fn handle_interactive_mode(&self) -> Result<()> {
        let interactive = InteractiveMode::new(self.capability_manager.clone());
        interactive.run().await
    }

    /// Handle list templates command
    /// 
    /// Delegates to the command executor
    pub async fn handle_list_templates(&self) -> Result<()> {
        // For now, just return Ok - this is a placeholder
        Ok(())
    }

}



impl Default for Interactive {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                command_executor: Commands::default(),
                capability_manager: Capabilities::default(),
            }
        })
    }
}