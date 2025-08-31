//! Monitoring & Security Integration
//!
//! This module integrates monitoring and security enhancements:
//! 1. Lightweight dashboard with essential metrics
//! 2. Security hardening with threat detection and input validation
//!
//! This system focuses on monitoring and security while maintaining simplicity
//! and following YAGNI principles.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{info, warn, error};

use crate::monitoring::{
    Dashboard, DashboardConfig, DashboardBuilder, AlertThresholds,
    EssentialMetricsCollector
};
use crate::security::{
    ThreatDetector, ThreatDetectionConfig, InputValidator,
    ValidationConfig, Capabilities
};
use crate::error::Result;

/// Security-focused application runner with monitoring
pub struct SecurityRunner {
    dashboard: Option<Dashboard>,
    threat_detector: ThreatDetector,
    input_validator: InputValidator,
    capability_manager: Capabilities,
    metrics_collector: Arc<EssentialMetricsCollector>,
    config: MonitoringSecurityConfig,
}

/// Monitoring & security configuration
#[derive(Debug, Clone)]
pub struct MonitoringSecurityConfig {
    pub enable_dashboard: bool,
    pub enable_threat_detection: bool,
    pub enable_input_validation: bool,
    pub dashboard_config: DashboardConfig,
    pub threat_config: ThreatDetectionConfig,
    pub validation_config: ValidationConfig,
}

/// Security context for monitoring & security
#[derive(Debug)]
pub struct SecurityContext {
    pub threats_detected: u64,
    pub validation_failures: u64,
    pub security_level: SecurityLevel,
    pub last_threat_time: Option<String>,
}

/// Security level assessment
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    Safe,
    Elevated,
    High,
    Critical,
}

/// Monitoring summary for monitoring & security
#[derive(Debug)]
pub struct MonitoringSummary {
    pub dashboard_active: bool,
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub uptime: Duration,
    pub performance_score: f64,
}

impl Default for MonitoringSecurityConfig {
    fn default() -> Self {
        Self {
            enable_dashboard: true,
            enable_threat_detection: true,
            enable_input_validation: true,
            dashboard_config: DashboardBuilder::new()
                .update_interval(Duration::from_secs(1))
                .compact_mode(false)
                .show_progress_bar(true)
                .alert_thresholds(AlertThresholds {
                    max_failure_rate: 5.0,
                    min_success_rate: 95.0,
                    max_response_time: 50.0,
                    min_throughput: 10.0,
                })
                .build(),
            threat_config: ThreatDetectionConfig {
                enable_rate_limiting: true,
                enable_input_validation: true,
                enable_anomaly_detection: true,
                max_requests_per_minute: 60,
                max_packet_size: 65535,
                max_target_ports: 100,
                suspicious_pattern_threshold: 5,
            },
            validation_config: ValidationConfig {
                max_string_length: 1024,
                max_array_size: 100,
                allow_special_chars: false,
                strict_ip_validation: true,
                enable_pattern_detection: true,
            },
        }
    }
}

impl SecurityRunner {
    /// Create a new monitoring & security runner
    pub fn new(config: MonitoringSecurityConfig) -> Result<Self> {
        let metrics_collector = Arc::new(EssentialMetricsCollector::new());
        
        // Initialize threat detector
        let threat_detector = ThreatDetector::new(config.threat_config.clone());
        
        // Initialize input validator
        let input_validator = InputValidator::new(config.validation_config.clone());
        
        // Initialize capability manager
        let capability_manager = Capabilities::new()?;
        
        // Initialize dashboard if enabled
        let dashboard = if config.enable_dashboard {
            Some(Dashboard::new(
                Arc::clone(&metrics_collector),
                config.dashboard_config.clone(),
            ))
        } else {
            None
        };
        
        Ok(Self {
            dashboard,
            threat_detector,
            input_validator,
            capability_manager,
            metrics_collector,
            config,
        })
    }

    /// Start monitoring and security
    pub async fn start(&self, running: Arc<AtomicBool>) -> Result<()> {
        info!("🚀 Starting Monitoring & Security");
        
        // Display security context
        self.display_security_status();
        
        // Start dashboard if enabled
        if let Some(ref dashboard) = self.dashboard {
            info!("📊 Starting dashboard");
            let dashboard_running = Arc::clone(&running);
            
            // Note: Dashboard doesn't implement Clone, so we'll start it directly
            dashboard.start(dashboard_running).await;
        }
        
        // Start security monitoring
        if self.config.enable_threat_detection {
            info!("🛡️ Starting threat detection");
            self.start_security_monitoring(Arc::clone(&running)).await;
        }
        
        Ok(())
    }

