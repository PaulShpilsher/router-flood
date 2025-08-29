//! Chain of Responsibility pattern for packet processing
//!
//! This module implements a flexible chain of handlers for packet processing,
//! allowing for extensible packet modification and validation pipelines.

use super::Target;
use crate::error::{Result, PacketError};
use std::sync::Arc;

/// Context for packet processing through the chain
pub struct PacketContext {
    pub buffer: Vec<u8>,
    pub size: usize,
    pub target: Target,
    pub protocol: String,
    pub metadata: PacketMetadata,
}

/// Metadata associated with a packet
#[derive(Default, Clone)]
pub struct PacketMetadata {
    pub ttl: Option<u8>,
    pub flags: u32,
    pub sequence: u64,
    pub timestamp: u64,
    pub custom: std::collections::HashMap<String, String>,
}

impl PacketContext {
    /// Create a new packet context
    pub fn new(buffer: Vec<u8>, size: usize, target: Target, protocol: String) -> Self {
        Self {
            buffer,
            size,
            target,
            protocol,
            metadata: PacketMetadata::default(),
        }
    }
}

/// Result of processing a packet in the chain
pub enum ProcessResult {
    /// Continue to the next handler
    Continue,
    /// Stop processing and return success
    Complete,
    /// Stop processing with an error
    Abort(String),
}

/// Handler trait for packet processing chain
pub trait PacketHandler: Send + Sync {
    /// Process the packet context
    fn handle(&self, context: &mut PacketContext) -> Result<ProcessResult>;
    
    /// Get handler name for debugging
    fn name(&self) -> &str;
}

/// Chain of packet handlers
pub struct HandlerChain {
    handlers: Vec<Arc<dyn PacketHandler>>,
}

impl HandlerChain {
    /// Create a new handler chain
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    /// Add a handler to the chain
    pub fn add_handler(mut self, handler: Arc<dyn PacketHandler>) -> Self {
        self.handlers.push(handler);
        self
    }
    
    /// Process a packet through the chain
    pub fn process(&self, context: &mut PacketContext) -> Result<()> {
        for handler in &self.handlers {
            match handler.handle(context)? {
                ProcessResult::Continue => continue,
                ProcessResult::Complete => return Ok(()),
                ProcessResult::Abort(reason) => {
                    return Err(PacketError::InvalidParameters(reason).into());
                }
            }
        }
        Ok(())
    }
    
    /// Get the number of handlers in the chain
    pub fn len(&self) -> usize {
        self.handlers.len()
    }
    
    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

impl Default for HandlerChain {
    fn default() -> Self {
        Self::new()
    }
}

// === Built-in Handlers ===

/// Handler that validates packet size
pub struct SizeValidationHandler {
    min_size: usize,
    max_size: usize,
}

impl SizeValidationHandler {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        Self { min_size, max_size }
    }
}

impl PacketHandler for SizeValidationHandler {
    fn handle(&self, context: &mut PacketContext) -> Result<ProcessResult> {
        if context.size < self.min_size {
            return Ok(ProcessResult::Abort(
                format!("Packet size {} is below minimum {}", context.size, self.min_size)
            ));
        }
        if context.size > self.max_size {
            return Ok(ProcessResult::Abort(
                format!("Packet size {} exceeds maximum {}", context.size, self.max_size)
            ));
        }
        Ok(ProcessResult::Continue)
    }
    
    fn name(&self) -> &str {
        "SizeValidation"
    }
}

/// Handler that adds checksums to packets
pub struct ChecksumHandler;

impl PacketHandler for ChecksumHandler {
    fn handle(&self, context: &mut PacketContext) -> Result<ProcessResult> {
        // Simplified checksum calculation
        if context.size >= 2 {
            let mut sum: u32 = 0;
            for chunk in context.buffer[..context.size].chunks(2) {
                let value = if chunk.len() == 2 {
                    u16::from_be_bytes([chunk[0], chunk[1]]) as u32
                } else {
                    (chunk[0] as u32) << 8
                };
                sum = sum.wrapping_add(value);
            }
            
            while (sum >> 16) != 0 {
                sum = (sum & 0xffff) + (sum >> 16);
            }
            
            let checksum = !(sum as u16);
            context.metadata.custom.insert("checksum".to_string(), format!("{:04x}", checksum));
        }
        Ok(ProcessResult::Continue)
    }
    
