//! Transport layer abstractions
//!
//! This module provides trait-based abstractions for different transport
//! mechanisms, enabling easy testing and multiple implementations.

pub mod layer;
pub mod raw_socket;
pub mod mock;

pub use layer::{TransportLayer, ChannelType};
pub use raw_socket::RawSocketTransport;
pub use mock::MockTransport;

// Re-export existing transport types for backward compatibility
pub use crate::transport_original::{WorkerChannels, ChannelFactory};