    /// Validate configuration with security enhancements
    pub fn validate_configuration(&self, config_data: &str) -> Result<()> {
        if !self.config.enable_input_validation {
            return Ok(());
        }
        
        info!("🔍 Validating configuration with security enhancements");
        
        // Threat detection validation
        let validation_result = self.threat_detector.validate_configuration(config_data)?;
        
        if !validation_result.is_valid {
            error!("❌ Configuration validation failed due to security threats");
            for threat in &validation_result.threats {
                warn!("🚨 Threat detected: {} - {}", 
                    threat.threat_type_str(), threat.description);
            }
            return Err(crate::error::ValidationError::SystemRequirement(
                "Configuration contains security threats"
            ).into());
        }
        
        if !validation_result.threats.is_empty() {
            warn!("⚠️ {} security warnings detected in configuration", 
                validation_result.threats.len());
        }
        
        info!("✅ Configuration validation passed");
        Ok(())
    }

    /// Validate target IP with security enhancements
    pub fn validate_target_ip(&self, ip_str: &str) -> Result<()> {
        if !self.config.enable_input_validation {
            return Ok(());
        }
        
        // Input validation
        let validation_result = self.input_validator.validate_ip_address(ip_str)?;
        
        if !validation_result.warnings.is_empty() {
            for warning in &validation_result.warnings {
                warn!("⚠️ IP validation warning: {}", warning);
            }
        }
        
        // Threat detection validation
        self.threat_detector.validate_target_ip(&validation_result.value.addr)?;
        
        info!("✅ Target IP {} validated successfully", ip_str);
        Ok(())
    }

    /// Validate ports with security enhancements
    pub fn validate_ports(&self, ports: &[u16]) -> Result<()> {
        if !self.config.enable_input_validation {
            return Ok(());
        }
        
        // Input validation
        let validation_result = self.input_validator.validate_port_list(ports)?;
        
        if !validation_result.warnings.is_empty() {
            for warning in &validation_result.warnings {
                warn!("⚠️ Port validation warning: {}", warning);
            }
        }
        
        // Threat detection validation
        self.threat_detector.validate_ports(ports)?;
        
        info!("✅ {} ports validated successfully", ports.len());
        Ok(())
    }

    /// Get metrics collector for integration
    pub fn metrics_collector(&self) -> Arc<EssentialMetricsCollector> {
        Arc::clone(&self.metrics_collector)
    }

    /// Get security context
    pub fn get_security_context(&self) -> SecurityContext {
        let threat_summary = self.threat_detector.get_threat_summary();
        
        let security_level = match threat_summary.total_threats {
            0 => SecurityLevel::Safe,
            1..=5 => SecurityLevel::Elevated,
            6..=20 => SecurityLevel::High,
            _ => SecurityLevel::Critical,
        };
        
        SecurityContext {
            threats_detected: threat_summary.total_threats as u64,
            validation_failures: 0, // Would be tracked in a real implementation
            security_level,
            last_threat_time: threat_summary.last_threat
                .map(|t| t.timestamp),
        }
    }

    /// Get monitoring summary
    pub fn get_monitoring_summary(&self) -> MonitoringSummary {
        let metrics = self.metrics_collector.get_metrics();
        
        // Calculate performance score based on success rate and throughput
        let performance_score = (metrics.success_rate / 100.0) * 
            (metrics.packets_per_second / 100.0).min(1.0);
        
        MonitoringSummary {
            dashboard_active: self.dashboard.is_some(),
            total_alerts: 0, // Would be tracked from dashboard
            critical_alerts: 0,
            uptime: Duration::from_secs_f64(metrics.duration_secs),
            performance_score,
        }
    }

    /// Display security status
    fn display_security_status(&self) {
        println!("🔒 Monitoring & Security Status");
        println!("═══════════════════════════════════");
        
        // Capability status
        let security_context = self.capability_manager.security_context();
        println!("Process ID: {}", security_context.process_id);
        println!("Effective UID: {}", security_context.effective_uid);
        println!("CAP_NET_RAW: {}", 
            if security_context.has_net_raw { "✅" } else { "❌" });
        
        // Feature status
        println!("\\n🛡️ Security Features:");
        println!("Dashboard: {}", 
            if self.config.enable_dashboard { "✅ Enabled" } else { "❌ Disabled" });
        println!("Threat Detection: {}", 
            if self.config.enable_threat_detection { "✅ Enabled" } else { "❌ Disabled" });
        println!("Input Validation: {}", 
            if self.config.enable_input_validation { "✅ Enabled" } else { "❌ Disabled" });
        
        println!();
    }

