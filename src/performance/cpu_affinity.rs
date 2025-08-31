//! CPU affinity and NUMA optimization
//!
//! This module provides CPU affinity management and NUMA-aware optimizations
//! for high-performance packet generation workloads.

use crate::error::{SystemError, Result};
use std::collections::HashMap;
use std::fs;
use std::thread;

/// CPU topology information
#[derive(Debug, Clone)]
pub struct CpuTopology {
    pub total_cpus: usize,
    pub numa_nodes: Vec<NumaNode>,
    pub cpu_to_node: HashMap<usize, usize>,
    pub hyperthreading_enabled: bool,
}

/// NUMA node information
#[derive(Debug, Clone)]
pub struct NumaNode {
    pub node_id: usize,
    pub cpus: Vec<usize>,
    pub memory_total: Option<u64>,
    pub memory_free: Option<u64>,
}

/// CPU affinity manager for performance optimization
pub struct CpuAffinity {
    topology: CpuTopology,
    worker_assignments: HashMap<usize, usize>, // worker_id -> cpu_id
}

impl CpuAffinity {
    /// Create a new CPU affinity manager
    pub fn new() -> Result<Self> {
        let topology = Self::detect_cpu_topology()?;
        Ok(Self {
            topology,
            worker_assignments: HashMap::new(),
        })
    }

    /// Get CPU topology information
    pub fn topology(&self) -> &CpuTopology {
        &self.topology
    }

    /// Assign workers to CPUs for optimal performance
    pub fn assign_workers(&mut self, num_workers: usize) -> Result<Vec<CpuAssignment>> {
        let mut assignments = Vec::new();
        
        if num_workers == 0 {
            return Ok(assignments);
        }

        // Strategy: Distribute workers across NUMA nodes and avoid hyperthreads
        let available_cpus = self.get_optimal_cpu_list();
        
        for worker_id in 0..num_workers {
            let cpu_id = available_cpus[worker_id % available_cpus.len()];
            let numa_node = self.topology.cpu_to_node.get(&cpu_id).copied().unwrap_or(0);
            
            self.worker_assignments.insert(worker_id, cpu_id);
            
            assignments.push(CpuAssignment {
                worker_id,
                cpu_id,
                numa_node,
                thread_handle: None,
            });
        }

        Ok(assignments)
    }

