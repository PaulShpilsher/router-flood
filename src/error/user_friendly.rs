//! User-friendly error messages and suggestions
//!
//! This module provides enhanced error formatting with actionable suggestions
//! to improve the user experience when configuration or validation errors occur.

use crate::error::*;
use std::fmt;

/// Enhanced error display with user-friendly messages and suggestions
pub struct UserFriendlyError<'a> {
    error: &'a RouterFloodError,
}

impl<'a> UserFriendlyError<'a> {
    pub fn new(error: &'a RouterFloodError) -> Self {
        Self { error }
    }
}

impl<'a> fmt::Display for UserFriendlyError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error {
            RouterFloodError::Validation(ValidationError::InvalidIpRange { ip, reason }) => {
                writeln!(f, "❌ Invalid target IP address: {}", ip)?;
                writeln!(f, "   Reason: {}", reason)?;
                writeln!(f)?;
                writeln!(f, "💡 Suggestions:")?;
                writeln!(f, "   • Use a private IP address from these ranges:")?;
                writeln!(f, "     - 192.168.0.0/16 (e.g., 192.168.1.1)")?;
                writeln!(f, "     - 10.0.0.0/8 (e.g., 10.0.0.1)")?;
                writeln!(f, "     - 172.16.0.0/12 (e.g., 172.16.0.1)")?;
                writeln!(f, "   • For testing, try: --target 192.168.1.1")?;
                writeln!(f, "   • Use --dry-run to test configuration without sending packets")
            }
            RouterFloodError::Validation(ValidationError::ExceedsLimit { field, value, limit }) => {
                writeln!(f, "❌ Configuration limit exceeded: {}", field)?;
                writeln!(f, "   Current value: {}, Maximum allowed: {}", value, limit)?;
                writeln!(f)?;
                writeln!(f, "💡 Suggestions:")?;
                match field.as_str() {
                    "threads" => {
                        writeln!(f, "   • Reduce thread count: --threads {}", limit)?;
                        writeln!(f, "   • For most scenarios, 4-8 threads are sufficient")?;
                        writeln!(f, "   • Higher thread counts may not improve performance")
                    }
                    "packet_rate" => {
                        writeln!(f, "   • Reduce packet rate: --rate {}", limit)?;
                        writeln!(f, "   • Start with lower rates (100-500 pps) for testing")?;
                        writeln!(f, "   • Monitor system resources when increasing rates")
                    }
                    _ => writeln!(f, "   • Reduce {} to {} or below", field, limit)
                }
            }
            RouterFloodError::Validation(ValidationError::PrivilegeRequired(msg)) => {
                writeln!(f, "❌ Insufficient privileges: {}", msg)?;
                writeln!(f)?;
                writeln!(f, "💡 Solutions:")?;
                writeln!(f, "   • Run with sudo: sudo ./router-flood [options]")?;
                writeln!(f, "   • Use dry-run mode: --dry-run (no privileges required)")?;
                writeln!(f, "   • Test configuration first: --dry-run --target 192.168.1.1")
            }
            RouterFloodError::Network(NetworkError::InterfaceNotFound(name)) => {
                writeln!(f, "❌ Network interface not found: {}", name)?;
                writeln!(f)?;
                writeln!(f, "💡 Solutions:")?;
                writeln!(f, "   • List available interfaces: --list-interfaces")?;
                writeln!(f, "   • Let the tool auto-detect: remove --interface option")?;
                writeln!(f, "   • Check interface is up: ip link show {}", name)?;
                writeln!(f, "   • Common interface names: eth0, wlan0, enp0s3")
            }
            RouterFloodError::Config(ConfigError::FileNotFound(path)) => {
                writeln!(f, "❌ Configuration file not found: {}", path)?;
                writeln!(f)?;
                writeln!(f, "💡 Solutions:")?;
                writeln!(f, "   • Create config file: cp router_flood_config.yaml {}", path)?;
                writeln!(f, "   • Use default config: remove --config option")?;
                writeln!(f, "   • Check file path and permissions")
            }
            RouterFloodError::Config(ConfigError::ParseError(msg)) => {
                writeln!(f, "❌ Configuration file parsing error:")?;
                writeln!(f, "   {}", msg)?;
                writeln!(f)?;
                writeln!(f, "💡 Solutions:")?;
                writeln!(f, "   • Check YAML syntax with: yamllint config.yaml")?;
                writeln!(f, "   • Validate against example: router_flood_config.yaml")?;
                writeln!(f, "   • Common issues: incorrect indentation, missing quotes")
            }
            _ => {
                // Fallback to standard error display
                write!(f, "{}", self.error)
            }
        }
    }
}

/// Helper function to display user-friendly errors
pub fn display_user_friendly_error(error: &RouterFloodError) {
    eprintln!("{}", UserFriendlyError::new(error));
}

/// Enhanced error context for better debugging
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T>;
    fn with_suggestion(self, suggestion: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<RouterFloodError>,
{
    fn with_context(self, _context: &str) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            // Add context to error (this would require extending error types)
            error
        })
    }

    fn with_suggestion(self, _suggestion: &str) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_friendly_ip_error() {
        let error = RouterFloodError::Validation(ValidationError::InvalidIpRange {
            ip: "8.8.8.8".to_string(),
            reason: "Public IP not allowed".to_string(),
        });
        
        let friendly = UserFriendlyError::new(&error);
        let output = format!("{}", friendly);
        
        assert!(output.contains("❌ Invalid target IP address"));
        assert!(output.contains("💡 Suggestions"));
        assert!(output.contains("192.168.1.1"));
    }

    #[test]
    fn test_user_friendly_limit_error() {
        let error = RouterFloodError::Validation(ValidationError::ExceedsLimit {
            field: "threads".to_string(),
            value: 200,
            limit: 100,
        });
        
        let friendly = UserFriendlyError::new(&error);
        let output = format!("{}", friendly);
        
        assert!(output.contains("❌ Configuration limit exceeded"));
        assert!(output.contains("threads"));
        assert!(output.contains("4-8 threads"));
    }
}