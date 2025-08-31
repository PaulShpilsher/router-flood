//! Alert management system

use super::dashboard::{Alert, AlertLevel};
use super::metrics::MetricsCollector;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use chrono::Utc;
use serde::{Serialize, Deserialize};

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub level: AlertLevel,
    pub message_template: String,
    pub enabled: bool,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}

/// Alert system for handling rules and notifications
pub struct Alerts {
    rules: RwLock<HashMap<String, AlertRule>>,
    active_alerts: RwLock<HashMap<String, Alert>>,
    metrics_collector: Arc<MetricsCollector>,
    alert_history: RwLock<Vec<Alert>>,
    max_history: usize,
}

impl Alerts {
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            active_alerts: RwLock::new(HashMap::new()),
            metrics_collector,
            alert_history: RwLock::new(Vec::new()),
            max_history: 1000,
        }
    }
    
    /// Add an alert rule
    pub fn add_rule(&self, rule: AlertRule) {
        if let Ok(mut rules) = self.rules.write() {
            rules.insert(rule.name.clone(), rule);
        }
    }
    
    /// Check all rules and generate alerts
    pub fn check_alerts(&self) -> Vec<Alert> {
        let mut new_alerts = Vec::new();
        
        if let Ok(rules) = self.rules.read() {
            for rule in rules.values() {
                if !rule.enabled {
                    continue;
                }
                
                if let Some(alert) = self.check_rule(rule) {
                    new_alerts.push(alert);
                }
            }
        }
        
        // Update active alerts
        self.update_active_alerts(&new_alerts);
        
        new_alerts
    }
    
    /// Check a single rule
    fn check_rule(&self, rule: &AlertRule) -> Option<Alert> {
        let metrics = self.metrics_collector.get_all_metrics();
        
        if let Some(metric) = metrics.get(&rule.metric_name) {
            let current_value = metric.current_value();
            
            let condition_met = match rule.condition {
                AlertCondition::GreaterThan => current_value > rule.threshold,
                AlertCondition::LessThan => current_value < rule.threshold,
                AlertCondition::Equals => (current_value - rule.threshold).abs() < f64::EPSILON,
                AlertCondition::NotEquals => (current_value - rule.threshold).abs() > f64::EPSILON,
            };
            
            if condition_met {
                let message = rule.message_template
                    .replace("{metric}", &rule.metric_name)
                    .replace("{value}", &format!("{:.2}", current_value))
                    .replace("{threshold}", &format!("{:.2}", rule.threshold));
                
                return Some(Alert {
                    level: rule.level.clone(),
                    message,
                    timestamp: Utc::now().format("%H:%M:%S").to_string(),
                });
            }
        }
        
        None
    }
    
    /// Update active alerts
    fn update_active_alerts(&self, new_alerts: &[Alert]) {
        if let Ok(mut active) = self.active_alerts.write() {
            active.clear();
            
            for alert in new_alerts {
                let key = format!("{:?}_{}", alert.level, alert.message);
                active.insert(key, alert.clone());
            }
        }
        
        // Add to history
        if let Ok(mut history) = self.alert_history.write() {
            for alert in new_alerts {
                history.push(alert.clone());
            }
            
            // Trim history
            if history.len() > self.max_history {
                let excess = history.len() - self.max_history;
                history.drain(0..excess);
            }
        }
    }
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        if let Ok(active) = self.active_alerts.read() {
            active.values().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get alert history
    pub fn get_alert_history(&self, duration: Duration) -> Vec<Alert> {
        let _cutoff = Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default();
        
        if let Ok(history) = self.alert_history.read() {
            history.iter()
                .filter(|_alert| {
                    // Parse timestamp string back to DateTime for comparison
                    // For now, we'll include all alerts since string comparison is complex
                    true
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get alert statistics
    pub fn get_alert_stats(&self, duration: Duration) -> AlertStats {
        let recent_alerts = self.get_alert_history(duration);
        
        let mut stats = AlertStats {
            total_alerts: recent_alerts.len(),
            critical_alerts: 0,
            warning_alerts: 0,
            info_alerts: 0,
            most_frequent_alerts: HashMap::new(),
        };
        
        let mut alert_counts: HashMap<String, usize> = HashMap::new();
        
        for alert in recent_alerts {
            match alert.level {
                AlertLevel::Critical => stats.critical_alerts += 1,
                AlertLevel::Warning => stats.warning_alerts += 1,
                AlertLevel::Info => stats.info_alerts += 1,
            }
            
            *alert_counts.entry(alert.message).or_insert(0) += 1;
        }
        
        // Find most frequent alerts
        let mut sorted_alerts: Vec<_> = alert_counts.into_iter().collect();
        sorted_alerts.sort_by(|a, b| b.1.cmp(&a.1));
        
        for (message, count) in sorted_alerts.into_iter().take(5) {
            stats.most_frequent_alerts.insert(message, count);
        }
        
        stats
    }
}

/// Alert statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct AlertStats {
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub info_alerts: usize,
    pub most_frequent_alerts: HashMap<String, usize>,
}

/// Pre-defined alert rules for router-flood
pub fn create_default_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "high_cpu_usage".to_string(),
            metric_name: "cpu_usage_percent".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 90.0,
            level: AlertLevel::Critical,
            message_template: "High CPU usage: {value}% (threshold: {threshold}%)".to_string(),
            enabled: true,
        },
        AlertRule {
            name: "elevated_cpu_usage".to_string(),
            metric_name: "cpu_usage_percent".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 75.0,
            level: AlertLevel::Warning,
            message_template: "Elevated CPU usage: {value}% (threshold: {threshold}%)".to_string(),
            enabled: true,
        },
        AlertRule {
            name: "low_success_rate".to_string(),
            metric_name: "success_rate_percent".to_string(),
            condition: AlertCondition::LessThan,
            threshold: 95.0,
            level: AlertLevel::Warning,
            message_template: "Low packet success rate: {value}% (threshold: {threshold}%)".to_string(),
            enabled: true,
        },
        AlertRule {
            name: "slow_packet_building".to_string(),
            metric_name: "packet_build_duration_ms".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 10.0,
            level: AlertLevel::Info,
            message_template: "Slow packet building: {value}ms (threshold: {threshold}ms)".to_string(),
            enabled: true,
        },
        AlertRule {
            name: "high_memory_usage".to_string(),
            metric_name: "memory_usage_bytes".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 1024.0 * 1024.0 * 1024.0, // 1GB
            level: AlertLevel::Warning,
            message_template: "High memory usage: {value} bytes (threshold: {threshold} bytes)".to_string(),
            enabled: true,
        },
    ]
}

