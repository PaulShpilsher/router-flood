//! Actionable error messages with specific guidance
//!
//! This module provides actionable, beginner-friendly error messages with
//! specific guidance on how to fix common issues.

use crate::error::{RouterFloodError, ConfigError, NetworkError, ValidationError};

/// User-friendly error display with actionable guidance
pub struct UserError {
    pub title: String,
    pub description: String,
    pub solution: String,
    pub examples: Vec<String>,
    pub severity: ErrorSeverity,
}

/// Error severity levels for better user experience
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Info - helpful tips, not blocking
    Info,
    /// Warning - potential issues, but can continue
    Warning,
    /// Error - blocking issues that must be fixed
    Error,
    /// Critical - serious issues that could cause problems
    Critical,
}

// Compatibility alias for backward compatibility
pub type EnhancedUserError = UserError;

impl UserError {
    /// Create a new actionable user error
    pub fn new(title: &str, description: &str, solution: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            solution: solution.to_string(),
            examples: Vec::new(),
            severity: ErrorSeverity::Error,
        }
    }

    /// Add examples to help users understand the fix
    pub fn with_examples(mut self, examples: Vec<&str>) -> Self {
        self.examples = examples.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set the error severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Display the error with formatting
    pub fn display(&self) {
        let icon = match self.severity {
            ErrorSeverity::Info => "ℹ️",
            ErrorSeverity::Warning => "⚠️",
            ErrorSeverity::Error => "❌",
            ErrorSeverity::Critical => "🚨",
        };

        let color = match self.severity {
            ErrorSeverity::Info => "\x1b[36m",     // Cyan
            ErrorSeverity::Warning => "\x1b[33m",  // Yellow
            ErrorSeverity::Error => "\x1b[31m",    // Red
            ErrorSeverity::Critical => "\x1b[35m", // Magenta
        };
        let reset = "\x1b[0m";

        println!();
        println!("{}{} {}{}", color, icon, self.title, reset);
        println!();
        println!("📋 Problem:");
        println!("   {}", self.description);
        println!();
        println!("🔧 Solution:");
        println!("   {}", self.solution);

        if !self.examples.is_empty() {
            println!();
            println!("💡 Examples:");
            for example in &self.examples {
                println!("   {}", example);
            }
        }

        println!();
    }
}

