use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use tracing::info;
use crate::config::Config;

pub const DEFAULT_AUDIT_LOG_FILE: &str = "router_flood_audit.log";

/// Common audit event types
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Start,
    Stop,
    Error,
    ConfigChange,
    SecurityViolation,
    RateLimitExceeded,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "engine_start",
            Self::Stop => "engine_stop",
            Self::Error => "error",
            Self::ConfigChange => "config_change",
            Self::SecurityViolation => "security_violation",
            Self::RateLimitExceeded => "rate_limit_exceeded",
        }
    }
}

/// Audit logger with configurable output
///
/// # Example Usage
/// ```no_run
/// use router_flood::security::audit::{AuditLogger, EventType};
/// use router_flood::Config;
/// use std::net::IpAddr;
/// 
/// # fn example() -> std::io::Result<()> {
/// // Create from config
/// let config = Config::default();
/// let audit_logger = AuditLogger::from_config(&config);
/// 
/// // Or create with custom settings
/// let audit_logger = AuditLogger::new(Some("/var/log/audit.log".to_string()), true);
/// 
/// // Log with typed event
/// let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
/// let ports = vec![80, 443];
/// let threads = 4;
/// let packet_rate = 1000;
/// let duration = Some(60);
/// let interface = Some("eth0");
/// let session_id = "test-session";
/// 
/// let _ = audit_logger.log_event(
///     EventType::Start,
///     &target_ip,
///     &ports,
///     threads,
///     packet_rate,
///     duration,
///     interface,
///     session_id,
/// );
/// # Ok(())
/// # }
/// ```
pub struct AuditLogger {
    enabled: bool,
    log_file: PathBuf,
    user: String,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_file: Option<String>, enabled: bool) -> Self {
        let log_file = log_file
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_AUDIT_LOG_FILE));
        
        // Get user once at initialization
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))  // Windows fallback
            .unwrap_or_else(|_| "unknown".to_string());
        
        Self { enabled, log_file, user }
    }
    
    /// Create an audit logger from configuration
    pub fn from_config(config: &Config) -> Self {
        Self::new(Some(config.audit.log_file.clone()), config.audit.enabled)
    }
    
    /// Create an audit entry with custom event type string
    pub fn create_entry(
        &self,
        event_type: &str,
        target_ip: &IpAddr,
        target_ports: &[u16],
        threads: usize,
        packet_rate: u64,
        duration: Option<u64>,
        interface: Option<&str>,
        session_id: &str,
    ) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            target_ip: target_ip.to_string(),
            target_ports: target_ports.to_vec(),
            threads,
            packet_rate,
            duration,
            user: self.user.clone(),
            interface: interface.map(|s| s.to_string()),
            session_id: session_id.to_string(),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(|e| format!("Failed to open audit log: {}", e))?;

        let log_line = format!(
            "{}\n",
            serde_json::to_string(&entry)
                .map_err(|e| format!("Failed to serialize audit entry: {}", e))?
        );

        file.write_all(log_line.as_bytes())
            .map_err(|e| format!("Failed to write audit entry: {}", e))?;

        info!("Audit entry created for session {}", session_id);
        Ok(())
    }
    
    /// Create an audit entry with typed event
    pub fn log_event(
        &self,
        event: EventType,
        target_ip: &IpAddr,
        target_ports: &[u16],
        threads: usize,
        packet_rate: u64,
        duration: Option<u64>,
        interface: Option<&str>,
        session_id: &str,
    ) -> Result<(), String> {
        self.create_entry(
            event.as_str(),
            target_ip,
            target_ports,
            threads,
            packet_rate,
            duration,
            interface,
            session_id,
        )
    }
    
    /// Check if audit logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get the log file path
    pub fn log_file(&self) -> &PathBuf {
        &self.log_file
    }
    
    /// Get the user associated with this logger
    pub fn user(&self) -> &str {
        &self.user
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(None, true)
    }
}

/// Audit entry data structure
#[derive(Debug, Serialize, Deserialize)]
struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub target_ip: String,
    pub target_ports: Vec<u16>,
    pub threads: usize,
    pub packet_rate: u64,
    pub duration: Option<u64>,
    pub user: String,
    pub interface: Option<String>,
    pub session_id: String,
}

