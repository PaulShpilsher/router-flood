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
pub mod core;
pub mod error;
pub mod system_monitor;
pub mod monitoring;
pub mod packet;
pub mod performance;
pub mod cli_runner;
pub mod security_runner;
pub mod security;
pub mod stats;
pub mod transport;
pub mod ui;
pub mod utils;

// Re-export key types for convenience
pub use config::{Config, TargetConfig, AttackConfig, SafetyConfig, MonitoringConfig, ExportConfig, ExportFormat, ProtocolMix};
pub use core::simulation::{Simulation, SimulationRAII};
pub use core::worker_manager::WorkerManager;
pub use core::target::MultiPortTarget;
pub use error::{Result, RouterFloodError};
pub use packet::{PacketBuilder, PacketStrategy, PacketType, Target};
pub use stats::Stats;
pub use utils::buffer_pool::BufferPool;
pub use utils::terminal::{Terminal, TerminalGuard};
pub use utils::raii::ResourceGuard;

// Tests moved to tests/ directory
