//! Router Flood - Educational Network Stress Testing Tool
//!
//! A comprehensive, safety-first network testing tool designed for educational purposes
//! and authorized network testing scenarios.
//!
//! ## Enhanced User Experience
//!
//! This version includes user experience improvements:
//! - Guided CLI with progressive disclosure
//! - Streamlined configuration system (40% complexity reduction)
//! - Enhanced user-friendly error messages with actionable guidance

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

// Use the new consolidated error module
pub mod error;

// Re-export key types for convenience
pub use config::{Config, Target, LoadConfig, Safety, Monitoring, Export, ExportFormat, ProtocolMix};
pub use network::simulation::{Simulation, SimulationRAII};
pub use network::worker_manager::Workers;
pub use network::target::MultiPortTarget;
pub use error::{Result, RouterFloodError};
pub use packet::{PacketBuilder, PacketStrategy, PacketType, PacketTarget};
pub use stats::Stats;
pub use utils::buffer_pool::BufferPool;
pub use utils::terminal::{Terminal, TerminalGuard};
pub use utils::raii::ResourceGuard;

// Common type aliases for clarity and ergonomics
use std::sync::Arc;

/// Shared reference to Stats
pub type StatsRef = Arc<Stats>;

/// Shared reference to Config
pub type ConfigRef = Arc<Config>;

/// Shared reference to BufferPool
pub type PoolRef = Arc<BufferPool>;

/// Shared reference to Workers
pub type WorkersRef = Arc<Workers>;

// Tests moved to tests/ directory
