//! Transport layer abstractions
//!
//! This module provides trait-based abstractions for different transport
//! mechanisms, enabling easy testing and multiple implementations.

pub mod layer;
pub mod mock;

pub use layer::{TransportLayer, ChannelType};
pub use mock::MockTransport;

// Note: RawSocketTransport temporarily disabled due to thread safety issues
// Will be re-enabled in Phase 3 with proper thread-safe implementation

// Re-export existing transport types for backward compatibility
pub use crate::transport_original::{WorkerChannels, ChannelFactory};