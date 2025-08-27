//! Command-line interface handling
//!
//! This module handles all CLI argument parsing, validation, and help text
//! generation, keeping main.rs focused on orchestration.

use clap::{Arg, ArgMatches, Command};
use std::str::FromStr;
use tracing::info;

use crate::constants::{defaults, MAX_THREADS};
use crate::error::{ConfigError, Result, RouterFloodError};
use crate::config::{Config, ExportFormat};

/// Generate comprehensive help text with examples
fn get_long_help() -> &'static str {
    r#"Educational DDoS simulation for local network testing with multi-protocol support

🎯 SAFETY FIRST:
  • Only works with private IP ranges (192.168.x.x, 10.x.x.x, 172.16-31.x.x)
  • Built-in rate limiting and safety checks
  • Comprehensive audit logging
  • Use --dry-run for safe configuration testing

📚 COMMON EXAMPLES:

  Basic simulation:
    sudo ./router-flood --target 192.168.1.1 --ports 80,443 --threads 4 --rate 100

  Safe testing (no packets sent):
    ./router-flood --target 192.168.1.1 --ports 80 --dry-run

  Perfect simulation (100% success rate):
    ./router-flood --target 192.168.1.1 --ports 80 --dry-run --perfect-simulation

  High-performance test:
    sudo ./router-flood --target 10.0.0.1 --ports 80,443,22,53 --threads 8 --rate 500 --duration 60

  With configuration file:
    sudo ./router-flood --config my_test.yaml --export json

  List available interfaces:
    ./router-flood --list-interfaces

🔧 CONFIGURATION:
  Use YAML files for complex scenarios. See router_flood_config.yaml for examples.
  CLI arguments override configuration file values.

📊 MONITORING:
  Real-time statistics are displayed during execution.
  Use --export to save results in JSON or CSV format.

⚠️  REQUIREMENTS:
  • Root privileges (unless using --dry-run)
  • Network interface access
  • Target must be in private IP range

📖 For more information, see the README.md file.
"#
}

/// Parse command line arguments and return matches
pub fn parse_arguments() -> ArgMatches {
    Command::new("Router Flood - Enhanced Network Stress Tester")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Educational DDoS simulation for local network testing with multi-protocol support")
        .long_about(get_long_help())
        .arg(
            Arg::new("target")
                .long("target")
                .short('t')
                .value_name("IP")
                .help("Target router IP (must be private range)")
                .long_help("Target router IP address. Must be in private ranges:\n  • 192.168.0.0/16 (e.g., 192.168.1.1)\n  • 10.0.0.0/8 (e.g., 10.0.0.1)\n  • 172.16.0.0/12 (e.g., 172.16.0.1)")
                .required_unless_present_any(&["config", "list-interfaces"]),
        )
        .arg(
            Arg::new("ports")
                .long("ports")
                .short('p')
                .value_name("PORTS")
                .help("Target ports (comma-separated, e.g., 80,443,22)")
                .long_help("Target ports for testing (comma-separated).\nCommon ports: 80 (HTTP), 443 (HTTPS), 22 (SSH), 53 (DNS), 21 (FTP), 25 (SMTP)")
                .required_unless_present_any(&["config", "list-interfaces"]),
        )
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_name("NUM")
                .help(&format!("Number of async tasks (default: {}, max: {})", 
                    defaults::THREAD_COUNT, MAX_THREADS))
                .default_value("4"),
        )
        .arg(
            Arg::new("rate")
                .long("rate")
                .value_name("PPS")
                .help(&format!("Packets per second per thread (default: {})", 
                    defaults::PACKET_RATE))
                .default_value("100"),
        )
        .arg(
            Arg::new("duration")
                .long("duration")
                .short('d')
                .value_name("SECONDS")
                .help("Test duration in seconds (default: unlimited)"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("YAML configuration file path"),
        )
        .arg(
            Arg::new("interface")
                .long("interface")
                .short('i')
                .value_name("NAME")
                .help("Network interface to use (default: auto-detect)"),
        )
        .arg(
            Arg::new("export")
                .long("export")
                .value_name("FORMAT")
                .help("Export statistics (json, csv, both)")
                .value_parser(["json", "csv", "both"]),
        )
        .arg(
            Arg::new("list-interfaces")
                .long("list-interfaces")
                .help("List available network interfaces")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Simulate the attack without sending actual packets (safe testing)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("perfect-simulation")
                .long("perfect-simulation")
                .help("Use 100% success rate in dry-run mode (no simulated failures)")
                .long_help("When used with --dry-run, ensures 100% packet success rate.\nBy default, dry-run mode simulates 98% success rate for realistic training.\nThis flag removes simulated failures for pure configuration validation.")
                .action(clap::ArgAction::SetTrue)
                .requires("dry-run"),
        )
        .get_matches()
}

