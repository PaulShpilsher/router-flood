//! Linux capabilities-based security system
//!
//! This module provides capability-based security instead of requiring full root privileges,
//! following the principle of least privilege for enhanced security.

use crate::error::{SystemError, ValidationError, Result};
use std::fs;
use std::process;
use tokio::io::AsyncWriteExt;

/// Required Linux capabilities for network operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredCapability {
    /// CAP_NET_RAW - Required for raw socket creation
    NetRaw,
    /// CAP_NET_ADMIN - Required for network interface manipulation
    NetAdmin,
}

/// Security context with capability information
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub has_net_raw: bool,
    pub has_net_admin: bool,
    pub effective_uid: u32,
    pub real_uid: u32,
    pub process_id: u32,
    pub capabilities_available: bool,
}

/// Capability-based security manager
#[derive(Clone)]
pub struct Capabilities {
    context: SecurityContext,
}

impl Capabilities {
    /// Create a new capability manager and detect current security context
    pub fn new() -> Result<Self> {
        let context = Self::detect_security_context()?;
        Ok(Self { context })
    }

    /// Get the current security context
    pub fn security_context(&self) -> &SecurityContext {
        &self.context
    }

    /// Check if we have the required capabilities for network operations
    pub fn has_required_capabilities(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            // Dry run mode doesn't require any special capabilities
            return Ok(());
        }

        // Check for raw socket capability
        if !self.context.has_net_raw {
            return Err(ValidationError::PrivilegeRequired(
                "CAP_NET_RAW capability required for raw socket operations. \
                 Run with sudo or grant CAP_NET_RAW capability."
            ).into());
        }

        Ok(())
    }

    /// Validate that we're not running as root unnecessarily
    pub fn validate_privilege_level(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            // Dry run is safe regardless of privileges
            return Ok(());
        }

        if self.context.effective_uid == 0 && self.context.real_uid == 0 {
            // Running as root - warn but allow
            eprintln!("⚠️  WARNING: Running as root user. Consider using capabilities instead:");
            eprintln!("   sudo setcap cap_net_raw+ep ./router-flood");
            eprintln!("   Then run as regular user for better security.");
        }

        Ok(())
    }

    /// Drop unnecessary privileges after initialization
    pub fn drop_privileges(&self) -> Result<()> {
        // In a full implementation, this would drop capabilities we don't need
        // For now, we just validate the current state
        if self.context.effective_uid == 0 {
            eprintln!("💡 Consider running with minimal privileges using capabilities");
        }
        Ok(())
    }

    /// Detect the current security context
    fn detect_security_context() -> Result<SecurityContext> {
        let effective_uid = unsafe { libc::geteuid() };
        let real_uid = unsafe { libc::getuid() };
        let process_id = process::id();

        // Try to detect capabilities
        let (has_net_raw, has_net_admin, capabilities_available) = Self::detect_capabilities();

        Ok(SecurityContext {
            has_net_raw,
            has_net_admin,
            effective_uid,
            real_uid,
            process_id,
            capabilities_available,
        })
    }

    /// Detect available Linux capabilities
    fn detect_capabilities() -> (bool, bool, bool) {
        // Try to read capabilities from /proc/self/status
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            let has_net_raw = Self::parse_capability(&status, "CapEff", 13); // CAP_NET_RAW = 13
            let has_net_admin = Self::parse_capability(&status, "CapEff", 12); // CAP_NET_ADMIN = 12
            return (has_net_raw, has_net_admin, true);
        }

        // Fallback: check if running as root
        let effective_uid = unsafe { libc::geteuid() };
        let is_root = effective_uid == 0;
        (is_root, is_root, false)
    }

    /// Parse capability from /proc/self/status
    pub fn parse_capability(status: &str, cap_type: &str, cap_number: u8) -> bool {
        for line in status.lines() {
            if line.starts_with(cap_type) {
                if let Some(hex_caps) = line.split_whitespace().nth(1) {
                    if let Ok(caps) = u64::from_str_radix(hex_caps, 16) {
                        let cap_bit = 1u64 << cap_number;
                        return (caps & cap_bit) != 0;
                    }
                }
            }
        }
        false
    }

    /// Get a human-readable security status report
    pub fn security_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("🔒 Security Context Report:\n");
        report.push_str(&format!("   Process ID: {}\n", self.context.process_id));
        report.push_str(&format!("   Real UID: {}\n", self.context.real_uid));
        report.push_str(&format!("   Effective UID: {}\n", self.context.effective_uid));
        report.push_str(&format!("   Capabilities Available: {}\n", self.context.capabilities_available));
        
        if self.context.capabilities_available {
            report.push_str("   Capabilities:\n");
            report.push_str(&format!("     CAP_NET_RAW: {}\n", 
                if self.context.has_net_raw { "✅ Available" } else { "❌ Missing" }));
            report.push_str(&format!("     CAP_NET_ADMIN: {}\n", 
                if self.context.has_net_admin { "✅ Available" } else { "❌ Missing" }));
        } else {
            report.push_str("   Capability detection not available\n");
        }

        // Security recommendations
        report.push_str("\n💡 Security Recommendations:\n");
        
        if self.context.effective_uid == 0 {
            report.push_str("   • Consider using capabilities instead of root:\n");
            report.push_str("     sudo setcap cap_net_raw+ep ./router-flood\n");
            report.push_str("   • Run as regular user after setting capabilities\n");
        } else if !self.context.has_net_raw {
            report.push_str("   • Grant CAP_NET_RAW capability:\n");
            report.push_str("     sudo setcap cap_net_raw+ep ./router-flood\n");
            report.push_str("   • Or use --dry-run for testing without privileges\n");
        } else {
            report.push_str("   • ✅ Good security posture detected\n");
        }

        report
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback security context if detection fails
            Self {
                context: SecurityContext {
                    has_net_raw: false,
                    has_net_admin: false,
                    effective_uid: unsafe { libc::geteuid() },
                    real_uid: unsafe { libc::getuid() },
                    process_id: process::id(),
                    capabilities_available: false,
                },
            }
        })
    }
}

