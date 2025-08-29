//! Decorator pattern for packet modification
//!
//! This module implements the Decorator pattern for composable packet
//! modifications, allowing strategies to be enhanced with additional behavior.

use super::{PacketStrategy, Target};
use crate::error::Result;
use std::net::IpAddr;
use std::sync::Arc;

/// Base decorator that wraps a packet strategy
pub struct StrategyDecorator {
    inner: Arc<dyn PacketStrategy>,
}

impl StrategyDecorator {
    /// Create a new decorator wrapping a strategy
    pub fn new(strategy: Arc<dyn PacketStrategy>) -> Self {
        Self { inner: strategy }
    }
}

impl PacketStrategy for StrategyDecorator {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        // Cast to get mutable reference - simplified for example
        // In production, would use interior mutability
        Arc::get_mut(&mut self.inner)
            .ok_or_else(|| crate::error::PacketError::InvalidParameters("Cannot get mutable reference".into()))?
            .build_packet(target, buffer)
    }
    
    fn protocol_name(&self) -> &'static str {
        self.inner.protocol_name()
    }
    
    fn max_packet_size(&self) -> usize {
        self.inner.max_packet_size()
    }
    
    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        self.inner.is_compatible_with(target_ip)
    }
}

/// Decorator that adds fragmentation support
pub struct FragmentationDecorator {
    inner: Box<dyn PacketStrategy>,
    fragment_size: usize,
    fragment_offset: usize,
}

impl FragmentationDecorator {
    /// Create a new fragmentation decorator
    pub fn new(strategy: Box<dyn PacketStrategy>, fragment_size: usize) -> Self {
        Self {
            inner: strategy,
            fragment_size,
            fragment_offset: 0,
        }
    }
}

impl PacketStrategy for FragmentationDecorator {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let size = self.inner.build_packet(target, buffer)?;
        
        // Add fragmentation headers if packet is large
        if size > self.fragment_size {
            // Simplified fragmentation logic
            let fragment_size = self.fragment_size.min(size - self.fragment_offset);
            self.fragment_offset += fragment_size;
            
            if self.fragment_offset >= size {
                self.fragment_offset = 0;
            }
            
            Ok(fragment_size)
        } else {
            Ok(size)
        }
    }
    
    fn protocol_name(&self) -> &'static str {
        self.inner.protocol_name()
    }
    
    fn max_packet_size(&self) -> usize {
        self.fragment_size
    }
    
    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        self.inner.is_compatible_with(target_ip)
    }
}

/// Decorator that adds encryption
pub struct EncryptionDecorator {
    inner: Box<dyn PacketStrategy>,
    key: Vec<u8>,
}

impl EncryptionDecorator {
    /// Create a new encryption decorator
    pub fn new(strategy: Box<dyn PacketStrategy>, key: Vec<u8>) -> Self {
        Self { inner: strategy, key }
    }
    
    /// Simple XOR encryption for demonstration
    fn encrypt(&self, data: &mut [u8]) {
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= self.key[i % self.key.len()];
        }
    }
}

impl PacketStrategy for EncryptionDecorator {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let size = self.inner.build_packet(target, buffer)?;
        
        // Encrypt the packet payload
        if size > 0 {
            self.encrypt(&mut buffer[..size]);
        }
        
        Ok(size)
    }
    
    fn protocol_name(&self) -> &'static str {
        self.inner.protocol_name()
    }
    
    fn max_packet_size(&self) -> usize {
        self.inner.max_packet_size()
    }
    
    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        self.inner.is_compatible_with(target_ip)
    }
}

/// Decorator that adds compression
pub struct CompressionDecorator {
    inner: Box<dyn PacketStrategy>,
    compression_level: u8,
}

impl CompressionDecorator {
    /// Create a new compression decorator
    pub fn new(strategy: Box<dyn PacketStrategy>, compression_level: u8) -> Self {
        Self {
            inner: strategy,
            compression_level,
        }
    }
    
