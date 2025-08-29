//! Router Flood - Educational Network Stress Testing Tool

// Allow clippy warnings for format strings and other style issues
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::print_literal)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::let_and_return)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::useless_format)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::manual_strip)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::nonminimal_bool)]
//!
//! # Disclaimer
//!
//! - The software is for educational and authorized testing purposes only.
//! - Unauthorized use (especially against systems you don't own or lack explicit permission to test) is strictly prohibited and may be illegal.

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
