//! Command execution handlers
//!
//! This module handles the execution of various CLI commands.

use crate::config::{ConfigTemplates, ConfigSchema};
use crate::error::{ConfigError, Result};
use crate::security::CapabilityManager;
use crate::performance::CpuAffinityManager;
use clap::ArgMatches;

/// Command executor for handling CLI commands
pub struct CommandExecutor {
    capability_manager: CapabilityManager,
    cpu_manager: CpuAffinityManager,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Result<Self> {
        Ok(Self {
            capability_manager: CapabilityManager::new()?,
            cpu_manager: CpuAffinityManager::new()?,
        })
    }

    /// Handle configuration subcommands
    pub async fn handle_config_command(&self, matches: &ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("generate", sub_matches)) => {
                self.handle_config_generate(sub_matches).await
            }
            Some(("validate", sub_matches)) => {
                self.handle_config_validate(sub_matches).await
            }
            Some(("list-templates", _)) => {
                self.handle_list_templates().await
            }
            _ => {
                eprintln!("No config subcommand specified. Use --help for options.");
                Ok(())
            }
        }
    }

    /// Handle system subcommands
    pub async fn handle_system_command(&self, matches: &ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("info", _)) => {
                self.display_system_info().await
            }
            Some(("security", _)) => {
                self.display_security_info().await
            }
            Some(("performance", sub_matches)) => {
                self.display_performance_info(sub_matches).await
            }
            _ => {
                eprintln!("No system subcommand specified. Use --help for options.");
                Ok(())
            }
        }
    }

    /// Generate configuration template
    async fn handle_config_generate(&self, matches: &ArgMatches) -> Result<()> {
        let template_name = matches.get_one::<String>("template").unwrap();
        let output_path = matches.get_one::<String>("output").unwrap();

        println!("📝 Generating {} configuration template...", template_name);

        let template = ConfigTemplates::get_template(template_name)
            .ok_or_else(|| ConfigError::InvalidValue {
                field: "template".to_string(),
                value: template_name.clone(),
                reason: "Unknown template type".to_string(),
            })?;

        let yaml_content = ConfigTemplates::template_to_yaml(&template)?;

        tokio::fs::write(output_path, yaml_content).await
            .map_err(|e| ConfigError::FileNotFound(format!("Failed to write template: {}", e)))?;

        println!("✅ Template generated: {}", output_path);
        println!("💡 Edit the file and run: router-flood run --config {}", output_path);

        Ok(())
    }

    /// Validate configuration file
    async fn handle_config_validate(&self, matches: &ArgMatches) -> Result<()> {
        let config_path = matches.get_one::<String>("config").unwrap();

        println!("🔍 Validating configuration: {}", config_path);

        let config_content = tokio::fs::read_to_string(config_path).await
            .map_err(|e| ConfigError::FileNotFound(format!("Failed to read config: {}", e)))?;

        let config: crate::config::Config = serde_yaml::from_str(&config_content)
            .map_err(|e| ConfigError::ParseError(format!("YAML parse error: {}", e)))?;

        ConfigSchema::validate(&config)?;

        println!("✅ Configuration is valid");
        println!("📊 Configuration summary:");
        println!("   Target: {}:{:?}", config.target.ip, config.target.ports);
        println!("   Threads: {}", config.attack.threads);
        println!("   Rate: {} PPS", config.attack.packet_rate);
        println!("   Dry run: {}", config.safety.dry_run);

        Ok(())
    }

    /// List available templates
    async fn handle_list_templates(&self) -> Result<()> {
        println!("📋 Available Configuration Templates:");
        println!("====================================");

        for template_name in ConfigTemplates::list_templates() {
            if let Some(template) = ConfigTemplates::get_template(template_name) {
                println!();
                println!("🔧 {}", template_name);
                println!("   Target: {}:{:?}", template.target.ip, template.target.ports);
                println!("   Threads: {}, Rate: {} PPS", template.attack.threads, template.attack.packet_rate);
                
                if let Some(duration) = template.attack.duration {
                    println!("   Duration: {} seconds", duration);
                }
                
                println!("   Dry run: {}", template.safety.dry_run);
                
                // Show protocol mix
                let mix = &template.target.protocol_mix;
                println!("   Protocols: UDP:{:.1}% TCP-SYN:{:.1}% TCP-ACK:{:.1}% ICMP:{:.1}%",
                    mix.udp_ratio * 100.0,
                    mix.tcp_syn_ratio * 100.0,
                    mix.tcp_ack_ratio * 100.0,
                    mix.icmp_ratio * 100.0
                );
            }
        }

        println!();
        println!("💡 Generate a template with: router-flood config generate --template <name>");

        Ok(())
    }

    /// Display system information
    async fn display_system_info(&self) -> Result<()> {
        println!("🖥️  System Information");
        println!("=====================");

        let topology = self.cpu_manager.topology();
        println!("CPU Information:");
        println!("  Total CPUs: {}", topology.total_cpus);
        println!("  NUMA Nodes: {}", topology.numa_nodes.len());
        println!("  Hyperthreading: {}", if topology.hyperthreading_enabled { "Enabled" } else { "Disabled" });

        for node in &topology.numa_nodes {
            println!("  Node {}: {} CPUs ({:?})", node.node_id, node.cpus.len(), node.cpus);
            if let Some(total) = node.memory_total {
                println!("    Memory: {:.2} GB total", total as f64 / (1024.0 * 1024.0 * 1024.0));
            }
        }

        // System capabilities
        println!();
        println!("System Capabilities:");
        println!("  Raw sockets: {}", if self.capability_manager.security_context().has_net_raw { "✅" } else { "❌" });
        println!("  Network admin: {}", if self.capability_manager.security_context().has_net_admin { "✅" } else { "❌" });

        Ok(())
    }

    /// Display security information
    async fn display_security_info(&self) -> Result<()> {
        let report = self.capability_manager.security_report();
        println!("{}", report);
        Ok(())
    }

    /// Display performance recommendations
    async fn display_performance_info(&self, matches: &ArgMatches) -> Result<()> {
        let workers = *matches.get_one::<usize>("workers").unwrap();
        
        println!("⚡ Performance Analysis for {} workers", workers);
        println!("=====================================");

        let recommendations = self.cpu_manager.get_performance_recommendations(workers);
        
        if recommendations.is_empty() {
            println!("✅ Configuration looks optimal for your system");
        } else {
            println!("💡 Recommendations:");
            for (i, rec) in recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
        }

        // Show CPU assignments
        let mut cpu_manager = CpuAffinityManager::new()?;
        let assignments = cpu_manager.assign_workers(workers)?;
        
        println!();
        println!("🎯 Proposed CPU Assignments:");
        for assignment in assignments {
            println!("  Worker {} → CPU {} (NUMA Node {})", 
                assignment.worker_id, assignment.cpu_id, assignment.numa_node);
        }

        Ok(())
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                capability_manager: CapabilityManager::default(),
                cpu_manager: CpuAffinityManager::default(),
            }
        })
    }
}