//! RAII (Resource Acquisition Is Initialization) guards for resource management
//!
//! This module provides RAII guards that ensure proper cleanup of resources
//! when they go out of scope, following Rust's ownership principles.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, error};

use crate::error::{RouterFloodError, Result};
use crate::stats::Stats;
use crate::utils::terminal::Terminal;
use crate::transport::WorkerChannels;
use crate::network::worker_manager::Workers;

/// RAII guard for worker thread management
/// 
/// Ensures workers are stopped gracefully when the guard is dropped
pub struct WorkerGuard {
    manager: Option<Workers>,
    name: String,
}

impl WorkerGuard {
    /// Create a new worker guard
    pub fn new(manager: Workers, name: &str) -> Self {
        let name = name.to_string();
        debug!("WorkerGuard created for: {}", name);
        Self {
            manager: Some(manager),
            name,
        }
    }
    
    /// Take ownership of the manager (for explicit shutdown)
    pub fn take(&mut self) -> Option<Workers> {
        self.manager.take()
    }
    
    /// Check if workers are still running
    pub fn is_running(&self) -> bool {
        self.manager.as_ref().is_some_and(|m| m.is_running())
    }
    
    /// Stop workers gracefully
    pub fn stop(&self) {
        if let Some(manager) = &self.manager {
            debug!("Stopping workers for: {}", self.name);
            manager.stop();
        }
    }
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        if let Some(manager) = &self.manager {
            debug!("WorkerGuard dropping for: {}", self.name);
            manager.stop();
        }
    }
}

/// RAII guard for network channels
///
/// Ensures channels are properly closed when dropped
pub struct ChannelGuard {
    channels: Option<WorkerChannels>,
    name: String,
}

impl ChannelGuard {
    /// Create a new channel guard
    pub fn new(channels: WorkerChannels, name: &str) -> Self {
        let name = name.to_string();
        debug!("ChannelGuard created for: {}", name);
        Self {
            channels: Some(channels),
            name,
        }
    }
    
    /// Get mutable reference to channels
    pub fn channels_mut(&mut self) -> Option<&mut WorkerChannels> {
        self.channels.as_mut()
    }
    
    /// Take ownership of channels
    pub fn take(&mut self) -> Option<WorkerChannels> {
        self.channels.take()
    }
}

impl Drop for ChannelGuard {
    fn drop(&mut self) {
        debug!("ChannelGuard dropping for: {}", self.name);
        self.channels = None;
    }
}

/// RAII guard for signal handling
///
/// Sets up signal handlers and ensures they are cleaned up
pub struct SignalGuard {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl SignalGuard {
    /// Create a new signal guard and set up handlers
    pub async fn new() -> Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        
        let handle = tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    debug!("SIGINT received, setting shutdown flag");
                    running_clone.store(false, Ordering::SeqCst);
                }
                Err(e) => {
                    error!("Failed to listen for SIGINT: {}", e);
                }
            }
        });
        
        debug!("SignalGuard created with SIGINT handler");
        
        Ok(Self {
            running,
            handle: Some(handle),
        })
    }
    
    /// Check if shutdown was requested
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// Get a clone of the running flag
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        debug!("SignalGuard dropping");
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

/// RAII guard for stats flushing
///
/// Ensures stats are exported when the guard is dropped
pub struct StatsGuard {
    stats: Arc<Stats>,
    name: String,
}

impl StatsGuard {
    /// Create a new stats guard
    pub fn new(stats: Arc<Stats>, name: &str) -> Self {
        let name = name.to_string();
        debug!("StatsGuard created for: {}", name);
        Self { stats, name }
    }
    
    /// Get reference to stats
    pub fn stats(&self) -> &Arc<Stats> {
        &self.stats
    }
    
    /// Export stats explicitly
    pub async fn export(&self) -> Result<()> {
        debug!("Exporting stats for: {}", self.name);
        self.stats.export_stats(None).await
    }
}

impl Drop for StatsGuard {
    fn drop(&mut self) {
        debug!("StatsGuard dropping for: {}", self.name);
        let stats = self.stats.clone();
        let name = self.name.clone();
        
        tokio::spawn(async move {
            if let Err(e) = stats.export_stats(None).await {
                error!("Failed to export stats on drop for {}: {}", name, e);
            }
        });
    }
}

/// RAII guard for terminal settings
///
/// Ensures terminal is restored to original state
pub struct TerminalRAIIGuard {
    controller: Option<Terminal>,
}

impl TerminalRAIIGuard {
    /// Create a new terminal guard
    pub fn new() -> Result<Self> {
        let mut controller = Terminal::new();
        
        if Terminal::is_tty() {
            controller.disable_ctrl_echo()
                .map_err(|e| RouterFloodError::Network(format!("Terminal setup failed: {}", e)))?
        }
        
        debug!("TerminalRAIIGuard created");
        
        Ok(Self {
            controller: Some(controller),
        })
    }
}

impl Drop for TerminalRAIIGuard {
    fn drop(&mut self) {
        if let Some(mut controller) = self.controller.take() {
            debug!("TerminalRAIIGuard dropping, restoring terminal");
            if let Err(e) = controller.restore() {
                error!("Failed to restore terminal: {}", e);
            }
        }
    }
}

/// Composite RAII guard that manages multiple resources
///
/// Ensures all resources are cleaned up in the correct order
pub struct ResourceGuard {
    terminal: Option<TerminalRAIIGuard>,
    signal: Option<SignalGuard>,
    workers: Option<WorkerGuard>,
    stats: Option<StatsGuard>,
}

impl Default for ResourceGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceGuard {
    /// Create a new resource guard
    pub fn new() -> Self {
        debug!("ResourceGuard created");
        Self {
            terminal: None,
            signal: None,
            workers: None,
            stats: None,
        }
    }
    
    /// Set terminal guard
    pub fn with_terminal(mut self, guard: TerminalRAIIGuard) -> Self {
        self.terminal = Some(guard);
        self
    }
    
    /// Set signal guard
    pub fn with_signal(mut self, guard: SignalGuard) -> Self {
        self.signal = Some(guard);
        self
    }
    
    /// Set worker guard
    pub fn with_workers(mut self, guard: WorkerGuard) -> Self {
        self.workers = Some(guard);
        self
    }
    
    /// Set stats guard
    pub fn with_stats(mut self, guard: StatsGuard) -> Self {
        self.stats = Some(guard);
        self
    }
    
    /// Check if shutdown was requested
    pub fn is_running(&self) -> bool {
        self.signal.as_ref().is_none_or(|s| s.is_running())
    }
    
    /// Stop all managed resources
    pub async fn shutdown(&mut self) -> Result<()> {
        debug!("ResourceGuard initiating shutdown");
        
        // Stop workers first
        if let Some(workers) = &self.workers {
            workers.stop();
        }
        
        // Export stats
        if let Some(stats) = &self.stats {
            stats.export().await?;
        }
        
        // Clear all guards in reverse order
        self.workers = None;
        self.stats = None;
        self.signal = None;
        self.terminal = None;
        
        debug!("ResourceGuard shutdown complete");
        Ok(())
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        debug!("ResourceGuard dropping");
        
        // Resources are dropped in reverse order of declaration
        // This ensures workers stop before stats are exported,
        // and terminal is restored last
        self.workers = None;
        self.stats = None;
        self.signal = None;
        self.terminal = None;
    }
}

// Tests moved to tests/ directory
