//! Core network engine components
//!
//! This module contains the core components that power the network testing engine.

use pnet::datalink::{self, NetworkInterface};

pub mod worker;
pub mod simulation;
pub mod target;
pub mod worker_manager;

// Re-export commonly used types
pub use simulation::Simulation;
pub use target::MultiPortTarget;
pub use worker_manager::Workers;

// Network interface utilities
pub fn list_network_interfaces() -> Vec<NetworkInterface> {
    datalink::interfaces()
}

pub fn find_interface_by_name(name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == name)
}

pub fn default_interface() -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
}