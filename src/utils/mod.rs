//! Utility modules and helper functions
//!
//! This module contains essential utility functions used throughout the application.

pub mod protocol_utils;
pub mod raii;
pub mod rng;
pub mod terminal;

// Re-export commonly used types
pub use protocol_utils::{ProtocolUtils, PacketTypeExt};
pub use raii::{ResourceGuard, SignalGuard, StatsGuard, TerminalRAIIGuard, WorkerGuard};
pub use rng::BatchedRng;
pub use terminal::TerminalGuard;