//! CPU affinity and NUMA optimization
//!
//! This module provides CPU affinity management and NUMA-aware optimizations
//! for high-performance packet generation workloads.
//!
//! ## Key Concepts:
//!
//! **CPU Affinity**: Binding a thread to specific CPU cores to reduce context switching
//! and improve cache locality. This prevents the OS scheduler from moving threads
//! between cores, which can cause cache misses and performance degradation.
//!
//! **NUMA (Non-Uniform Memory Access)**: Modern multi-socket systems where each CPU
//! socket has its own local memory. Accessing local memory is faster than remote memory.
//! Proper NUMA awareness can significantly improve performance.
//!
//! **Hyperthreading**: Intel's SMT technology where each physical core appears as
//! two logical cores. For CPU-intensive workloads, using only physical cores
//! often provides better performance.

use crate::error::{SystemError, Result};
use std::collections::HashMap;
use std::fs;
use std::thread;

// Note on libc usage:
// We use libc for direct system call access to CPU affinity functions.
// These are thin wrappers around Linux kernel syscalls:
// - sched_setaffinity: Binds threads to specific CPUs
// - CPU_SET/CPU_ZERO: Macros for manipulating CPU bitmasks
// The 'unsafe' blocks are required because we're directly calling C functions
// that can potentially violate Rust's memory safety guarantees.

/// CPU topology information
///
/// Represents the physical layout of CPUs in the system, including
/// NUMA topology and hyperthreading configuration.
#[derive(Debug, Clone)]
pub struct CpuTopology {
    /// Total number of logical CPUs (includes hyperthreads)
    pub total_cpus: usize,
    /// List of NUMA nodes in the system
    pub numa_nodes: Vec<NumaNode>,
    /// Maps CPU ID to its NUMA node for quick lookups
    pub cpu_to_node: HashMap<usize, usize>,
    /// Whether SMT/Hyperthreading is enabled (logical cores > physical cores)
    pub hyperthreading_enabled: bool,
}

