//! RAII utilities for resource management
//!
//! Minimal RAII utilities - most functionality moved to main implementations.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

/// Simple guard for resource cleanup
pub struct ResourceGuard {
    name: String,
    cleanup_fn: Option<Box<dyn FnOnce() + Send>>,
}

impl ResourceGuard {
    pub fn new() -> Self {
        Self {
            name: "resource".to_string(),
            cleanup_fn: None,
        }
    }
    
    pub fn with_cleanup<F>(name: String, cleanup: F) -> Self 
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            name,
            cleanup_fn: Some(Box::new(cleanup)),
        }
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        debug!("Dropping ResourceGuard: {}", self.name);
        if let Some(cleanup) = self.cleanup_fn.take() {
            cleanup();
        }
    }
}

/// Simple signal handler guard
pub struct SignalGuard {
    running: Arc<AtomicBool>,
}

impl SignalGuard {
    pub fn new(running: Arc<AtomicBool>) -> Self {
        Self { running }
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        debug!("SignalGuard: Signaling shutdown");
    }
}

// Simplified exports for compatibility
pub struct StatsGuard;
pub struct WorkerGuard;  
pub struct TerminalRAIIGuard;

impl StatsGuard {
    pub fn new(_: Arc<crate::stats::Stats>) -> Self { Self }
}

impl WorkerGuard {
    pub fn new(_: &str) -> Self { Self }
}

impl TerminalRAIIGuard {
    pub fn new() -> Self { Self }
}