/// Process CLI arguments and merge with config
pub fn process_cli_config(matches: &ArgMatches, mut config: Config) -> Result<Config> {
    // Override config with CLI arguments
    if let Some(target) = matches.get_one::<String>("target") {
        config.target.ip = target.clone();
    }

    if let Some(ports_str) = matches.get_one::<String>("ports") {
        config.target.ports = parse_ports(ports_str)?;
    }

    if let Some(threads_str) = matches.get_one::<String>("threads") {
        config.attack.threads = parse_positive_number(threads_str, "threads")?;
    }

    if let Some(rate_str) = matches.get_one::<String>("rate") {
        config.attack.packet_rate = parse_positive_number(rate_str, "rate")?;
    }

    if let Some(duration_str) = matches.get_one::<String>("duration") {
        config.attack.duration = Some(parse_positive_number(duration_str, "duration")?);
    }

    if let Some(interface) = matches.get_one::<String>("interface") {
        config.target.interface = Some(interface.clone());
    }

    if let Some(export_format) = matches.get_one::<String>("export") {
        config.export.enabled = true;
        config.export.format = parse_export_format(export_format)?;
    }

    // Handle dry-run flag
    let cli_dry_run = matches.get_flag("dry-run");
    if cli_dry_run || config.safety.dry_run {
        config.safety.dry_run = true;
        if cli_dry_run {
            info!("🔍 DRY-RUN MODE ENABLED (CLI) - No packets will be sent");
        } else {
            info!("🔍 DRY-RUN MODE ENABLED (CONFIG) - No packets will be sent");
        }
    }

    // Handle perfect simulation flag
    let cli_perfect_simulation = matches.get_flag("perfect-simulation");
    if cli_perfect_simulation {
        config.safety.perfect_simulation = true;
        info!("✨ PERFECT SIMULATION MODE ENABLED - 100% success rate in dry-run");
    }

    Ok(config)
}

/// Check if any pre-execution commands were requested
pub fn handle_pre_execution_commands(matches: &ArgMatches) -> bool {
    if matches.get_flag("list-interfaces") {
        list_network_interfaces();
        return true;
    }
    false
}

/// Parse comma-separated ports
pub fn parse_ports(ports_str: &str) -> Result<Vec<u16>> {
    ports_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<u16>()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "ports".to_string(),
                    value: s.trim().to_string(),
                    reason: "must be a valid port number (1-65535)".to_string(),
                })
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(RouterFloodError::from)
}

/// Parse positive numbers with field context
pub fn parse_positive_number<T>(value_str: &str, field: &str) -> Result<T>
where
    T: FromStr + PartialOrd + Default,
    T::Err: std::fmt::Display,
{
    let value = value_str.parse::<T>().map_err(|e| ConfigError::InvalidValue {
        field: field.to_string(),
        value: value_str.to_string(),
        reason: e.to_string(),
    })?;

    if value <= T::default() {
        return Err(ConfigError::InvalidValue {
            field: field.to_string(),
            value: value_str.to_string(),
            reason: "must be greater than 0".to_string(),
        }.into());
    }

    Ok(value)
}

/// Parse export format string
pub fn parse_export_format(format_str: &str) -> Result<ExportFormat> {
    match format_str.to_lowercase().as_str() {
        "json" => Ok(ExportFormat::Json),
        "csv" => Ok(ExportFormat::Csv),
        "both" => Ok(ExportFormat::Both),
        _ => Err(ConfigError::InvalidValue {
            field: "export".to_string(),
            value: format_str.to_string(),
            reason: "must be 'json', 'csv', or 'both'".to_string(),
        }.into()),
    }
}

/// List available network interfaces with pretty formatting
fn list_network_interfaces() {
    use crate::network::list_network_interfaces as list_interfaces;
    
    println!("\n🌐 Available Network Interfaces:\n");
    println!("{:<20} {:<10} {:<15} {}", "Interface", "Status", "IPv4", "IPv6");
    println!("{}", "─".repeat(80));
    
    for iface in list_interfaces() {
        let status = if iface.is_up() {
            "🟢 Up"
        } else {
            "🔴 Down"
        };
        
        let mut ipv4_addrs = Vec::new();
        let mut ipv6_addrs = Vec::new();
        
        // Extract and format IP addresses
        for ip in &iface.ips {
            match ip {
                pnet::ipnetwork::IpNetwork::V4(ipv4_net) => {
                    ipv4_addrs.push(format!("{}/{}", ipv4_net.ip(), ipv4_net.prefix()));
                }
                pnet::ipnetwork::IpNetwork::V6(ipv6_net) => {
                    ipv6_addrs.push(format!("{}/{}", ipv6_net.ip(), ipv6_net.prefix()));
                }
            }
        }
        
        let ipv4_display = if ipv4_addrs.is_empty() {
            "─".to_string()
        } else {
            ipv4_addrs.join(", ")
        };
        
        let ipv6_display = if ipv6_addrs.is_empty() {
            "─".to_string()
        } else {
            // Truncate long IPv6 addresses for display
            let ipv6_str = ipv6_addrs.join(", ");
            if ipv6_str.len() > 25 {
                format!("{}...", &ipv6_str[..22])
            } else {
                ipv6_str
            }
        };
        
        println!(
            "{:<20} {:<8} {:<15} {}",
            format!("📡 {}", iface.name),
            status,
            ipv4_display,
            ipv6_display
        );
        
        // Add description if available and non-empty
        if !iface.description.is_empty() {
            println!("    📝 {}", iface.description);
        }
        
        // Add separator for readability
        if ipv4_addrs.len() > 1 || ipv6_addrs.len() > 1 {
            println!("    💡 Multiple addresses configured");
        }
        println!();
    }
}

