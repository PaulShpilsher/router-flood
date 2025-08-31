//! Simulation orchestration with RAII resource management
//!
//! This module provides an enhanced simulation controller that uses RAII guards
//! to ensure proper resource cleanup even in error cases.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

use crate::audit::create_audit_entry;
use crate::config::Config;
use crate::error::Result;
use crate::monitor::SystemMonitor;
use crate::utils::raii::{ResourceGuard, SignalGuard, StatsGuard, TerminalRAIIGuard, WorkerGuard};
use crate::core::simulation::setup_network_interface;
use crate::stats::FloodStatsTracker;
use crate::core::target::MultiPortTarget;
use crate::core::worker::WorkerManager;

/// Enhanced simulation with RAII resource management
pub struct SimulationRAII {
    config: Config,
    target_ip: IpAddr,
    selected_interface: Option<pnet::datalink::NetworkInterface>,
    resource_guard: ResourceGuard,
}

impl SimulationRAII {
    /// Create a new RAII-managed simulation
    pub async fn new(
        config: Config,
        target_ip: IpAddr,
    ) -> Result<Self> {
        // Setup network interface
        let selected_interface = setup_network_interface(&config)?;
        
        // Create resource guard
        let resource_guard = ResourceGuard::new();
        
        Ok(Self {
            config,
            target_ip,
            selected_interface,
            resource_guard,
        })
    }
    
    /// Run the simulation with automatic resource cleanup
    pub async fn run(mut self) -> Result<()> {
        // Setup terminal guard (restored on drop)
        let terminal_guard = TerminalRAIIGuard::new()?;
        self.resource_guard = self.resource_guard.with_terminal(terminal_guard);
        
        // Setup signal handling (cleaned up on drop)
        let signal_guard = SignalGuard::new().await?;
        let running = signal_guard.running_flag();
        self.resource_guard = self.resource_guard.with_signal(signal_guard);
        
        // Create stats with guard (exported on drop)
        let stats = Arc::new(FloodStatsTracker::new(
            self.config.export.enabled.then_some(self.config.export.clone()),
        ));
        let stats_guard = StatsGuard::new(stats.clone(), "simulation");
        self.resource_guard = self.resource_guard.with_stats(stats_guard);
        
        // Setup audit logging
        self.setup_audit_logging(&stats)?;
        
        // Print simulation info
        self.print_simulation_info();
        
        // Start monitoring tasks
        self.spawn_monitoring_tasks(stats.clone(), running.clone());
        
        // Create and start workers with guard (stopped on drop)
        let multi_port_target = Arc::new(MultiPortTarget::new(self.config.target.ports.clone()));
        let worker_manager = WorkerManager::new(
            &self.config,
            stats.clone(),
            multi_port_target,
            self.target_ip,
            self.selected_interface.as_ref(),
            self.config.safety.dry_run,
        )?;
        
        let worker_guard = WorkerGuard::new(worker_manager, "simulation-workers");
        self.resource_guard = self.resource_guard.with_workers(worker_guard);
        
        // Wait for completion
        tokio::select! {
            _ = self.wait_for_shutdown() => {
                info!("🛑 Shutdown signal received, cleaning up...");
            }
            _ = self.wait_for_duration() => {
                info!("⏰ Duration reached, cleaning up...");
            }
        }
        
        // Graceful shutdown (handled by RAII guards)
        self.resource_guard.shutdown().await?;
        
        // Final stats
        self.print_final_stats(&stats).await;
        
        info!("✅ Simulation completed successfully");
        Ok(())
    }
    
    async fn wait_for_shutdown(&self) {
        while self.resource_guard.is_running() {
            time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn wait_for_duration(&self) {
        if let Some(duration_secs) = self.config.attack.duration {
            time::sleep(Duration::from_secs(duration_secs)).await;
        } else {
            std::future::pending().await
        }
    }
    
    fn setup_audit_logging(&self, stats: &Arc<FloodStatsTracker>) -> Result<()> {
        if self.config.safety.audit_logging {
            create_audit_entry(
                &self.target_ip,
                &self.config.target.ports,
                self.config.attack.threads,
                self.config.attack.packet_rate,
                self.config.attack.duration,
                self.selected_interface.as_ref().map(|i| i.name.as_str()),
                &stats.session_id,
            ).map_err(|e| crate::error::NetworkError::PacketSend(
                format!("Audit setup failed: {}", e)
            ))?;
        }
        Ok(())
    }
    
    fn print_simulation_info(&self) {
        let version = env!("CARGO_PKG_VERSION");
        
        if self.config.safety.dry_run {
            info!("🔍 Starting RAII-Enhanced Router Flood v{} (DRY-RUN)", version);
            info!("   ⚠️  DRY-RUN MODE: No actual packets will be sent!");
        } else {
            info!("🚀 Starting RAII-Enhanced Router Flood v{}", version);
        }
        
        info!("📡 Target: {} on ports {:?}", self.target_ip, self.config.target.ports);
        info!("⚙️  Threads: {}, Rate: {} pps", 
            self.config.attack.threads, 
            self.config.attack.packet_rate
        );
        
        if let Some(duration) = self.config.attack.duration {
            info!("⏱️  Duration: {} seconds", duration);
        } else {
            info!("⏱️  Duration: Unlimited (Ctrl+C to stop)");
        }
        
        if let Some(iface) = &self.selected_interface {
            info!("🔌 Interface: {} ({})", iface.name, iface.description);
        }
    }
    
    fn spawn_monitoring_tasks(&self, stats: Arc<FloodStatsTracker>, running: Arc<std::sync::atomic::AtomicBool>) {
        use std::sync::atomic::Ordering;
        
        // Spawn stats reporter
        let stats_clone = stats.clone();
        let running_clone = running.clone();
        let system_monitor = Arc::new(SystemMonitor::new(self.config.monitoring.system_monitoring));
        let interval = self.config.monitoring.stats_interval;
        
        tokio::spawn(async move {
            while running_clone.load(Ordering::Relaxed) {
                time::sleep(Duration::from_secs(interval)).await;
                let sys_stats = system_monitor.get_system_stats().await;
                stats_clone.print_stats(sys_stats.as_ref());
            }
        });
        
        // Spawn export task
        if let Some(export_interval) = self.config.monitoring.export_interval {
            if self.config.export.enabled {
                let stats_clone = stats.clone();
                let running_clone = running.clone();
                
                tokio::spawn(async move {
                    while running_clone.load(Ordering::Relaxed) {
                        time::sleep(Duration::from_secs(export_interval)).await;
                        if let Err(e) = stats_clone.export_stats(None).await {
                            error!("Failed to export stats: {}", e);
                        }
                    }
                });
            }
        }
    }
    
    async fn print_final_stats(&self, stats: &Arc<FloodStatsTracker>) {
        let system_monitor = SystemMonitor::new(self.config.monitoring.system_monitoring);
        let sys_stats = system_monitor.get_system_stats().await;
        
        info!("");
        info!("📊 Final Statistics:");
        stats.print_stats(sys_stats.as_ref());
        
        // Clear the in-place display before showing final messages
        if let Some(display) = crate::stats::get_display() {
            display.clear();
        }
        
        if self.config.export.enabled {
            if let Err(e) = stats.export_stats(sys_stats.as_ref()).await {
                error!("Failed to export final stats: {}", e);
            } else {
                info!("📁 Stats exported successfully");
            }
        }
    }
}

// Tests moved to tests/ directory