/// Enhanced audit logging with tamper detection
pub struct TamperProofAuditLog {
    log_file: String,
    session_id: String,
    previous_hash: [u8; 32],
    entry_count: u64,
}

impl TamperProofAuditLog {
    /// Create a new tamper-proof audit log
    pub fn new(log_file: &str, session_id: &str) -> Result<Self> {
        let log_file = log_file.to_string();
        let session_id = session_id.to_string();
        
        // Initialize with genesis hash
        let genesis_data = format!("GENESIS:{}", session_id);
        let previous_hash = Self::calculate_hash(genesis_data.as_bytes());
        
        let mut audit_log = Self {
            log_file,
            session_id,
            previous_hash,
            entry_count: 0,
        };
        
        // Write genesis entry
        audit_log.write_genesis_entry()?;
        
        Ok(audit_log)
    }

    /// Write a new audit entry with integrity protection
    pub async fn write_entry(&mut self, event_type: &str, details: &str) -> Result<()> {
        let timestamp = chrono::Utc::now();
        let entry_data = format!(
            "{}|{}|{}|{}|{}|{}",
            self.entry_count + 1,
            timestamp.to_rfc3339(),
            event_type,
            details,
            self.session_id,
            hex::encode(self.previous_hash)
        );
        
        // Calculate hash of this entry
        let current_hash = Self::calculate_hash(entry_data.as_bytes());
        
        // Create the full log entry
        let log_entry = format!(
            "{}\n  Hash: {}\n  PrevHash: {}\n",
            entry_data,
            hex::encode(current_hash),
            hex::encode(self.previous_hash)
        );
        
        // Write to file
        tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .await
            .map_err(|e| SystemError::ResourceUnavailable(format!("Failed to open audit log: {}", e)))?
            .write_all(log_entry.as_bytes())
            .await
            .map_err(|e| SystemError::ResourceUnavailable(format!("Failed to write audit log: {}", e)))?;
        
        // Update state
        self.previous_hash = current_hash;
        self.entry_count += 1;
        
        Ok(())
    }

    /// Verify the integrity of the audit log
    pub async fn verify_integrity(&self) -> Result<bool> {
        let content = tokio::fs::read_to_string(&self.log_file)
            .await
            .map_err(|e| SystemError::ResourceUnavailable(format!("Failed to read audit log: {}", e)))?;
        
        // Start with genesis hash
        let mut expected_hash = Self::calculate_hash(format!("GENESIS:{}", self.session_id).as_bytes());
        let mut _entry_count = 0u64;
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }
            
            // Skip hash lines
            if line.starts_with("  Hash:") || line.starts_with("  PrevHash:") {
                i += 1;
                continue;
            }
            
            _entry_count += 1;
            
            // Parse entry and verify hash chain
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 6 {
                let stored_prev_hash = parts[5];
                if hex::encode(expected_hash) != stored_prev_hash {
                    return Ok(false); // Hash chain broken
                }
                
                // Calculate hash for this entry
                expected_hash = Self::calculate_hash(line.as_bytes());
            }
            
            i += 1;
        }
        
        Ok(true)
    }

    /// Write the genesis entry
    fn write_genesis_entry(&mut self) -> Result<()> {
        let genesis_entry = format!(
            "0|{}|GENESIS|Session started|{}|{}\n  Hash: {}\n  PrevHash: 0000000000000000000000000000000000000000000000000000000000000000\n",
            chrono::Utc::now().to_rfc3339(),
            self.session_id,
            hex::encode(self.previous_hash),
            hex::encode(self.previous_hash)
        );
        
        std::fs::write(&self.log_file, genesis_entry)
            .map_err(|e| SystemError::ResourceUnavailable(format!("Failed to write genesis entry: {}", e)))?;
        
        Ok(())
    }

    /// Calculate SHA-256 hash
    fn calculate_hash(data: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

