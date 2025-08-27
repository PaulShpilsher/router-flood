//! Compatibility adapters between old and new architecture
//!
//! This module provides adapters to bridge the gap between the original
//! implementation and the new trait-based architecture, enabling gradual migration.

pub mod channel_adapter;
pub mod stats_adapter;

pub use channel_adapter::ChannelTypeAdapter;
pub use stats_adapter::SystemStatsAdapter;