//! Core engine components
//!
//! This module contains the core components that power the network testing engine.

pub mod network;
pub mod worker;
pub mod simulation;
pub mod target;
pub mod traits;
pub mod worker_manager;

// Re-export commonly used types
pub use simulation::{Simulation, SimulationRAII};
pub use target::MultiPortTarget;
pub use worker_manager::Workers;

// Re-export core traits from consolidated module
pub use traits::{
    StatsCollector, PacketBuilder as PacketBuilderTrait, TargetProvider, WorkerConfig
};