    /// Start security monitoring background task
    async fn start_security_monitoring(&self, running: Arc<AtomicBool>) {
        let mut interval = time::interval(Duration::from_secs(30));
        
        while running.load(Ordering::Relaxed) {
            interval.tick().await;
            
            // Check for anomalies
            let metrics = self.metrics_collector.get_metrics();
            let anomalies = self.threat_detector.check_anomalies(
                metrics.packets_per_second,
                metrics.bytes_sent as f64 / metrics.packets_sent.max(1) as f64,
            );
            
            if !anomalies.is_empty() {
                warn!("🚨 {} anomalies detected", anomalies.len());
                for anomaly in anomalies {
                    warn!("   - {}", anomaly.description);
                }
            }
            
            // Log security status periodically
            let security_context = self.get_security_context();
            if security_context.threats_detected > 0 {
                info!("🛡️ Security status: {} threats detected, level: {:?}", 
                    security_context.threats_detected, security_context.security_level);
            }
        }
    }

    /// Export security and monitoring report
    pub async fn export_report(&self, filename: &str) -> Result<()> {
        let security_context = self.get_security_context();
        let monitoring_summary = self.get_monitoring_summary();
        let threat_summary = self.threat_detector.get_threat_summary();
        
        let report = serde_json::json!({
            "monitoring_security_report": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "security_context": {
                    "threats_detected": security_context.threats_detected,
                    "security_level": format!("{:?}", security_context.security_level),
                    "last_threat_time": security_context.last_threat_time
                },
                "monitoring_summary": {
                    "dashboard_active": monitoring_summary.dashboard_active,
                    "uptime_seconds": monitoring_summary.uptime.as_secs(),
                    "performance_score": monitoring_summary.performance_score
                },
                "threat_summary": threat_summary,
                "metrics": self.metrics_collector.get_metrics()
            }
        });
        
        let json = serde_json::to_string_pretty(&report)
            .map_err(|e| crate::error::StatsError::SerializationError(e.to_string()))?;
        
        tokio::fs::write(filename, json).await
            .map_err(|e| crate::error::StatsError::FileWriteError(e.to_string()))?;
        
        info!("📄 Monitoring & security report exported to {}", filename);
        Ok(())
    }
}

/// Initialize security runner with default configuration
pub fn init_security_runner() -> Result<SecurityRunner> {
    let config = MonitoringSecurityConfig::default();
    SecurityRunner::new(config)
}

/// Initialize security runner with custom configuration
pub fn init_security_runner_with_config(config: MonitoringSecurityConfig) -> Result<SecurityRunner> {
    SecurityRunner::new(config)
}

/// Create a minimal monitoring & security configuration for testing
pub fn create_minimal_config() -> MonitoringSecurityConfig {
    MonitoringSecurityConfig {
        enable_dashboard: false,
        enable_threat_detection: true,
        enable_input_validation: true,
        dashboard_config: DashboardBuilder::new()
            .compact_mode(true)
            .build(),
        threat_config: ThreatDetectionConfig {
            enable_rate_limiting: false,
            enable_input_validation: true,
            enable_anomaly_detection: false,
            max_requests_per_minute: 1000,
            max_packet_size: 65535,
            max_target_ports: 1000,
            suspicious_pattern_threshold: 100,
        },
        validation_config: ValidationConfig::default(),
    }
}

/// Create a high-security monitoring & security configuration
pub fn create_high_security_config() -> MonitoringSecurityConfig {
    MonitoringSecurityConfig {
        enable_dashboard: true,
        enable_threat_detection: true,
        enable_input_validation: true,
        dashboard_config: DashboardBuilder::new()
            .update_interval(Duration::from_millis(500))
            .alert_thresholds(AlertThresholds {
                max_failure_rate: 1.0,
                min_success_rate: 99.0,
                max_response_time: 10.0,
                min_throughput: 50.0,
            })
            .build(),
        threat_config: ThreatDetectionConfig {
            enable_rate_limiting: true,
            enable_input_validation: true,
            enable_anomaly_detection: true,
            max_requests_per_minute: 30,
            max_packet_size: 1500,
            max_target_ports: 10,
            suspicious_pattern_threshold: 1,
        },
        validation_config: ValidationConfig {
            max_string_length: 256,
            max_array_size: 10,
            allow_special_chars: false,
            strict_ip_validation: true,
            enable_pattern_detection: true,
        },
    }
}