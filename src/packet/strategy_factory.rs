//! Factory pattern for packet strategies
//!
//! This module provides a consistent factory pattern for creating packet strategies,
//! following the Abstract Factory and Registry patterns.

use super::{PacketStrategy, PacketType};
use super::strategies::{UdpStrategy, TcpStrategy, IcmpStrategy};
use crate::error::{Result, RouterFloodError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for packet strategy factories
pub trait StrategyFactory: Send + Sync {
    /// Create a new strategy instance
    fn create(&self) -> Box<dyn PacketStrategy>;
    
    /// Get the packet type this factory creates
    fn packet_type(&self) -> PacketType;
    
    /// Get a description of the strategy
    fn description(&self) -> &str;
}

/// Factory for UDP packet strategies
pub struct UdpStrategyFactory;

impl StrategyFactory for UdpStrategyFactory {
    fn create(&self) -> Box<dyn PacketStrategy> {
        let mut rng = crate::utils::rng::BatchedRng::new();
        Box::new(UdpStrategy::new((64, 1400), &mut rng))
    }
    
    fn packet_type(&self) -> PacketType {
        PacketType::Udp
    }
    
    fn description(&self) -> &str {
        "UDP packet strategy for connectionless datagrams"
    }
}

/// Factory for TCP SYN packet strategies
pub struct TcpSynStrategyFactory;

impl StrategyFactory for TcpSynStrategyFactory {
    fn create(&self) -> Box<dyn PacketStrategy> {
        let mut rng = crate::utils::rng::BatchedRng::new();
        Box::new(TcpStrategy::new_syn(&mut rng))
    }
    
    fn packet_type(&self) -> PacketType {
        PacketType::TcpSyn
    }
    
    fn description(&self) -> &str {
        "TCP SYN packet strategy for connection initiation"
    }
}

/// Factory for TCP ACK packet strategies
pub struct TcpAckStrategyFactory;

impl StrategyFactory for TcpAckStrategyFactory {
    fn create(&self) -> Box<dyn PacketStrategy> {
        let mut rng = crate::utils::rng::BatchedRng::new();
        Box::new(TcpStrategy::new_ack(&mut rng))
    }
    
    fn packet_type(&self) -> PacketType {
        PacketType::TcpAck
    }
    
    fn description(&self) -> &str {
        "TCP ACK packet strategy for acknowledgments"
    }
}

/// Factory for ICMP packet strategies
pub struct IcmpStrategyFactory;

impl StrategyFactory for IcmpStrategyFactory {
    fn create(&self) -> Box<dyn PacketStrategy> {
        let mut rng = crate::utils::rng::BatchedRng::new();
        Box::new(IcmpStrategy::new(&mut rng))
    }
    
    fn packet_type(&self) -> PacketType {
        PacketType::Icmp
    }
    
    fn description(&self) -> &str {
        "ICMP packet strategy for echo requests"
    }
}

/// Registry for packet strategy factories
pub struct StrategyRegistry {
    factories: RwLock<HashMap<PacketType, Arc<dyn StrategyFactory>>>,
}

impl StrategyRegistry {
    /// Create a new strategy registry with default factories
    pub fn new() -> Self {
        let mut registry = Self {
            factories: RwLock::new(HashMap::new()),
        };
        registry.register_defaults();
        registry
    }
    
    /// Register default factories
    fn register_defaults(&mut self) {
        self.register(Arc::new(UdpStrategyFactory));
        self.register(Arc::new(TcpSynStrategyFactory));
        self.register(Arc::new(TcpAckStrategyFactory));
        self.register(Arc::new(IcmpStrategyFactory));
    }
    
    /// Register a strategy factory
    pub fn register(&self, factory: Arc<dyn StrategyFactory>) {
        let mut factories = self.factories.write().unwrap();
        factories.insert(factory.packet_type(), factory);
    }
    
    /// Create a strategy for the given packet type
    pub fn create_strategy(&self, packet_type: PacketType) -> Result<Box<dyn PacketStrategy>> {
        let factories = self.factories.read().unwrap();
        factories
            .get(&packet_type)
            .map(|factory| factory.create())
            .ok_or_else(|| RouterFloodError::Packet(crate::error::PacketError::InvalidParameters(format!("Unsupported protocol: {:?}", packet_type))))
    }
    
    /// Get all registered packet types
    pub fn registered_types(&self) -> Vec<PacketType> {
        let factories = self.factories.read().unwrap();
        factories.keys().cloned().collect()
    }
    
    /// Get factory for a packet type
    pub fn get_factory(&self, packet_type: PacketType) -> Option<Arc<dyn StrategyFactory>> {
        let factories = self.factories.read().unwrap();
        factories.get(&packet_type).cloned()
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::OnceLock;

/// Global strategy registry singleton
static GLOBAL_REGISTRY: OnceLock<StrategyRegistry> = OnceLock::new();

/// Get the global strategy registry
pub fn global_registry() -> &'static StrategyRegistry {
    GLOBAL_REGISTRY.get_or_init(StrategyRegistry::new)
}

/// Builder for configuring packet strategies
pub struct StrategyBuilder {
    packet_type: Option<PacketType>,
    custom_factory: Option<Arc<dyn StrategyFactory>>,
}

impl StrategyBuilder {
    /// Create a new strategy builder
    pub fn new() -> Self {
        Self {
            packet_type: None,
            custom_factory: None,
        }
    }
    
    /// Set the packet type
    pub fn packet_type(mut self, packet_type: PacketType) -> Self {
        self.packet_type = Some(packet_type);
        self
    }
    
    /// Set a custom factory
    pub fn custom_factory(mut self, factory: Arc<dyn StrategyFactory>) -> Self {
        self.custom_factory = Some(factory);
        self
    }
    
    /// Build the strategy
    pub fn build(self) -> Result<Box<dyn PacketStrategy>> {
        if let Some(factory) = self.custom_factory {
            Ok(factory.create())
        } else if let Some(packet_type) = self.packet_type {
            global_registry().create_strategy(packet_type)
        } else {
            Err(RouterFloodError::Packet(crate::error::PacketError::InvalidParameters(
                "No packet type or factory specified".into()
            )))
        }
    }
}

impl Default for StrategyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to define a new strategy factory
#[macro_export]
macro_rules! define_strategy_factory {
    ($name:ident, $strategy:ty, $packet_type:expr, $description:literal) => {
        pub struct $name;
        
        impl $crate::packet::strategy_factory::StrategyFactory for $name {
            fn create(&self) -> Box<dyn $crate::packet::PacketStrategy> {
                Box::new(<$strategy>::new())
            }
            
            fn packet_type(&self) -> $crate::packet::PacketType {
                $packet_type
            }
            
            fn description(&self) -> &str {
                $description
            }
        }
    };
}