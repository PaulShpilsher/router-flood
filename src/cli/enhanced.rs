//! Enhanced CLI with advanced features
//!
//! This module provides an enhanced command-line interface with features like
//! configuration templates, interactive mode, and advanced validation.
//!
//! This module now serves as a facade that delegates to specialized modules
//! following the Single Responsibility Principle.

use crate::error::Result;
use crate::security::CapabilityManager;
use clap::{ArgMatches, Command};

// Re-export the specialized modules for backward compatibility
pub use self::parser::CliParser;
pub use self::commands::CommandExecutor;
pub use self::interactive::{InteractiveMode, InteractiveConfig};
pub use self::prompts::PromptUtils;

// Internal modules - these are submodules of enhanced
#[path = "parser.rs"]
mod parser;
#[path = "commands.rs"]
mod commands;
#[path = "interactive.rs"]
mod interactive;
#[path = "prompts.rs"]
mod prompts;

/// Enhanced CLI manager with advanced features
/// 
/// This struct maintains backward compatibility while delegating
/// to specialized modules internally.
pub struct EnhancedCli {
    command_executor: CommandExecutor,
    capability_manager: CapabilityManager,
}

impl EnhancedCli {
    /// Create a new enhanced CLI manager
    pub fn new() -> Result<Self> {
        let capability_manager = CapabilityManager::new()?;
        let command_executor = CommandExecutor::new()?;
        
        Ok(Self {
            command_executor,
            capability_manager,
        })
    }

    /// Build the enhanced command structure
    /// 
    /// Delegates to the parser module for command building
    pub fn build_command() -> Command {
        CliParser::build_command()
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

    /// Prompt for user input with default value
    /// 
    /// Delegates to the prompt utilities
    /// 
    /// # Deprecated
    /// This method is deprecated. Use `PromptUtils::prompt_for_input` directly.
    #[deprecated(since = "0.0.2", note = "Use PromptUtils::prompt_for_input directly")]
    pub fn prompt_for_input(&self, prompt: &str, default: &str) -> Result<String> {
        PromptUtils::prompt_for_input(prompt, default)
    }

    /// Prompt for yes/no input
    /// 
    /// Delegates to the prompt utilities
    /// 
    /// # Deprecated
    /// This method is deprecated. Use `PromptUtils::prompt_yes_no` directly.
    #[deprecated(since = "0.0.2", note = "Use PromptUtils::prompt_yes_no directly")]
    pub fn prompt_yes_no(&self, prompt: &str, default: bool) -> Result<bool> {
        PromptUtils::prompt_yes_no(prompt, default)
    }
}

impl Default for EnhancedCli {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                command_executor: CommandExecutor::default(),
                capability_manager: CapabilityManager::default(),
            }
        })
    }
}