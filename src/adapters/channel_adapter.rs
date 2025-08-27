//! Channel type compatibility adapter

use crate::transport::layer::ChannelType as NewChannelType;
use crate::transport_original::ChannelType as OriginalChannelType;

/// Adapter to convert between old and new ChannelType enums
pub struct ChannelTypeAdapter;

impl ChannelTypeAdapter {
    /// Convert from new ChannelType to original ChannelType
    #[inline]
    pub fn to_original(channel_type: NewChannelType) -> OriginalChannelType {
        match channel_type {
            NewChannelType::IPv4 => OriginalChannelType::IPv4,
            NewChannelType::IPv6 => OriginalChannelType::IPv6,
            NewChannelType::Layer2 => OriginalChannelType::Layer2,
        }
    }
    
    /// Convert from original ChannelType to new ChannelType
    #[inline]
    pub fn from_original(channel_type: OriginalChannelType) -> NewChannelType {
        match channel_type {
            OriginalChannelType::IPv4 => NewChannelType::IPv4,
            OriginalChannelType::IPv6 => NewChannelType::IPv6,
            OriginalChannelType::Layer2 => NewChannelType::Layer2,
        }
    }
}