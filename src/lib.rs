//! Router Flood - Educational Network Stress Testing Library
//!
//! # Disclaimer
//!
//! - The software is for educational and authorized testing purposes only.
//! - Unauthorized use (especially against systems you don't own or lack explicit permission to test) is strictly prohibited and may be illegal.

pub mod adapters;
pub mod audit;
pub mod buffer_pool;
pub mod cli;
pub mod config;
pub mod config_original;
pub mod constants;
pub mod error;
pub mod monitor;
pub mod monitoring;
pub mod network;
pub mod packet;
pub mod performance;
pub mod rng;
pub mod security;
pub mod simulation;
pub mod stats;
pub mod stats_original;
pub mod target;
pub mod transport;
pub mod transport_original;
pub mod ui;
pub mod validation;
pub mod worker;

// Re-export key types for easier access
pub use packet::{PacketBuilder, PacketStrategy, PacketType, Target};
pub use stats::{StatsCollector, SessionStats, SystemStats};
pub use transport::{TransportLayer, ChannelType};
pub use config::{ConfigBuilder, ConfigValidator};
pub use error::{RouterFloodError, Result};
