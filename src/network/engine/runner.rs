//! Engine and lifecycle management for network operations

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::constants::GRACEFUL_SHUTDOWN_TIMEOUT;
use crate::error::{RouterFloodError, Result};
use crate::system_monitor::SystemMonitor;
use crate::network::{find_interface_by_name, default_interface};
use crate::stats::Stats;
use crate::network::target::PortTarget;
use crate::network::worker_manager::Workers;
use crate::security::{AuditLogger, EventType};

/// Network interface setup
pub fn setup_network_interface(config: &Config) -> Result<Option<pnet::datalink::NetworkInterface>> {
    if let Some(iface_name) = &config.target.interface {
        match find_interface_by_name(iface_name) {
            Some(iface) => {
                info!("Using specified interface: {}", iface.name);
                Ok(Some(iface))
            }
            None => Err(RouterFloodError::Network(format!("Interface not found: {}", iface_name))),
        }
    } else {
        match default_interface() {
            Some(iface) => {
                info!("Using default interface: {}", iface.name);
                Ok(Some(iface))
            }
            None => {
                warn!("No suitable network interface found");
                Ok(None)
            }
        }
    }
}

/// Monitoring tasks manager
pub(crate) struct MonitoringTasks {
    stats: Arc<Stats>,
    system_monitor: Arc<SystemMonitor>,
    running: Arc<AtomicBool>,
    config: Config,
}

impl MonitoringTasks {
    fn new(stats: Arc<Stats>, config: Config, running: Arc<AtomicBool>) -> Self {
        let system_monitor = Arc::new(SystemMonitor::new(config.export.include_system_stats));
        Self { stats, system_monitor, running, config }
    }
    
    fn spawn_all(&self) {
        self.spawn_stats_reporter();
        self.spawn_export_task();
    }
    
    fn spawn_stats_reporter(&self) {
        let stats = Arc::clone(&self.stats);
        let running = Arc::clone(&self.running);
        let system_monitor = Arc::clone(&self.system_monitor);
        let interval = self.config.monitoring.interval_ms / 1000;
        
        tokio::spawn(async move {
            // Print initial line
            println!();
            
            while running.load(Ordering::Relaxed) {
                time::sleep(Duration::from_secs(interval)).await;
                let sys_stats = system_monitor.get_system_stats().await;
                stats.print_stats_inplace(sys_stats.as_ref());
            }
        });
    }
    
    fn spawn_export_task(&self) {
        if self.config.export.enabled {
            let export_interval = self.config.export.interval_seconds;
            if self.config.export.enabled {
                let stats = Arc::clone(&self.stats);
                let running = Arc::clone(&self.running);
                
                tokio::spawn(async move {
                    while running.load(Ordering::Relaxed) {
                        time::sleep(Duration::from_secs(export_interval)).await;
                        if let Err(e) = stats.export_stats().await {
                            error!("Failed to export stats: {}", e);
                        }
                    }
                });
            }
        }
    }
}

/// Main engine that drives the network operations
pub struct Engine {
    config: Config,
    target_ip: IpAddr,
    selected_interface: Option<pnet::datalink::NetworkInterface>,
    stats: Arc<Stats>,
    running: Arc<AtomicBool>,
    audit_logger: AuditLogger,
}

impl Engine {
    pub fn new(
        config: Config,
        target_ip: IpAddr,
        selected_interface: Option<pnet::datalink::NetworkInterface>,
    ) -> Self {
        let stats = Arc::new(Stats::new(
            config.export.enabled.then_some(config.export.clone()),
        ));
        let running = Arc::new(AtomicBool::new(true));
        let audit_logger = AuditLogger::from_config(&config);
        
        Self {
            config,
            target_ip,
            selected_interface,
            stats,
            running,
            audit_logger,
        }
    }
    
