//! Worker thread management with CPU affinity support
//!
//! Manages worker threads for optimized packet generation with batch processing
//! and optional CPU affinity for improved performance.

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::config::Config;
use crate::error::{RouterFloodError, Result};
use crate::stats::Stats;
use crate::network::target::MultiPortTarget;
use crate::network::worker::{Worker, WorkerConfig};
use crate::performance::cpu_affinity::CpuAffinity;

/// Manages the lifecycle of worker threads with optional CPU affinity
pub struct Workers {
    handles: Vec<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    cpu_affinity: Option<Arc<CpuAffinity>>,
}

impl Workers {
    /// Create a new worker manager and spawn worker threads
    pub fn new(
        config: &Config,
        stats: Arc<Stats>,
        multi_port_target: Arc<MultiPortTarget>,
        target_ip: IpAddr,
        interface: Option<&pnet::datalink::NetworkInterface>,
        dry_run: bool,
    ) -> Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        
        // Initialize CPU affinity if not in dry-run mode
        let cpu_affinity = if !dry_run && config.attack.threads > 1 {
            match CpuAffinity::new() {
                Ok(affinity) => {
                    info!("CPU affinity initialized: {} CPUs available", 
                          affinity.topology().total_cpus);
                    Some(Arc::new(affinity))
                }
                Err(e) => {
                    warn!("Failed to initialize CPU affinity: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        let handles = Self::spawn_workers(
            config,
            stats,
            running.clone(),
            multi_port_target,
            target_ip,
            interface,
            dry_run,
            cpu_affinity.clone(),
        )?;

        Ok(Self { handles, running, cpu_affinity })
    }

    /// Spawn worker threads based on configuration with optional CPU pinning
    fn spawn_workers(
        config: &Config,
        stats: Arc<Stats>,
        running: Arc<AtomicBool>,
        multi_port_target: Arc<MultiPortTarget>,
        target_ip: IpAddr,
        _interface: Option<&pnet::datalink::NetworkInterface>,
        _dry_run: bool,
        cpu_affinity: Option<Arc<CpuAffinity>>,
    ) -> Result<Vec<JoinHandle<()>>> {
        let mut handles = Vec::with_capacity(config.attack.threads);
        
        let per_worker_rate = (config.attack.packet_rate / config.attack.threads as f64) as u64;
        
        for task_id in 0..config.attack.threads {
            let running = running.clone();
            let stats = stats.clone();
            let target = multi_port_target.clone();
            let packet_size_range = (config.attack.payload_size, config.attack.payload_size);
            let protocol_mix = config.target.protocol_mix.clone();
            let randomize_timing = false;  // Simplified for now
            let dry_run = config.safety.dry_run;
            
            let worker_config = WorkerConfig {
                packet_rate: per_worker_rate,
                packet_size_range,
                protocol_mix,
                randomize_timing,
                dry_run,
            };
            
            let mut worker = Worker::new(
                stats,
                target_ip,
                target,
                worker_config,
            );
            
            let affinity = cpu_affinity.clone();
            let worker_id = task_id;
            
            let handle = tokio::spawn(async move {
                // Set CPU affinity for this worker if available
                if let Some(ref affinity) = affinity {
                    if let Err(e) = affinity.set_thread_affinity(worker_id) {
                        warn!("Failed to pin worker {} to CPU: {}", worker_id, e);
                    } else {
                        info!("Worker {} pinned to CPU {}", worker_id, worker_id);
                    }
                }
                
                worker.run(running).await;
            });

            handles.push(handle);
        }

        Ok(handles)
    }

    /// Stop all worker threads gracefully
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Wait for all worker threads to complete
    pub async fn join_all(self) -> Result<()> {
        for handle in self.handles {
            handle.await.map_err(|e| RouterFloodError::Network(format!("Worker join error: {}", e)))?;
        }
        Ok(())
    }

    /// Check if workers are still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// Get CPU affinity information if available
    pub fn cpu_affinity(&self) -> Option<&CpuAffinity> {
        self.cpu_affinity.as_deref()
    }
}