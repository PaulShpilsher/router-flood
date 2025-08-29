//! Simulation orchestration and lifecycle management

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

use crate::audit::create_audit_entry;
use crate::config::Config;
use crate::constants::GRACEFUL_SHUTDOWN_TIMEOUT;
use crate::error::{NetworkError, Result};
use crate::monitor::SystemMonitor;
use crate::core::network::{find_interface_by_name, get_default_interface};
use crate::stats::FloodStats;
use crate::core::target::MultiPortTarget;
use crate::core::worker::WorkerManager;

/// Network interface setup
pub fn setup_network_interface(config: &Config) -> Result<Option<pnet::datalink::NetworkInterface>> {
    if let Some(iface_name) = &config.target.interface {
        match find_interface_by_name(iface_name) {
            Some(iface) => {
                info!("Using specified interface: {}", iface.name);
                Ok(Some(iface))
            }
            None => Err(NetworkError::InterfaceNotFound(iface_name.to_string()).into()),
        }
    } else {
        match get_default_interface() {
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
    stats: Arc<FloodStats>,
    system_monitor: Arc<SystemMonitor>,
    running: Arc<AtomicBool>,
    config: Config,
}

impl MonitoringTasks {
    fn new(stats: Arc<FloodStats>, config: Config, running: Arc<AtomicBool>) -> Self {
        let system_monitor = Arc::new(SystemMonitor::new(config.monitoring.system_monitoring));
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
        let interval = self.config.monitoring.stats_interval;
        
        tokio::spawn(async move {
            while running.load(Ordering::Relaxed) {
                time::sleep(Duration::from_secs(interval)).await;
                let sys_stats = system_monitor.get_system_stats().await;
                stats.print_stats(sys_stats.as_ref());
            }
        });
    }
    
    fn spawn_export_task(&self) {
        if let Some(export_interval) = self.config.monitoring.export_interval {
            if self.config.export.enabled {
                let stats = Arc::clone(&self.stats);
                let running = Arc::clone(&self.running);
                
                tokio::spawn(async move {
                    while running.load(Ordering::Relaxed) {
                        time::sleep(Duration::from_secs(export_interval)).await;
                        if let Err(e) = stats.export_stats(None).await {
                            error!("Failed to export stats: {}", e);
                        }
                    }
                });
            }
        }
    }
}

/// Main simulation controller
pub struct Simulation {
    config: Config,
    target_ip: IpAddr,
    selected_interface: Option<pnet::datalink::NetworkInterface>,
    stats: Arc<FloodStats>,
    running: Arc<AtomicBool>,
}

impl Simulation {
    pub fn new(
        config: Config,
        target_ip: IpAddr,
        selected_interface: Option<pnet::datalink::NetworkInterface>,
    ) -> Self {
        let stats = Arc::new(FloodStats::new(
            config.export.enabled.then_some(config.export.clone()),
        ));
        let running = Arc::new(AtomicBool::new(true));
        
        Self {
            config,
            target_ip,
            selected_interface,
            stats,
            running,
        }
    }
    
    pub async fn run(self) -> Result<()> {
        // Setup
        self.setup_audit_logging()?;
        self.print_simulation_info();
        
        // Start monitoring
        let monitoring = MonitoringTasks::new(Arc::clone(&self.stats), self.config.clone(), Arc::clone(&self.running));
        monitoring.spawn_all();
        
        // Create and start workers
        let multi_port_target = Arc::new(MultiPortTarget::new(self.config.target.ports.clone()));
        let worker_manager = WorkerManager::new(
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
        
        self.finalize_simulation().await?;
        Ok(())
    }
    
    fn setup_audit_logging(&self) -> Result<()> {
        if self.config.safety.audit_logging {
            create_audit_entry(
                &self.target_ip,
                &self.config.target.ports,
                self.config.attack.threads,
                self.config.attack.packet_rate,
                self.config.attack.duration,
                self.selected_interface.as_ref().map(|i| i.name.as_str()),
                &self.stats.session_id,
            ).map_err(|e| NetworkError::PacketSend(format!("Audit setup failed: {}", e)))?;
        }
        Ok(())
    }
    
    async fn wait_for_duration(&self) {
        if let Some(duration_secs) = self.config.attack.duration {
            time::sleep(Duration::from_secs(duration_secs)).await;
        } else {
            std::future::pending().await
        }
    }
    
    fn print_simulation_info(&self) {
        let version = env!("CARGO_PKG_VERSION");
        
        if self.config.safety.dry_run {
            info!("🔍 Starting Enhanced Router Flood SIMULATION v{} (DRY-RUN)", version);
            info!("   ⚠️  DRY-RUN MODE: No actual packets will be sent!");
        } else {
            info!("🚀 Starting Enhanced Router Flood Simulation v{}", version);
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
            "   Protocols: UDP({:.0}%), TCP-SYN({:.0}%), TCP-ACK({:.0}%), ICMP({:.0}%), IPv6({:.0}%), ARP({:.0}%)",
            mix.udp_ratio * 100.0,
            mix.tcp_syn_ratio * 100.0,
            mix.tcp_ack_ratio * 100.0,
            mix.icmp_ratio * 100.0,
            mix.ipv6_ratio * 100.0,
            mix.arp_ratio * 100.0
        );
        
        if self.config.safety.dry_run {
            info!("   📋 Mode: SIMULATION ONLY - Safe for testing configurations");
        }
        
        info!("   Press Ctrl+C to stop gracefully");
        println!();
    }
    
    async fn finalize_simulation(&self) -> Result<()> {
        time::sleep(GRACEFUL_SHUTDOWN_TIMEOUT).await;
        
        if self.config.safety.dry_run {
            info!("📈 Final Simulation Statistics (DRY-RUN):");
        } else {
            info!("📈 Final Statistics:");
        }
        
        self.stats.print_stats(None);
        
        if self.config.export.enabled {
            if let Err(e) = self.stats.export_stats(None).await {
                error!("Failed to export final stats: {}", e);
            }
        }
        
        if self.config.safety.dry_run {
            info!("✅ Simulation completed successfully (NO PACKETS SENT)");
            info!("📋 Dry-run mode: Configuration validated, packet generation tested");
        } else {
            info!("✅ Simulation completed successfully");
        }
        
        Ok(())
    }
}