//! High-performance network stress testing library for authorized testing.
//!
//! router-flood provides a safe, capability-based framework for testing network
//! infrastructure resilience. The library enforces strict safety controls including
//! private IP validation and rate limiting.
//!
//! # Safety
//!
//! This library enforces RFC 1918 private IP ranges and requires appropriate
//! network capabilities (CAP_NET_RAW) for operation.
//!
//! # Example
//!
//! ```no_run
//! use router_flood::{Config, Simulation};
//! use std::net::IpAddr;
//!
//! # async fn example() -> router_flood::Result<()> {
//! let config = Config::default();
//! let target_ip: IpAddr = "192.168.1.1".parse().unwrap();
//! let mut simulation = Simulation::new(config, target_ip, None);
//! simulation.run().await?;
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod config;
pub mod constants;
pub mod network;
pub mod system_monitor;
pub mod packet;
pub mod performance;
pub mod cli_runner;
pub mod security_runner;
pub mod security;
pub mod stats;
pub mod transport;
pub mod ui;
pub mod utils;

pub mod error;

// Re-export key types
pub use config::{Config, Target, LoadConfig, Safety, Monitoring, Export, ExportFormat, ProtocolMix, Audit};
pub use network::engine::Engine;
pub use network::worker_manager::Workers;
pub use network::target::PortTarget;
pub use error::{Result, RouterFloodError};
pub use packet::{PacketBuilder, PacketStrategy, PacketType, PacketTarget};
pub use stats::Stats;
pub use utils::terminal::{Terminal, TerminalGuard};
pub use utils::raii::ResourceGuard;
pub use security::AuditLogger;

use std::sync::Arc;

/// Shared reference to statistics collector
pub type StatsRef = Arc<Stats>;

/// Shared reference to configuration
pub type ConfigRef = Arc<Config>;

/// Shared reference to worker manager
pub type WorkersRef = Arc<Workers>;