    /// Set CPU affinity for the current thread
    pub fn set_thread_affinity(&self, cpu_id: usize) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.set_linux_affinity(cpu_id)
        }
        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("CPU affinity not supported on this platform");
            Ok(())
        }
    }

    /// Set CPU affinity on Linux using sched_setaffinity
    #[cfg(target_os = "linux")]
    fn set_linux_affinity(&self, cpu_id: usize) -> Result<()> {
        use std::mem;
        
        if cpu_id >= self.topology.total_cpus {
            return Err(SystemError::ResourceUnavailable(
                format!("CPU {} not available (total: {})", cpu_id, self.topology.total_cpus)
            ).into());
        }

        // Create CPU set with only the specified CPU
        let mut cpu_set: libc::cpu_set_t = unsafe { mem::zeroed() };
        unsafe {
            libc::CPU_SET(cpu_id, &mut cpu_set);
        }

        // Set affinity for current thread
        let result = unsafe {
            libc::sched_setaffinity(
                0, // Current thread
                mem::size_of::<libc::cpu_set_t>(),
                &cpu_set,
            )
        };

        if result != 0 {
            return Err(SystemError::ResourceUnavailable(
                format!("Failed to set CPU affinity to CPU {}: {}", cpu_id, 
                    std::io::Error::last_os_error())
            ).into());
        }

        Ok(())
    }

    /// Get optimal CPU list avoiding hyperthreads when possible
    fn get_optimal_cpu_list(&self) -> Vec<usize> {
        let mut cpus = Vec::new();
        
        if self.topology.hyperthreading_enabled {
            // Try to use only physical cores (even numbered CPUs typically)
            for node in &self.topology.numa_nodes {
                for &cpu in &node.cpus {
                    if cpu % 2 == 0 { // Heuristic: even CPUs are often physical cores
                        cpus.push(cpu);
                    }
                }
            }
            
            // If we don't have enough physical cores, add logical cores
            if cpus.len() < 4 {
                for node in &self.topology.numa_nodes {
                    for &cpu in &node.cpus {
                        if !cpus.contains(&cpu) {
                            cpus.push(cpu);
                        }
                    }
                }
            }
        } else {
            // No hyperthreading, use all CPUs
            for node in &self.topology.numa_nodes {
                cpus.extend(&node.cpus);
            }
        }
        
        cpus.sort();
        cpus
    }

    /// Detect CPU topology from /proc and /sys
    fn detect_cpu_topology() -> Result<CpuTopology> {
        let total_cpus = Self::get_cpu_count()?;
        let numa_nodes = Self::detect_numa_nodes(total_cpus)?;
        let cpu_to_node = Self::build_cpu_to_node_map(&numa_nodes);
        let hyperthreading_enabled = Self::detect_hyperthreading()?;

        Ok(CpuTopology {
            total_cpus,
            numa_nodes,
            cpu_to_node,
            hyperthreading_enabled,
        })
    }

    /// Get total CPU count
    fn get_cpu_count() -> Result<usize> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")
            .map_err(|e| SystemError::ResourceUnavailable(format!("Failed to read /proc/cpuinfo: {}", e)))?;
        
        let cpu_count = cpuinfo.lines()
            .filter(|line| line.starts_with("processor"))
            .count();
        
        if cpu_count == 0 {
            return Err(SystemError::ResourceUnavailable("No CPUs detected".to_string()).into());
        }
        
        Ok(cpu_count)
    }

    /// Detect NUMA nodes
    fn detect_numa_nodes(total_cpus: usize) -> Result<Vec<NumaNode>> {
        let mut nodes = Vec::new();
        
        // Try to read NUMA information from /sys/devices/system/node/
        if let Ok(entries) = fs::read_dir("/sys/devices/system/node/") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if name_str.starts_with("node") {
                    if let Ok(node_id) = name_str[4..].parse::<usize>() {
                        let cpus = Self::get_node_cpus(node_id).unwrap_or_default();
                        let (memory_total, memory_free) = Self::get_node_memory(node_id);
                        
                        nodes.push(NumaNode {
                            node_id,
                            cpus,
                            memory_total,
                            memory_free,
                        });
                    }
                }
            }
        }
        
        // Fallback: create single node with all CPUs
        if nodes.is_empty() {
            nodes.push(NumaNode {
                node_id: 0,
                cpus: (0..total_cpus).collect(),
                memory_total: None,
                memory_free: None,
            });
        }
        
        Ok(nodes)
    }

    /// Get CPUs for a specific NUMA node
    fn get_node_cpus(node_id: usize) -> Option<Vec<usize>> {
        let cpulist_path = format!("/sys/devices/system/node/node{}/cpulist", node_id);
        let cpulist = fs::read_to_string(cpulist_path).ok()?;
        
        Self::parse_cpu_list(cpulist.trim())
    }

    /// Parse CPU list format (e.g., "0-3,8-11")
    fn parse_cpu_list(cpulist: &str) -> Option<Vec<usize>> {
        let mut cpus = Vec::new();
        
        for range in cpulist.split(',') {
            if let Some((start, end)) = range.split_once('-') {
                let start: usize = start.parse().ok()?;
                let end: usize = end.parse().ok()?;
                cpus.extend(start..=end);
            } else {
                let cpu: usize = range.parse().ok()?;
                cpus.push(cpu);
            }
        }
        
        Some(cpus)
    }

    /// Get memory information for a NUMA node
    fn get_node_memory(node_id: usize) -> (Option<u64>, Option<u64>) {
        let meminfo_path = format!("/sys/devices/system/node/node{}/meminfo", node_id);
        
        if let Ok(meminfo) = fs::read_to_string(meminfo_path) {
            let mut total = None;
            let mut free = None;
            
            for line in meminfo.lines() {
                if line.contains("MemTotal:") {
                    total = Self::parse_memory_line(line);
                } else if line.contains("MemFree:") {
                    free = Self::parse_memory_line(line);
                }
            }
            
            (total, free)
        } else {
            (None, None)
        }
    }

    /// Parse memory line from meminfo
    fn parse_memory_line(line: &str) -> Option<u64> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(kb) = parts[1].parse::<u64>() {
                return Some(kb * 1024); // Convert KB to bytes
            }
        }
        None
    }

    /// Build CPU to NUMA node mapping
    fn build_cpu_to_node_map(numa_nodes: &[NumaNode]) -> HashMap<usize, usize> {
        let mut map = HashMap::new();
        
        for node in numa_nodes {
            for &cpu in &node.cpus {
                map.insert(cpu, node.node_id);
            }
        }
        
        map
    }

    /// Detect if hyperthreading is enabled
    fn detect_hyperthreading() -> Result<bool> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")
            .map_err(|e| SystemError::ResourceUnavailable(format!("Failed to read /proc/cpuinfo: {}", e)))?;
        
        let mut physical_cores = std::collections::HashSet::new();
        let mut logical_cores = 0;
        
        for line in cpuinfo.lines() {
            if line.starts_with("processor") {
                logical_cores += 1;
            } else if line.starts_with("core id") {
                if let Some(core_id) = line.split(':').nth(1) {
                    if let Ok(id) = core_id.trim().parse::<usize>() {
                        physical_cores.insert(id);
                    }
                }
            }
        }
        
        Ok(logical_cores > physical_cores.len())
    }

    /// Get performance recommendations
    pub fn get_performance_recommendations(&self, num_workers: usize) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if num_workers > self.topology.total_cpus {
            recommendations.push(format!(
                "Consider reducing worker count from {} to {} (number of CPUs)",
                num_workers, self.topology.total_cpus
            ));
        }
        
        if self.topology.hyperthreading_enabled && num_workers <= self.topology.total_cpus / 2 {
            recommendations.push(
                "Hyperthreading detected. Consider using only physical cores for better performance".to_string()
            );
        }
        
        if self.topology.numa_nodes.len() > 1 {
            recommendations.push(format!(
                "NUMA system detected ({} nodes). Workers will be distributed across nodes",
                self.topology.numa_nodes.len()
            ));
        }
        
        if num_workers == 1 {
            recommendations.push(
                "Single worker detected. Consider using multiple workers for better performance".to_string()
            );
        }
        
        recommendations
    }
}

/// CPU assignment for a worker thread
#[derive(Debug)]
pub struct CpuAssignment {
    pub worker_id: usize,
    pub cpu_id: usize,
    pub numa_node: usize,
    pub thread_handle: Option<thread::JoinHandle<()>>,
}

impl Default for CpuAffinity {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback if detection fails
            Self {
                topology: CpuTopology {
                    total_cpus: num_cpus::get(),
                    numa_nodes: vec![NumaNode {
                        node_id: 0,
                        cpus: (0..num_cpus::get()).collect(),
                        memory_total: None,
                        memory_free: None,
                    }],
                    cpu_to_node: (0..num_cpus::get()).map(|i| (i, 0)).collect(),
                    hyperthreading_enabled: false,
                },
                worker_assignments: HashMap::new(),
            }
        })
    }
}

// Tests moved to tests/ directory
