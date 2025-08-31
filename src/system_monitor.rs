use crate::stats::SystemStats;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;

/// System monitoring for performance tracking
pub struct SystemMonitor {
    system: Arc<Mutex<System>>,
    monitoring_enabled: bool,
}

impl SystemMonitor {
    pub fn new(monitoring_enabled: bool) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: Arc::new(Mutex::new(system)),
            monitoring_enabled,
        }
    }

    pub async fn get_system_stats(&self) -> Option<SystemStats> {
        if !self.monitoring_enabled {
            return None;
        }

        let mut system = self.system.lock().await;
        system.refresh_all();

        Some(SystemStats {
            cpu_usage: system.global_cpu_usage(),
            memory_usage: system.used_memory(),
            memory_total: system.total_memory(),
            network_sent: 0, // Would need more complex implementation
            network_received: 0,
        })
    }
}