    fn name(&self) -> &str {
        "Checksum"
    }
}

/// Handler that adds TTL to packets
pub struct TtlHandler {
    default_ttl: u8,
}

impl TtlHandler {
    pub fn new(default_ttl: u8) -> Self {
        Self { default_ttl }
    }
}

impl PacketHandler for TtlHandler {
    fn handle(&self, context: &mut PacketContext) -> Result<ProcessResult> {
        if context.metadata.ttl.is_none() {
            context.metadata.ttl = Some(self.default_ttl);
        }
        Ok(ProcessResult::Continue)
    }
    
    fn name(&self) -> &str {
        "TTL"
    }
}

/// Handler that logs packet information
pub struct LoggingHandler {
    verbose: bool,
}

impl LoggingHandler {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl PacketHandler for LoggingHandler {
    fn handle(&self, context: &mut PacketContext) -> Result<ProcessResult> {
        if self.verbose {
            println!("[PACKET] Protocol: {}, Target: {:?}, Size: {}", 
                context.protocol, context.target, context.size);
        }
        Ok(ProcessResult::Continue)
    }
    
    fn name(&self) -> &str {
        "Logging"
    }
}

/// Handler that rate limits packet processing
pub struct RateLimitHandler {
    last_time: std::sync::Mutex<std::time::Instant>,
    interval_ns: u64,
}

impl RateLimitHandler {
    pub fn new(max_rate: u64) -> Self {
        Self {
            last_time: std::sync::Mutex::new(std::time::Instant::now()),
            interval_ns: 1_000_000_000 / max_rate,
        }
    }
}

impl PacketHandler for RateLimitHandler {
    fn handle(&self, _context: &mut PacketContext) -> Result<ProcessResult> {
        let mut last_time = self.last_time.lock().unwrap();
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(*last_time).as_nanos() as u64;
        
        if elapsed < self.interval_ns {
            std::thread::sleep(std::time::Duration::from_nanos(self.interval_ns - elapsed));
        }
        
        *last_time = std::time::Instant::now();
        Ok(ProcessResult::Continue)
    }
    
    fn name(&self) -> &str {
        "RateLimit"
    }
}

/// Builder for creating handler chains
pub struct ChainBuilder {
    chain: HandlerChain,
}

impl ChainBuilder {
    /// Create a new chain builder
    pub fn new() -> Self {
        Self {
            chain: HandlerChain::new(),
        }
    }
    
    /// Add size validation
    pub fn with_size_validation(self, min: usize, max: usize) -> Self {
        self.add(Arc::new(SizeValidationHandler::new(min, max)))
    }
    
    /// Add checksum calculation
    pub fn with_checksum(self) -> Self {
        self.add(Arc::new(ChecksumHandler))
    }
    
    /// Add TTL handling
    pub fn with_ttl(self, default_ttl: u8) -> Self {
        self.add(Arc::new(TtlHandler::new(default_ttl)))
    }
    
    /// Add logging
    pub fn with_logging(self, verbose: bool) -> Self {
        self.add(Arc::new(LoggingHandler::new(verbose)))
    }
    
    /// Add rate limiting
    pub fn with_rate_limit(self, max_rate: u64) -> Self {
        self.add(Arc::new(RateLimitHandler::new(max_rate)))
    }
    
    /// Add a custom handler
    pub fn add(mut self, handler: Arc<dyn PacketHandler>) -> Self {
        self.chain = self.chain.add_handler(handler);
        self
    }
    
    /// Build the handler chain
    pub fn build(self) -> HandlerChain {
        self.chain
    }
}

impl Default for ChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}