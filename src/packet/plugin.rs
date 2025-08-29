//! Plugin system for extensible packet strategies
//!
//! This module provides a plugin architecture for dynamically registering
//! packet strategies, following the Plugin and Registry patterns.

use super::{PacketStrategy, PacketType};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Plugin trait for packet strategy providers
pub trait StrategyPlugin: Send + Sync {
    /// Name of the plugin
    fn name(&self) -> &str;
    
    /// Version of the plugin
    fn version(&self) -> &str;
    
    /// Initialize the plugin
    fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    /// Get the strategies provided by this plugin
    fn strategies(&self) -> Vec<(PacketType, Box<dyn PacketStrategy>)>;
    
    /// Shutdown the plugin
    fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

/// Plugin registry for managing strategy plugins
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<dyn StrategyPlugin>>>,
    strategies: RwLock<HashMap<PacketType, Vec<Arc<dyn PacketStrategy>>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            strategies: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a plugin
    pub fn register_plugin(&self, plugin: Arc<dyn StrategyPlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        
        // Initialize the plugin
        plugin.initialize()?;
        
        // Register all strategies from the plugin
        for (packet_type, strategy) in plugin.strategies() {
            self.register_strategy(packet_type, Arc::from(strategy))?;
        }
        
        // Store the plugin
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(name, plugin);
        
        Ok(())
    }
    
    /// Register a single strategy
    fn register_strategy(&self, packet_type: PacketType, strategy: Arc<dyn PacketStrategy>) -> Result<()> {
        let mut strategies = self.strategies.write().unwrap();
        strategies.entry(packet_type)
            .or_insert_with(Vec::new)
            .push(strategy);
        Ok(())
    }
    
    /// Get all strategies for a packet type
    pub fn get_strategies(&self, packet_type: PacketType) -> Vec<Arc<dyn PacketStrategy>> {
        let strategies = self.strategies.read().unwrap();
        strategies.get(&packet_type)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get all registered plugins
    pub fn plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }
    
    /// Unregister a plugin
    pub fn unregister_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(plugin) = plugins.remove(name) {
            plugin.shutdown()?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}



