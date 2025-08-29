//! Observer pattern for statistics collection
//!
//! This module implements the Observer pattern for extensible statistics
//! collection and reporting, allowing multiple observers to react to events.

use super::collector::{SessionStats, SystemStats};
use std::sync::{Arc, RwLock, Weak};

/// Events that can be observed in the statistics system
#[derive(Debug, Clone)]
pub enum StatsEvent {
    /// Packet successfully sent
    PacketSent { bytes: u64, protocol: String },
    /// Packet failed to send
    PacketFailed { protocol: String },
    /// Statistics interval reached
    IntervalReached { stats: SessionStats },
    /// Session started
    SessionStarted { session_id: String },
    /// Session ended
    SessionEnded { session_id: String, final_stats: SessionStats },
    /// System stats updated
    SystemStatsUpdated { stats: SystemStats },
}

/// Trait for statistics observers
pub trait StatsObserver: Send + Sync {
    /// Handle a statistics event
    fn on_event(&self, event: &StatsEvent);
    
    /// Called when observer is attached
    fn on_attach(&self) {}
    
    /// Called when observer is detached
    fn on_detach(&self) {}
}

/// Subject that manages statistics observers
pub struct StatsSubject {
    observers: RwLock<Vec<Weak<dyn StatsObserver>>>,
}

impl StatsSubject {
    /// Create a new stats subject
    pub fn new() -> Self {
        Self {
            observers: RwLock::new(Vec::new()),
        }
    }
    
    /// Attach an observer
    pub fn attach(&self, observer: Arc<dyn StatsObserver>) {
        observer.on_attach();
        let mut observers = self.observers.write().unwrap();
        observers.push(Arc::downgrade(&observer));
        
        // Clean up dead weak references
        observers.retain(|obs| obs.strong_count() > 0);
    }
    
    /// Detach an observer
    pub fn detach(&self, observer: &Arc<dyn StatsObserver>) {
        observer.on_detach();
        let mut observers = self.observers.write().unwrap();
        observers.retain(|obs| {
            if let Some(strong) = obs.upgrade() {
                !Arc::ptr_eq(&strong, observer)
            } else {
                false
            }
        });
    }
    
    /// Notify all observers of an event
    pub fn notify(&self, event: &StatsEvent) {
        let observers = self.observers.read().unwrap();
        for observer_weak in observers.iter() {
            if let Some(observer) = observer_weak.upgrade() {
                observer.on_event(event);
            }
        }
    }
    
    /// Get the number of active observers
    pub fn observer_count(&self) -> usize {
        let observers = self.observers.read().unwrap();
        observers.iter().filter(|obs| obs.strong_count() > 0).count()
    }
}

impl Default for StatsSubject {
    fn default() -> Self {
        Self::new()
    }
}

/// Console observer that prints statistics to stdout
pub struct ConsoleObserver {
    verbose: bool,
}

impl ConsoleObserver {
    /// Create a new console observer
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl StatsObserver for ConsoleObserver {
    fn on_event(&self, event: &StatsEvent) {
        match event {
            StatsEvent::IntervalReached { stats } => {
                println!("\n[STATS] Session: {}", stats.session_id);
                println!("  Packets sent: {}", stats.packets_sent);
                println!("  Packets failed: {}", stats.packets_failed);
                println!("  Rate: {:.2} pps", stats.packets_per_second);
                println!("  Throughput: {:.2} Mbps", stats.megabits_per_second);
            }
            StatsEvent::SessionEnded { session_id, final_stats } => {
                println!("\n[SESSION END] {}", session_id);
                println!("  Total packets: {}", final_stats.packets_sent);
                println!("  Total bytes: {}", final_stats.bytes_sent);
                println!("  Duration: {:.2}s", final_stats.duration_secs);
            }
            _ => {
                if self.verbose {
                    println!("[EVENT] {:?}", event);
                }
            }
        }
    }
}

/// File observer that logs statistics to a file
pub struct FileObserver {
    file_path: String,
}

impl FileObserver {
    /// Create a new file observer
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

impl StatsObserver for FileObserver {
    fn on_event(&self, event: &StatsEvent) {
        // Simplified implementation - in production would use async I/O
        if let StatsEvent::IntervalReached { stats } = event {
            if let Ok(json) = serde_json::to_string_pretty(stats) {
                let _ = std::fs::write(&self.file_path, json);
            }
        }
    }
}

/// Metrics observer for integration with monitoring systems
pub struct MetricsObserver {
    metrics: Arc<RwLock<MetricsData>>,
}

#[derive(Default)]
struct MetricsData {
    total_packets: u64,
    total_bytes: u64,
    total_failures: u64,
    current_rate: f64,
}

impl MetricsObserver {
    /// Create a new metrics observer
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(MetricsData::default())),
        }
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> (u64, u64, u64, f64) {
        let metrics = self.metrics.read().unwrap();
        (metrics.total_packets, metrics.total_bytes, metrics.total_failures, metrics.current_rate)
    }
}

impl StatsObserver for MetricsObserver {
    fn on_event(&self, event: &StatsEvent) {
        let mut metrics = self.metrics.write().unwrap();
        match event {
            StatsEvent::PacketSent { bytes, .. } => {
                metrics.total_packets += 1;
                metrics.total_bytes += bytes;
            }
            StatsEvent::PacketFailed { .. } => {
                metrics.total_failures += 1;
            }
            StatsEvent::IntervalReached { stats } => {
                metrics.current_rate = stats.packets_per_second;
            }
            _ => {}
        }
    }
}

/// Builder for creating composite observers
pub struct ObserverBuilder {
    observers: Vec<Arc<dyn StatsObserver>>,
}

impl ObserverBuilder {
    /// Create a new observer builder
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }
    
    /// Add an observer
    pub fn add(mut self, observer: Arc<dyn StatsObserver>) -> Self {
        self.observers.push(observer);
        self
    }
    
    /// Add console output
    pub fn with_console(self, verbose: bool) -> Self {
        self.add(Arc::new(ConsoleObserver::new(verbose)))
    }
    
    /// Add file logging
    pub fn with_file(self, path: impl Into<String>) -> Self {
        self.add(Arc::new(FileObserver::new(path)))
    }
    
    /// Add metrics collection
    pub fn with_metrics(self) -> Self {
        self.add(Arc::new(MetricsObserver::new()))
    }
    
    /// Build a composite observer
    pub fn build(self) -> Arc<CompositeObserver> {
        Arc::new(CompositeObserver {
            observers: self.observers,
        })
    }
}

/// Composite observer that delegates to multiple observers
pub struct CompositeObserver {
    observers: Vec<Arc<dyn StatsObserver>>,
}

impl StatsObserver for CompositeObserver {
    fn on_event(&self, event: &StatsEvent) {
        for observer in &self.observers {
            observer.on_event(event);
        }
    }
    
    fn on_attach(&self) {
        for observer in &self.observers {
            observer.on_attach();
        }
    }
    
    fn on_detach(&self) {
        for observer in &self.observers {
            observer.on_detach();
        }
    }
}