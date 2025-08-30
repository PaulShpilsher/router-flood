//! Core engine components
//!
//! This module contains the core components that power the network testing engine.

pub mod network;
pub mod batch_worker;
pub mod simulation;
pub mod target;
pub mod traits;
pub mod worker;

// Re-export commonly used types
pub use simulation::{Simulation, SimulationRAII};
pub use target::MultiPortTarget;
pub use worker::WorkerManager;

// Re-export core traits from consolidated module
pub use traits::{
    StatsCollector, PacketBuilder as PacketBuilderTrait, TargetProvider, WorkerConfig
};