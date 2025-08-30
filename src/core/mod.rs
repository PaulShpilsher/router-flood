//! Core engine components
//!
//! This module contains the core components that power the network testing engine.

pub mod network;
pub mod batch_worker;
pub mod packet_worker;
pub mod simulation;
pub mod simple_interfaces;
pub mod target;
pub mod traits;
pub mod worker;

// Re-export commonly used types
pub use simulation::{Simulation, SimulationRAII};
pub use target::MultiPortTarget;
// Use consolidated packet_worker as primary implementation
pub use packet_worker::{PacketWorker, PacketWorkerManager, WorkerManager};

// Re-export core traits from consolidated module
pub use traits::{
    StatsCollector, PacketBuilder as PacketBuilderTrait, TargetProvider, WorkerConfig
};

// Re-export implementations (will be cleaned up)
pub use simple_interfaces::{SimpleWorker, SimpleWorkerFactory, SimpleWorkerManager};

// Re-export batch worker components
pub use batch_worker::{
    BatchWorker, BatchWorkerManager, WorkerMetrics
};