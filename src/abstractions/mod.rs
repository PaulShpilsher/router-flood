//! Abstractions for external dependencies
//!
//! This module provides trait-based abstractions for external libraries
//! to improve testability and reduce coupling.

pub mod network;
pub mod system;

pub use network::NetworkProvider;
pub use system::SystemProvider;