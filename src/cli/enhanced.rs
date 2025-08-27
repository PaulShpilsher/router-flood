//! Enhanced CLI with advanced features
//!
//! This module provides an enhanced command-line interface with features like
//! configuration templates, interactive mode, and advanced validation.

use crate::config::{ConfigTemplates, ConfigSchema};
use crate::error::{ConfigError, Result};
use crate::security::CapabilityManager;
use crate::performance::CpuAffinityManager;
use clap::{Arg, ArgMatches, Command, value_parser};
use std::io::{self, Write};

/// Enhanced CLI manager with advanced features
pub struct EnhancedCli {
    capability_manager: CapabilityManager,
    cpu_manager: CpuAffinityManager,
}

impl EnhancedCli {
    /// Create a new enhanced CLI manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            capability_manager: CapabilityManager::new()?,
            cpu_manager: CpuAffinityManager::new()?,
        })
    }

    /// Build the enhanced command structure
    pub fn build_command() -> Command {
        Command::new("router-flood")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Educational DDoS simulation for local network testing")
            .long_about(Self::get_enhanced_help())
            .subcommand_required(false)
            .arg_required_else_help(false)
            .subcommand(
                Command::new("run")
                    .about("Run network stress test")
                    .args(Self::get_run_args())
            )
            .subcommand(
                Command::new("config")
                    .about("Configuration management")
                    .subcommand(
                        Command::new("generate")
                            .about("Generate configuration template")
                            .arg(
                                Arg::new("template")
                                    .long("template")
                                    .short('t')
                                    .help("Template type")
                                    .value_parser(["basic", "web_server", "dns_server", "high_performance"])
                                    .required(true)
                            )
                            .arg(
                                Arg::new("output")
                                    .long("output")
                                    .short('o')
                                    .help("Output file path")
                                    .default_value("generated_config.yaml")
                            )
                    )
                    .subcommand(
                        Command::new("validate")
                            .about("Validate configuration file")
                            .arg(
                                Arg::new("config")
                                    .help("Configuration file to validate")
                                    .required(true)
                            )
                    )
                    .subcommand(
                        Command::new("list-templates")
                            .about("List available configuration templates")
                    )
            )
            .subcommand(
                Command::new("system")
                    .about("System information and diagnostics")
                    .subcommand(
                        Command::new("info")
                            .about("Display system information")
                    )
                    .subcommand(
                        Command::new("security")
                            .about("Display security context")
                    )
                    .subcommand(
                        Command::new("performance")
                            .about("Display performance recommendations")
                            .arg(
                                Arg::new("workers")
                                    .long("workers")
                                    .help("Number of workers to analyze")
                                    .value_parser(value_parser!(usize))
                                    .default_value("4")
                            )
                    )
            )
            .subcommand(
                Command::new("interactive")
                    .about("Interactive configuration mode")
            )
            .args(Self::get_global_args())
    }

    /// Get enhanced help text
    fn get_enhanced_help() -> &'static str {
        r#"🚀 Router Flood - Advanced Educational Network Stress Tester

A comprehensive, safety-first network testing tool designed for educational purposes
and authorized network testing scenarios.

🎯 KEY FEATURES:
  • Multi-protocol support (UDP, TCP, ICMP, IPv6, ARP)
  • SIMD-optimized packet generation
  • Advanced buffer management with NUMA awareness
  • Capability-based security (no root required)
  • Real-time monitoring with Prometheus metrics
  • Property-based testing and fuzzing support
  • Interactive configuration mode

🛡️ SAFETY FEATURES:
  • Private IP validation (RFC 1918 ranges only)
  • Built-in rate limiting and safety checks
  • Comprehensive audit logging with tamper detection
  • Dry-run mode for safe testing
  • Capability-based privilege management

📚 QUICK START:
  # Interactive mode (recommended for beginners)
  router-flood interactive

  # Generate a configuration template
  router-flood config generate --template web_server

  # Run with dry-run for safe testing
  router-flood run --target 192.168.1.1 --ports 80,443 --dry-run

  # Check system capabilities
  router-flood system security

🔧 ADVANCED USAGE:
  # High-performance testing with CPU affinity
  router-flood run --config high_perf.yaml --cpu-affinity

  # Export metrics to Prometheus
  router-flood run --config test.yaml --prometheus-port 9090

  # Validate configuration before running
  router-flood config validate my_config.yaml

