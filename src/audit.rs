use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::net::IpAddr;
use tracing::info;

pub const AUDIT_LOG_FILE: &str = "router_flood_audit.log";

/// Audit logging entry creation
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntry {
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

pub fn create_audit_entry(
    target_ip: &IpAddr,
    target_ports: &[u16],
    threads: usize,
    packet_rate: u64,
    duration: Option<u64>,
    interface: Option<&str>,
    session_id: &str,
) -> Result<(), String> {
    let entry = AuditEntry {
        timestamp: Utc::now(),
        event_type: "flood_simulation_start".to_string(),
        target_ip: target_ip.to_string(),
        target_ports: target_ports.to_vec(),
        threads,
        packet_rate,
        duration,
        user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
        interface: interface.map(|s| s.to_string()),
        session_id: session_id.to_string(),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(AUDIT_LOG_FILE)
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
