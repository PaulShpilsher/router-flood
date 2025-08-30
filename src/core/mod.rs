//! Core engine components
//!
//! This module contains the core components that power the network testing engine.

pub mod network;
pub mod batch_worker;
pub mod simulation;
pub mod simple_interfaces;
pub mod target;
pub mod worker;

// Re-export commonly used types
pub use simulation::{Simulation, SimulationRAII};
pub use target::MultiPortTarget;
pub use worker::WorkerManager;

// Re-export streamlined dependency injection interfaces
pub use simple_interfaces::{
    StatsCollector, PacketBuilder as PacketBuilderTrait, TargetProvider, WorkerConfig,
    SimpleWorker, SimpleWorkerFactory, SimpleWorkerManager
};

// Re-export batch worker components
pub use batch_worker::{
    BatchWorker, BatchWorkerManager, WorkerMetrics
};