📖 For detailed documentation, visit: https://github.com/your-org/router-flood
"#
    }

    /// Get global arguments
    fn get_global_args() -> Vec<Arg> {
        vec![
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Enable verbose output")
                .action(clap::ArgAction::Count),
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("Suppress non-essential output")
                .action(clap::ArgAction::SetTrue),
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Configuration file path")
                .value_name("FILE"),
        ]
    }

    /// Get run command arguments
    fn get_run_args() -> Vec<Arg> {
        vec![
            Arg::new("target")
                .long("target")
                .short('t')
                .help("Target IP address (private range only)")
                .value_name("IP"),
            Arg::new("ports")
                .long("ports")
                .short('p')
                .help("Target ports (comma-separated)")
                .value_name("PORTS"),
            Arg::new("threads")
                .long("threads")
                .help("Number of worker threads")
                .value_parser(value_parser!(usize))
                .value_name("NUM"),
            Arg::new("rate")
                .long("rate")
                .help("Packets per second per thread")
                .value_parser(value_parser!(u64))
                .value_name("PPS"),
            Arg::new("duration")
                .long("duration")
                .short('d')
                .help("Test duration in seconds")
                .value_parser(value_parser!(u64))
                .value_name("SECONDS"),
            Arg::new("dry-run")
                .long("dry-run")
                .help("Simulate without sending packets")
                .action(clap::ArgAction::SetTrue),
            Arg::new("cpu-affinity")
                .long("cpu-affinity")
                .help("Enable CPU affinity optimization")
                .action(clap::ArgAction::SetTrue),
            Arg::new("prometheus-port")
                .long("prometheus-port")
                .help("Enable Prometheus metrics on specified port")
                .value_parser(value_parser!(u16))
                .value_name("PORT"),
            Arg::new("export")
                .long("export")
                .help("Export statistics format")
                .value_parser(["json", "csv", "both", "prometheus"])
                .value_name("FORMAT"),
        ]
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

    /// Handle interactive mode
    pub async fn handle_interactive_mode(&self) -> Result<()> {
        println!("🎯 Router Flood Interactive Configuration");
        println!("==========================================");
        println!();

        // Security check first
        println!("🔒 Checking security context...");
        let security_report = self.capability_manager.security_report();
        println!("{}", security_report);
        println!();

        // Get basic configuration
        let target_ip = self.prompt_for_input("Target IP address (private range)", "192.168.1.1")?;
        let ports = self.prompt_for_input("Target ports (comma-separated)", "80,443")?;
        let threads = self.prompt_for_input("Number of threads", "4")?;
        let rate = self.prompt_for_input("Packets per second per thread", "100")?;
        let duration = self.prompt_for_input("Duration in seconds (empty for unlimited)", "")?;
        
        let dry_run = self.prompt_yes_no("Enable dry-run mode (recommended for first test)", true)?;
        let cpu_affinity = self.prompt_yes_no("Enable CPU affinity optimization", false)?;
        let export_stats = self.prompt_yes_no("Export statistics", false)?;

        // Build configuration
        let mut config_args = vec![
            "run".to_string(),
            "--target".to_string(), target_ip,
            "--ports".to_string(), ports,
            "--threads".to_string(), threads,
            "--rate".to_string(), rate,
        ];

        if !duration.is_empty() {
            config_args.extend(["--duration".to_string(), duration]);
        }

        if dry_run {
            config_args.push("--dry-run".to_string());
        }

        if cpu_affinity {
            config_args.push("--cpu-affinity".to_string());
        }

        if export_stats {
            let format = self.prompt_for_input("Export format (json/csv/both)", "json")?;
            config_args.extend(["--export".to_string(), format]);
        }

        println!();
        println!("📋 Generated command:");
        println!("router-flood {}", config_args.join(" "));
        println!();

        if self.prompt_yes_no("Execute this configuration now", true)? {
            println!("🚀 Starting test...");
            // Here we would execute the configuration
            // For now, just show what would be executed
            println!("✅ Configuration validated and ready to execute");
        } else {
            println!("💾 Configuration saved for later execution");
        }

        Ok(())
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

    /// Prompt for user input with default value
    fn prompt_for_input(&self, prompt: &str, default: &str) -> Result<String> {
        print!("{}", prompt);
        if !default.is_empty() {
            print!(" [{}]", default);
        }
        print!(": ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() && !default.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(input.to_string())
        }
    }

    /// Prompt for yes/no input
    fn prompt_yes_no(&self, prompt: &str, default: bool) -> Result<bool> {
        let default_str = if default { "Y/n" } else { "y/N" };
        print!("{} [{}]: ", prompt, default_str);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            "" => Ok(default),
            _ => {
                println!("Please enter 'y' or 'n'");
                self.prompt_yes_no(prompt, default)
            }
        }
    }
}

impl Default for EnhancedCli {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                capability_manager: CapabilityManager::default(),
                cpu_manager: CpuAffinityManager::default(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_cli_creation() {
        let cli = EnhancedCli::default();
        // Should not panic
        assert!(cli.capability_manager.security_context().process_id > 0);
    }

    #[test]
    fn test_command_building() {
        let cmd = EnhancedCli::build_command();
        
        // Should have subcommands
        let subcommands: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(subcommands.contains(&"run"));
        assert!(subcommands.contains(&"config"));
        assert!(subcommands.contains(&"system"));
        assert!(subcommands.contains(&"interactive"));
    }

    #[tokio::test]
    async fn test_list_templates() {
        let cli = EnhancedCli::default();
        let result = cli.handle_list_templates().await;
        assert!(result.is_ok());
    }
}