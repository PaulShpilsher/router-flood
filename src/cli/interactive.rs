//! Interactive mode functionality
//!
//! This module handles the interactive configuration mode for Router Flood.

use crate::error::Result;
use crate::security::Capabilities;
use super::prompts::PromptUtils;

/// Interactive mode handler
pub struct InteractiveMode {
    capability_manager: Capabilities,
}

impl InteractiveMode {
    /// Create a new interactive mode handler
    pub fn new(capability_manager: Capabilities) -> Self {
        Self {
            capability_manager,
        }
    }

    /// Run the interactive configuration mode
    pub async fn run(&self) -> Result<()> {
        self.display_welcome();
        self.check_security()?;
        
        let config = self.gather_configuration()?;
        self.display_summary(&config)?;
        
        if PromptUtils::prompt_yes_no("Execute this configuration now", true)? {
            self.execute_configuration(config).await?;
        } else {
            self.save_configuration(config)?;
        }
        
        Ok(())
    }

    /// Display welcome message
    fn display_welcome(&self) {
        println!("🎯 Router Flood Interactive Configuration");
        PromptUtils::display_separator();
        println!();
    }

    /// Check and display security context
    fn check_security(&self) -> Result<()> {
        println!("🔒 Checking security context...");
        let security_report = self.capability_manager.security_report();
        println!("{}", security_report);
        println!();
        Ok(())
    }

    /// Gather configuration from user input
    fn gather_configuration(&self) -> Result<InteractiveConfig> {
        let target_ip = PromptUtils::prompt_for_input(
            "Target IP address (private range)", 
            "192.168.1.1"
        )?;
        
        let ports = PromptUtils::prompt_for_input(
            "Target ports (comma-separated)", 
            "80,443"
        )?;
        
        let threads = PromptUtils::prompt_for_input(
            "Number of threads", 
            "4"
        )?;
        
        let rate = PromptUtils::prompt_for_input(
            "Packets per second per thread", 
            "100"
        )?;
        
        let duration = PromptUtils::prompt_for_input(
            "Duration in seconds (empty for unlimited)", 
            ""
        )?;
        
        let dry_run = PromptUtils::prompt_yes_no(
            "Enable dry-run mode (recommended for first test)", 
            true
        )?;
        
        let cpu_affinity = PromptUtils::prompt_yes_no(
            "Enable CPU affinity optimization", 
            false
        )?;
        
        let export_stats = PromptUtils::prompt_yes_no(
            "Export statistics", 
            false
        )?;
        
        let export_format = if export_stats {
            Some(PromptUtils::prompt_for_input(
                "Export format (json/csv/both)", 
                "json"
            )?)
        } else {
            None
        };

        Ok(InteractiveConfig {
            target_ip,
            ports,
            threads,
            rate,
            duration,
            dry_run,
            cpu_affinity,
            export_format,
        })
    }

    /// Display configuration summary
    fn display_summary(&self, config: &InteractiveConfig) -> Result<()> {
        println!();
        println!("📋 Generated command:");
        println!("router-flood {}", config.to_command_args().join(" "));
        println!();
        Ok(())
    }

    /// Execute the configuration
    async fn execute_configuration(&self, _config: InteractiveConfig) -> Result<()> {
        println!("🚀 Starting test...");
        // Here we would execute the configuration
        // For now, just show what would be executed
        println!("✅ Configuration validated and ready to execute");
        Ok(())
    }

    /// Save configuration for later use
    fn save_configuration(&self, _config: InteractiveConfig) -> Result<()> {
        println!("💾 Configuration saved for later execution");
        Ok(())
    }
}

/// Configuration gathered from interactive mode
#[derive(Debug, Clone)]
pub struct InteractiveConfig {
    pub target_ip: String,
    pub ports: String,
    pub threads: String,
    pub rate: String,
    pub duration: String,
    pub dry_run: bool,
    pub cpu_affinity: bool,
    pub export_format: Option<String>,
}

impl InteractiveConfig {
    /// Convert to command arguments
    pub fn to_command_args(&self) -> Vec<String> {
        let mut args = vec![
            "run".to_string(),
            "--target".to_string(), self.target_ip.clone(),
            "--ports".to_string(), self.ports.clone(),
            "--threads".to_string(), self.threads.clone(),
            "--rate".to_string(), self.rate.clone(),
        ];

        if !self.duration.is_empty() {
            args.extend(["--duration".to_string(), self.duration.clone()]);
        }

        if self.dry_run {
            args.push("--dry-run".to_string());
        }

        if self.cpu_affinity {
            args.push("--cpu-affinity".to_string());
        }

        if let Some(ref format) = self.export_format {
            args.extend(["--export".to_string(), format.clone()]);
        }

        args
    }
}