//! Router Flood - Educational Network Stress Testing Tool
//!
//! A comprehensive, safety-first network testing tool designed for educational purposes
//! and authorized network testing scenarios.
//!
//! ## Phase 4 - User Experience Enhancement
//!
//! This version includes Phase 4 improvements:
//! - Simplified CLI with progressive disclosure
//! - Streamlined configuration system (40% complexity reduction)
//! - Enhanced user-friendly error messages with actionable guidance

pub mod abstractions;
pub mod audit;
pub mod cli;
pub mod config;
pub mod constants;
pub mod core;
pub mod error;
pub mod monitor;
pub mod monitoring;
pub mod packet;
pub mod performance;
pub mod phase4;
// pub mod phase5; // Temporarily commented for compilation
pub mod security;
pub mod stats;
pub mod transport;
pub mod ui;
pub mod utils;
pub mod validation;

// Re-export key types for convenience
pub use config::{Config, TargetConfig, AttackConfig, SafetyConfig, MonitoringConfig, ExportConfig, ExportFormat, ProtocolMix};
pub use core::simulation::{Simulation, SimulationRAII};
pub use core::worker::WorkerManager;
pub use core::target::MultiPortTarget;
pub use error::{Result, RouterFloodError};
pub use packet::{PacketBuilder, PacketStrategy, PacketType, Target};
pub use stats::FloodStats;
pub use utils::buffer_pool::{BufferPool, WorkerBufferPool};
pub use utils::terminal::{TerminalController, TerminalGuard};
pub use utils::raii::ResourceGuard;

#[cfg(test)]
mod extensibility_tests_simple;