/// Convert RouterFloodError to actionable user-friendly error
pub fn to_user_error(error: &RouterFloodError) -> UserError {
    match error {
        RouterFloodError::Config(config_error) => config_error_to_user_friendly(config_error),
        RouterFloodError::Network(network_error) => network_error_to_user_friendly(network_error),
        RouterFloodError::Validation(validation_error) => validation_error_to_user_friendly(validation_error),
        RouterFloodError::Packet(packet_error) => {
            UserError::new(
                "Packet Generation Error",
                &format!("Failed to create network packets: {}", packet_error),
                "Check your network configuration and try again with different settings."
            )
            .with_examples(vec![
                "router-flood quick 192.168.1.1 --dry-run",
                "router-flood test --target 192.168.1.1 --intensity low"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        RouterFloodError::Stats(stats_error) => {
            UserError::new(
                "Statistics Export Error",
                &format!("Failed to export test results: {}", stats_error),
                "Check that you have write permissions in the current directory."
            )
            .with_examples(vec![
                "chmod 755 .",
                "router-flood test --target 192.168.1.1 --export json"
            ])
            .with_severity(ErrorSeverity::Warning)
        }
        RouterFloodError::System(system_error) => {
            UserError::new(
                "System Permission Error",
                &format!("System access denied: {}", system_error),
                "Try running with appropriate permissions or use dry-run mode."
            )
            .with_examples(vec![
                "sudo router-flood test --target 192.168.1.1",
                "router-flood quick 192.168.1.1 --dry-run"
            ])
            .with_severity(ErrorSeverity::Critical)
        }
        RouterFloodError::Audit(audit_error) => {
            UserError::new(
                "Audit Logging Error",
                &format!("Failed to create audit log: {}", audit_error),
                "Check write permissions or disable audit logging in config."
            )
            .with_examples(vec![
                "chmod 755 /var/log/",
                "router-flood test --target 192.168.1.1 --dry-run"
            ])
            .with_severity(ErrorSeverity::Warning)
        }
        RouterFloodError::Io(io_error) => {
            UserError::new(
                "File System Error",
                &format!("File operation failed: {}", io_error),
                "Check file permissions and available disk space."
            )
            .with_examples(vec![
                "ls -la",
                "df -h"
            ])
            .with_severity(ErrorSeverity::Error)
        }
    }
}

/// Convert configuration errors to user-friendly messages
fn config_error_to_user_friendly(error: &ConfigError) -> UserError {
    match error {
        ConfigError::FileNotFound(path) => {
            UserError::new(
                "Configuration File Not Found",
                &format!("Cannot find configuration file: {}", path),
                "Create a configuration file or use command-line options instead."
            )
            .with_examples(vec![
                "router-flood config create --output my-config.yaml",
                "router-flood quick 192.168.1.1 --dry-run",
                "router-flood test --target 192.168.1.1 --ports 80,443"
            ])
            .with_severity(ErrorSeverity::Warning)
        }
        ConfigError::ParseError(msg) => {
            UserError::new(
                "Configuration Format Error",
                &format!("Invalid configuration format: {}", msg),
                "Check your YAML syntax or create a new configuration file."
            )
            .with_examples(vec![
                "router-flood config validate my-config.yaml",
                "router-flood config create --output new-config.yaml",
                "yamllint my-config.yaml"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        ConfigError::InvalidValue { field, value, reason } => {
            create_invalid_value_error(field, value, reason)
        }
        ConfigError::MissingRequired(field) => {
            UserError::new(
                "Missing Required Setting",
                &format!("Required setting '{}' is missing from configuration.", field),
                "Add the missing setting to your configuration or command line."
            )
            .with_examples(vec![
                "router-flood test --target 192.168.1.1",
                "router-flood config create --output complete-config.yaml"
            ])
            .with_severity(ErrorSeverity::Error)
        }
    }
}

/// Create specific error messages for invalid values
fn create_invalid_value_error(field: &str, value: &str, reason: &str) -> UserError {
    match field {
        "target" | "target.ip" => {
            UserError::new(
                "Invalid Target IP Address",
                &format!("The IP address '{}' is not valid: {}", value, reason),
                "Use a valid private IP address from your local network."
            )
            .with_examples(vec![
                "router-flood quick 192.168.1.1",
                "router-flood test --target 10.0.0.1",
                "router-flood test --target 172.16.0.1"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        "ports" | "target.ports" => {
            UserError::new(
                "Invalid Port Configuration",
                &format!("Port setting '{}' is invalid: {}", value, reason),
                "Use valid port numbers between 1-65535, separated by commas."
            )
            .with_examples(vec![
                "router-flood test --target 192.168.1.1 --ports 80",
                "router-flood test --target 192.168.1.1 --ports 80,443",
                "router-flood test --target 192.168.1.1 --ports 22,80,443,8080"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        "intensity" => {
            UserError::new(
                "Invalid Test Intensity",
                &format!("Intensity level '{}' is not recognized: {}", value, reason),
                "Use 'low', 'medium', or 'high' for test intensity."
            )
            .with_examples(vec![
                "router-flood test --target 192.168.1.1 --intensity low",
                "router-flood test --target 192.168.1.1 --intensity medium",
                "router-flood test --target 192.168.1.1 --intensity high"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        "duration" | "test.duration" => {
            UserError::new(
                "Invalid Test Duration",
                &format!("Duration '{}' is invalid: {}", value, reason),
                "Use a duration between 1-3600 seconds (1 hour maximum)."
            )
            .with_examples(vec![
                "router-flood test --target 192.168.1.1 --duration 30",
                "router-flood test --target 192.168.1.1 --duration 120",
                "router-flood quick 192.168.1.1  # Uses 10 second default"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        "export" | "export.format" => {
            UserError::new(
                "Invalid Export Format",
                &format!("Export format '{}' is not supported: {}", value, reason),
                "Use 'json' for structured data or 'csv' for spreadsheets."
            )
            .with_examples(vec![
                "router-flood advanced --target 192.168.1.1 --export json",
                "router-flood advanced --target 192.168.1.1 --export csv"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        _ => {
            UserError::new(
                "Configuration Error",
                &format!("Setting '{}' has invalid value '{}': {}", field, value, reason),
                "Check the configuration documentation and fix the invalid setting."
            )
            .with_examples(vec![
                "router-flood config examples",
                "router-flood examples"
            ])
            .with_severity(ErrorSeverity::Error)
        }
    }
}

/// Convert network errors to user-friendly messages
fn network_error_to_user_friendly(error: &NetworkError) -> UserError {
    match error {
        NetworkError::InterfaceNotFound(name) => {
            UserError::new(
                "Network Interface Not Found",
                &format!("Cannot find network interface '{}' on this system.", name),
                "Check available interfaces or let the system auto-detect."
            )
            .with_examples(vec![
                "ip link show",
                "router-flood test --target 192.168.1.1  # Auto-detect interface",
                "ifconfig -a"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        NetworkError::ChannelCreation(msg) => {
            UserError::new(
                "Network Access Error",
                &format!("Cannot access network: {}", msg),
                "Try running with sudo or use dry-run mode for testing."
            )
            .with_examples(vec![
                "sudo router-flood test --target 192.168.1.1",
                "router-flood quick 192.168.1.1 --dry-run"
            ])
            .with_severity(ErrorSeverity::Critical)
        }
        NetworkError::PacketSend(msg) => {
            UserError::new(
                "Packet Transmission Error",
                &format!("Failed to send network packets: {}", msg),
                "Check network connectivity and permissions."
            )
            .with_examples(vec![
                "ping 192.168.1.1",
                "router-flood quick 192.168.1.1 --dry-run",
                "sudo router-flood test --target 192.168.1.1"
            ])
            .with_severity(ErrorSeverity::Error)
        }
        NetworkError::InvalidAddress(addr) => {
            UserError::new(
                "Invalid Network Address",
                &format!("Network address '{}' is not valid.", addr),
                "Use a valid IP address from your local network."
            )
            .with_examples(vec![
                "router-flood quick 192.168.1.1",
                "router-flood test --target 10.0.0.1",
                "nmap -sn 192.168.1.0/24  # Discover local IPs"
            ])
            .with_severity(ErrorSeverity::Error)
        }
    }
}

/// Convert validation errors to user-friendly messages
fn validation_error_to_user_friendly(error: &ValidationError) -> UserError {
    match error {
        ValidationError::InvalidIpRange { ip, reason } => {
            UserError::new(
                "IP Address Not Allowed",
                &format!("IP address '{}' cannot be used: {}", ip, reason),
                "Use only private IP addresses for safety (192.168.x.x, 10.x.x.x, 172.16-31.x.x)."
            )
            .with_examples(vec![
                "router-flood quick 192.168.1.1  # Home network",
                "router-flood test --target 10.0.0.1  # Corporate network",
                "router-flood test --target 172.16.0.1  # Enterprise network"
            ])
            .with_severity(ErrorSeverity::Critical)
        }
        ValidationError::ExceedsLimit { field, value, limit } => {
            UserError::new(
                "Safety Limit Exceeded",
                &format!("Value {} for '{}' exceeds safety limit of {}.", value, field, limit),
                "Reduce the value to stay within safety limits."
            )
            .with_examples(vec![
                "router-flood test --target 192.168.1.1 --intensity low",
                "router-flood test --target 192.168.1.1 --duration 30"
            ])
            .with_severity(ErrorSeverity::Warning)
        }
        ValidationError::SystemRequirement(msg) => {
            UserError::new(
                "System Requirement Not Met",
                &format!("System requirement missing: {}", msg),
                "Install required system components or use dry-run mode."
            )
            .with_examples(vec![
                "router-flood quick 192.168.1.1 --dry-run",
                "sudo apt-get install libpcap-dev  # On Ubuntu/Debian",
                "sudo yum install libpcap-devel  # On CentOS/RHEL"
            ])
            .with_severity(ErrorSeverity::Critical)
        }
        ValidationError::PrivilegeRequired(msg) => {
            UserError::new(
                "Insufficient Privileges",
                &format!("Additional privileges required: {}", msg),
                "Run with sudo for network access or use dry-run mode for testing."
            )
            .with_examples(vec![
                "sudo router-flood test --target 192.168.1.1",
                "router-flood quick 192.168.1.1 --dry-run  # No privileges needed"
            ])
            .with_severity(ErrorSeverity::Critical)
        }
        ValidationError::PermissionDenied(msg) => {
            UserError::new(
                "Permission Denied",
                &format!("Access denied: {}", msg),
                "Check file permissions or run with appropriate privileges."
            )
            .with_examples(vec![
                "sudo router-flood test --target 192.168.1.1",
                "chmod 755 .",
                "router-flood quick 192.168.1.1 --dry-run"
            ])
            .with_severity(ErrorSeverity::Error)
        }
    }
}

/// Display actionable user-friendly error
pub fn display_actionable_user_error(error: &RouterFloodError) {
    let user_error = to_user_error(error);
    user_error.display();
    
    // Add general help footer
    println!("📚 Need more help?");
    println!("   router-flood examples     # Show usage examples");
    println!("   router-flood config examples  # Configuration examples");
    println!("   router-flood --help       # Show all options");
    println!();
}

// Compatibility alias for backward compatibility
pub use display_actionable_user_error as display_enhanced_user_error;

/// Quick help for common scenarios
pub fn show_quick_help() {
    println!(r#"🚀 Quick Help - Router Flood

🎯 GETTING STARTED:
  router-flood quick 192.168.1.1 --dry-run    # Safest way to start
  router-flood test --target 192.168.1.1      # Standard test
  router-flood examples                        # Show more examples

🛡️ SAFETY FIRST:
  • Always start with --dry-run for new targets
  • Only use private IP addresses (192.168.x.x, 10.x.x.x, 172.16-31.x.x)
  • Use 'quick' mode when learning
  • Start with low intensity and short duration

❓ COMMON ISSUES:
  • Permission denied → Try 'sudo' or use --dry-run
  • Invalid IP → Use private IP addresses only
  • Network error → Check connectivity with 'ping'
  • Config error → Use 'router-flood config create'
"#);
}