    /// Simple RLE compression for demonstration
    fn compress(&self, data: &[u8], output: &mut [u8]) -> usize {
        if data.is_empty() {
            return 0;
        }
        
        let mut out_idx = 0;
        let mut i = 0;
        
        while i < data.len() && out_idx < output.len() - 1 {
            let start = i;
            let byte = data[i];
            
            while i < data.len() && data[i] == byte && (i - start) < 255 {
                i += 1;
            }
            
            output[out_idx] = (i - start) as u8;
            output[out_idx + 1] = byte;
            out_idx += 2;
        }
        
        out_idx
    }
}

impl PacketStrategy for CompressionDecorator {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        let size = self.inner.build_packet(target, buffer)?;
        
        // Compress the packet if beneficial
        if self.compression_level > 0 && size > 100 {
            let mut compressed = vec![0u8; size];
            let compressed_size = self.compress(&buffer[..size], &mut compressed);
            
            if compressed_size < size {
                buffer[..compressed_size].copy_from_slice(&compressed[..compressed_size]);
                return Ok(compressed_size);
            }
        }
        
        Ok(size)
    }
    
    fn protocol_name(&self) -> &'static str {
        self.inner.protocol_name()
    }
    
    fn max_packet_size(&self) -> usize {
        self.inner.max_packet_size()
    }
    
    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        self.inner.is_compatible_with(target_ip)
    }
}

/// Decorator that adds timing jitter
pub struct JitterDecorator {
    inner: Box<dyn PacketStrategy>,
    jitter_ms: u64,
}

impl JitterDecorator {
    /// Create a new jitter decorator
    pub fn new(strategy: Box<dyn PacketStrategy>, jitter_ms: u64) -> Self {
        Self {
            inner: strategy,
            jitter_ms,
        }
    }
}

impl PacketStrategy for JitterDecorator {
    fn build_packet(&mut self, target: &Target, buffer: &mut [u8]) -> Result<usize> {
        // Add random delay before building packet
        if self.jitter_ms > 0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let delay = rng.gen_range(0..self.jitter_ms);
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        
        self.inner.build_packet(target, buffer)
    }
    
    fn protocol_name(&self) -> &'static str {
        self.inner.protocol_name()
    }
    
    fn max_packet_size(&self) -> usize {
        self.inner.max_packet_size()
    }
    
    fn is_compatible_with(&self, target_ip: IpAddr) -> bool {
        self.inner.is_compatible_with(target_ip)
    }
}

/// Builder for creating decorated strategies
pub struct DecoratorBuilder {
    strategy: Box<dyn PacketStrategy>,
}

impl DecoratorBuilder {
    /// Create a new decorator builder
    pub fn new(strategy: Box<dyn PacketStrategy>) -> Self {
        Self { strategy }
    }
    
    /// Add fragmentation support
    pub fn with_fragmentation(mut self, fragment_size: usize) -> Self {
        self.strategy = Box::new(FragmentationDecorator::new(self.strategy, fragment_size));
        self
    }
    
    /// Add encryption
    pub fn with_encryption(mut self, key: Vec<u8>) -> Self {
        self.strategy = Box::new(EncryptionDecorator::new(self.strategy, key));
        self
    }
    
    /// Add compression
    pub fn with_compression(mut self, level: u8) -> Self {
        self.strategy = Box::new(CompressionDecorator::new(self.strategy, level));
        self
    }
    
    /// Add timing jitter
    pub fn with_jitter(mut self, jitter_ms: u64) -> Self {
        self.strategy = Box::new(JitterDecorator::new(self.strategy, jitter_ms));
        self
    }
    
    /// Build the decorated strategy
    pub fn build(self) -> Box<dyn PacketStrategy> {
        self.strategy
    }
}

/// Macro for easily creating decorated strategies
#[macro_export]
macro_rules! decorate_strategy {
    ($strategy:expr) => {
        $crate::packet::decorator::DecoratorBuilder::new(Box::new($strategy))
    };
    ($strategy:expr, $($decorator:ident($($args:expr),*)),* $(,)?) => {
        {
            let builder = $crate::packet::decorator::DecoratorBuilder::new(Box::new($strategy));
            $(
                let builder = builder.$decorator($($args),*);
            )*
            builder.build()
        }
    };
}