/// NUMA node information
///
/// Represents a NUMA domain - a group of CPUs with shared local memory.
/// Memory access within a NUMA node is faster than cross-node access.
#[derive(Debug, Clone)]
pub struct NumaNode {
    /// NUMA node identifier (typically 0, 1, 2, ...)
    pub node_id: usize,
    /// List of CPU IDs belonging to this NUMA node
    pub cpus: Vec<usize>,
    /// Total memory available on this node (in bytes)
    pub memory_total: Option<u64>,
    /// Free memory on this node (in bytes)
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
    ///
    /// Distribution strategy:
    /// 1. Prefer physical cores over hyperthreads (better for CPU-intensive work)
    /// 2. Distribute across NUMA nodes (balance memory bandwidth)
    /// 3. Round-robin assignment if workers > available CPUs
    ///
    /// Example on a 2-socket, 8-core system with hyperthreading:
    /// - Physical cores: 0,2,4,6 (socket 0), 8,10,12,14 (socket 1)
    /// - Workers 0-7 get one physical core each
    /// - Worker 8+ would reuse cores (round-robin)
    pub fn assign_workers(&mut self, num_workers: usize) -> Result<Vec<CpuAssignment>> {
        let mut assignments = Vec::new();
        
        if num_workers == 0 {
            return Ok(assignments);
        }

        // Get optimal CPU list (physical cores first, distributed across NUMA nodes)
        let available_cpus = self.get_optimal_cpu_list();
        
        for worker_id in 0..num_workers {
           
            // Round-robin assignment using modulo
            let cpu_id = available_cpus[worker_id % available_cpus.len()];
            
            // Lookup which NUMA node this CPU belongs to
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
    ///
    /// Uses the Linux sched_setaffinity system call to bind the current thread
    /// to a specific CPU. This is a direct interface to the kernel scheduler.
    #[cfg(target_os = "linux")]
    fn set_linux_affinity(&self, cpu_id: usize) -> Result<()> {
        use std::mem;
        
        if cpu_id >= self.topology.total_cpus {
            return Err(SystemError::resource_unavailable("CPU", 
                format!("CPU {} not available (total: {})", cpu_id, self.topology.total_cpus)
            ).into());
        }

        // Create CPU set - a bitmask where each bit represents a CPU
        // cpu_set_t is typically 1024 bits, supporting systems with up to 1024 CPUs
        let mut cpu_set: libc::cpu_set_t = unsafe { mem::zeroed() };
        
        // Set bit corresponding to our target CPU
        // CPU_SET is a macro that sets bit 'cpu_id' in the cpu_set bitmask
        unsafe {
            libc::CPU_SET(cpu_id, &mut cpu_set);
        }

        // tell kernel scheduler to only run this thread on the specified CPU
        let result = unsafe {
            libc::sched_setaffinity(
                0, // PID 0 means current thread/process
                mem::size_of::<libc::cpu_set_t>(), // Size of the CPU set structure
                &cpu_set, // Pointer to our CPU bitmask
            )
        };

        if result != 0 {
            return Err(SystemError::resource_unavailable("CPU", 
                format!("Failed to set CPU affinity to CPU {}: {}", cpu_id, 
                    std::io::Error::last_os_error())
            ).into());
        }

        Ok(())
    }


    /// Get optimal CPU list avoiding hyperthreads when possible
    ///
    /// Strategy: Prefer physical cores over logical cores (hyperthreads)
    /// because hyperthreads share execution units and can cause contention
    /// in CPU-intensive workloads like packet generation.
    fn get_optimal_cpu_list(&self) -> Vec<usize> {
        let mut cpus = Vec::new();
        
        if self.topology.hyperthreading_enabled {

            // Linux typically assigns CPUs as: 0,2,4,6 = physical cores
            //                                  1,3,5,7 = their hyperthreads
            // This is a heuristic - actual layout varies by system
            for node in &self.topology.numa_nodes {
                for &cpu in &node.cpus {
                    if cpu % 2 == 0 { // Even CPUs are typically physical cores
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
    ///
    /// Linux exposes CPU topology through virtual filesystems:
    /// - /proc/cpuinfo: Contains detailed CPU information
    /// - /sys/devices/system/node/: Contains NUMA topology
    /// - /sys/devices/system/cpu/: Contains CPU topology details
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
            .map_err(|e| SystemError::resource_unavailable("CPU", format!("Failed to read /proc/cpuinfo: {}", e)))?;
        
        let cpu_count = cpuinfo.lines()
            .filter(|line| line.starts_with("processor"))
            .count();
        
        if cpu_count == 0 {
            return Err(SystemError::resource_unavailable("CPU", "No CPUs detected".to_string()).into());
        }
        
        Ok(cpu_count)
    }

    /// Detect NUMA nodes
    ///
    /// NUMA nodes are exposed as directories under /sys/devices/system/node/
    /// Each nodeN directory contains:
    /// - cpulist: CPUs belonging to this node
    /// - meminfo: Memory information for this node
    /// - distance: NUMA distances to other nodes
    fn detect_numa_nodes(total_cpus: usize) -> Result<Vec<NumaNode>> {
        let mut nodes = Vec::new();
        
        // Each NUMA node appears as /sys/devices/system/node/nodeN
        if let Ok(entries) = fs::read_dir("/sys/devices/system/node/") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if let Some(stripped) = name_str.strip_prefix("node")
                    && let Ok(node_id) = stripped.parse::<usize>() {
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
        
        // Fallback for non-NUMA systems (UMA - Uniform Memory Access)
        // or when NUMA information is not available (e.g., in containers)
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
    ///
    /// Linux uses a compact format for CPU lists:
    /// - Single CPUs: "0,1,2"
    /// - Ranges: "0-3" means CPUs 0,1,2,3
    /// - Mixed: "0-3,8-11" means CPUs 0,1,2,3,8,9,10,11
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
    ///
    /// Reads from /sys/devices/system/node/nodeN/meminfo which contains:
    /// - MemTotal: Total memory on this NUMA node
    /// - MemFree: Currently free memory on this node
    /// - MemUsed: Memory in use (not always present)
    ///
    /// This helps identify memory pressure on specific NUMA nodes.
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
        if parts.len() >= 2
            && let Ok(kb) = parts[1].parse::<u64>() {
                return Some(kb * 1024); // Convert KB to bytes
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
    ///
    /// Compares physical core count with logical processor count.
    /// In /proc/cpuinfo:
    /// - "processor": Logical CPU ID (includes hyperthreads)
    /// - "core id": Physical core ID (unique per physical core)
    /// If logical > physical, hyperthreading is enabled.
    fn detect_hyperthreading() -> Result<bool> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")
            .map_err(|e| SystemError::resource_unavailable("CPU", format!("Failed to read /proc/cpuinfo: {}", e)))?;
        
        let mut physical_cores = std::collections::HashSet::new();
        let mut logical_cores = 0;
        
        for line in cpuinfo.lines() {
            if line.starts_with("processor") {
                logical_cores += 1;  // Count logical processors
            } else if line.starts_with("core id")
                && let Some(core_id) = line.split(':').nth(1)
                    && let Ok(id) = core_id.trim().parse::<usize>() {
                        physical_cores.insert(id);
                    }
        }
        
        Ok(logical_cores > physical_cores.len())
    }


    /// Get performance recommendations
    ///
    /// Analyzes the system topology and workload configuration to provide
    /// actionable performance tuning suggestions.
    pub fn get_performance_recommendations(&self, num_workers: usize) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Over-subscription causes context switching overhead
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
