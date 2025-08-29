//! CLI argument parsing and command structure
//!
//! This module handles the command-line argument parsing and command structure definition.

use clap::{Arg, Command, value_parser};

/// Build the command structure for the CLI
pub struct CliParser;

impl CliParser {
    /// Build the enhanced command structure
    pub fn build_command() -> Command {
        Command::new("router-flood")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Educational DDoS simulation for local network testing")
            .long_about(Self::get_enhanced_help())
            .subcommand_required(false)
            .arg_required_else_help(false)
            .subcommand(Self::build_run_command())
            .subcommand(Self::build_config_command())
            .subcommand(Self::build_system_command())
            .subcommand(
                Command::new("interactive")
                    .about("Interactive configuration mode")
            )
            .args(Self::get_global_args())
    }

    /// Build the 'run' subcommand
    fn build_run_command() -> Command {
        Command::new("run")
            .about("Run network stress test")
            .args(Self::get_run_args())
    }

    /// Build the 'config' subcommand
    fn build_config_command() -> Command {
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
    }

    /// Build the 'system' subcommand
    fn build_system_command() -> Command {
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
}