    pub async fn run(self) -> Result<()> {
        // Setup
        // Log operation start
        if let Err(e) = self.audit_logger.log_event(
            EventType::SimulationStart,
            &self.target_ip,
            &self.config.target.ports,
            self.config.attack.threads,
            self.config.attack.packet_rate as u64,
            self.config.attack.duration,
            self.config.target.interface.as_deref(),
            &self.stats.session_id,
        ) {
            warn!("Failed to create audit log entry: {}", e);
        }
        
        self.print_operation_info();
        
        // Start monitoring
        let monitoring = MonitoringTasks::new(Arc::clone(&self.stats), self.config.clone(), Arc::clone(&self.running));
        monitoring.spawn_all();
        
        // Create and start workers
        let multi_port_target = Arc::new(PortTarget::new(self.config.target.ports.clone()));
        let worker_manager = Workers::new(
            &self.config,
            Arc::clone(&self.stats),
            multi_port_target,
            self.target_ip,
            self.selected_interface.as_ref(),
            self.config.safety.dry_run,
        )?;
        
        // Wait for completion
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("🛑 Received Ctrl+C, shutting down gracefully...");
                self.running.store(false, Ordering::Relaxed);
                worker_manager.stop();
            }
            _ = self.wait_for_duration() => {
                info!("⏰ Duration reached, stopping...");
                self.running.store(false, Ordering::Relaxed);
                worker_manager.stop();
            }
        }
        
        // Cleanup
        if let Err(e) = worker_manager.join_all().await {
            error!("Worker error: {}", e);
        }
        
        self.finalize_operation().await?;
        Ok(())
    }
    
    async fn wait_for_duration(&self) {
        if let Some(duration_secs) = self.config.attack.duration {
            time::sleep(Duration::from_secs(duration_secs)).await;
        } else {
            std::future::pending().await
        }
    }
    
    fn print_operation_info(&self) {
        let version = env!("CARGO_PKG_VERSION");
        
        if self.config.safety.dry_run {
            info!("🔍 Starting Router Flood Engine v{} (DRY-RUN)", version);
            info!("   ⚠️  DRY-RUN MODE: No actual packets will be sent!");
        } else {
            info!("🚀 Starting Router Flood Engine v{}", version);
        }
        
        info!("   Session ID: {}", self.stats.session_id);
        info!("   Target: {} (Ports: {:?})", self.target_ip, self.config.target.ports);
        info!("   Threads: {}, Rate: {} pps/thread", 
            self.config.attack.threads, self.config.attack.packet_rate);
        
        if let Some(d) = self.config.attack.duration {
            info!("   Duration: {} seconds", d);
        }
        
        if let Some(ref iface) = self.selected_interface {
            info!("   Interface: {}", iface.name);
        }
        
        let mix = &self.config.target.protocol_mix;
        info!(
            "   Protocols: UDP({:.0}%), TCP-SYN({:.0}%), TCP-ACK({:.0}%), ICMP({:.0}%), Custom({:.0}%)",
            mix.udp_ratio * 100.0,
            mix.tcp_syn_ratio * 100.0,
            mix.tcp_ack_ratio * 100.0,
            mix.icmp_ratio * 100.0,
            mix.custom_ratio * 100.0
        );
        
        if self.config.safety.dry_run {
            info!("   📋 Mode: SIMULATION ONLY - Safe for testing configurations");
        }
        
        info!("   Press Ctrl+C to stop gracefully");
        println!();
    }
    
    async fn finalize_operation(&self) -> Result<()> {
        time::sleep(GRACEFUL_SHUTDOWN_TIMEOUT).await;
        
        // Log operation stop
        if let Err(e) = self.audit_logger.log_event(
            EventType::SimulationStop,
            &self.target_ip,
            &self.config.target.ports,
            self.config.attack.threads,
            self.config.attack.packet_rate as u64,
            self.config.attack.duration,
            self.config.target.interface.as_deref(),
            &self.stats.session_id,
        ) {
            warn!("Failed to create audit log entry: {}", e);
        }
        
        if self.config.safety.dry_run {
            info!("📈 Final Operation Statistics (DRY-RUN):");
        } else {
            info!("📈 Final Statistics:");
        }
        
        self.stats.print_stats(None);
        
        // Clear the in-place display before showing final messages
        if let Some(display) = crate::stats::display() {
            display.clear();
        }
        
        if self.config.export.enabled
            && let Err(e) = self.stats.export_stats().await {
                error!("Failed to export final stats: {}", e);
            }
        
        if self.config.safety.dry_run {
            info!("✅ Operation completed successfully (NO PACKETS SENT)");
            info!("📋 Dry-run mode: Configuration validated, packet generation tested");
        } else {
            info!("✅ Operation completed successfully");
        }
        
        Ok(())
    }
}