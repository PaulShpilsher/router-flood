//! Core engine components
//!
//! This module contains the core components that power the network testing engine.

pub mod network;
pub mod simulation;
pub mod target;
pub mod worker;

// Re-export commonly used types
pub use simulation::{Simulation, SimulationRAII};
pub use target::MultiPortTarget;
pub use worker::WorkerManager;