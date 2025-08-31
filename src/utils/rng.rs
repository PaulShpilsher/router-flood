//! Optimized random number generation with batching
//!
//! This module provides high-performance RNG with pre-computed batches
//! to reduce the overhead of generating random numbers in hot paths.

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::VecDeque;

/// Batch size for pre-generating random numbers
pub const DEFAULT_BATCH_SIZE: usize = 1000;

/// Different types of random values needed for packet generation
#[derive(Debug, Clone, Copy)]
pub enum RandomValueType {
    Port,        // 16-bit port numbers (1024-65535)
    Sequence,    // 32-bit sequence numbers
    Identification, // 16-bit IP identification
    Ttl,         // 8-bit TTL (32-128)
    Window,      // 16-bit TCP window size
    FlowLabel,   // 20-bit IPv6 flow label
    Byte,        // 8-bit random byte for payload
}

/// Optimized RNG with batched random value generation
pub struct BatchedRng {
    rng: StdRng,
    port_batch: VecDeque<u16>,
    sequence_batch: VecDeque<u32>,
    id_batch: VecDeque<u16>,
    ttl_batch: VecDeque<u8>,
    window_batch: VecDeque<u16>,
    flow_batch: VecDeque<u32>,
    byte_batch: VecDeque<u8>,
    batch_size: usize,
}

impl BatchedRng {
    /// Create a new batched RNG with default batch size
    pub fn new() -> Self {
        Self::with_batch_size(DEFAULT_BATCH_SIZE)
    }

    /// Create a new batched RNG with custom batch size
    pub fn with_batch_size(batch_size: usize) -> Self {
        let rng = StdRng::from_entropy();
        let mut batched_rng = Self {
            rng,
            port_batch: VecDeque::with_capacity(batch_size),
            sequence_batch: VecDeque::with_capacity(batch_size),
            id_batch: VecDeque::with_capacity(batch_size),
            ttl_batch: VecDeque::with_capacity(batch_size),
            window_batch: VecDeque::with_capacity(batch_size),
            flow_batch: VecDeque::with_capacity(batch_size),
            byte_batch: VecDeque::with_capacity(batch_size),
            batch_size,
        };
        
        // Pre-populate all batches
        batched_rng.replenish_all_batches();
        batched_rng
    }

    /// Get a random port number (1024-65535)
    #[inline]
    pub fn port(&mut self) -> u16 {
        if self.port_batch.is_empty() {
            self.replenish_port_batch();
        }
        self.port_batch.pop_front().unwrap_or(1024)
    }

    /// Get a random 32-bit sequence number
    #[inline]
    pub fn sequence(&mut self) -> u32 {
        if self.sequence_batch.is_empty() {
            self.replenish_sequence_batch();
        }
        self.sequence_batch.pop_front().unwrap_or(0)
    }

    /// Get a random 16-bit identification
    #[inline]
    pub fn identification(&mut self) -> u16 {
        if self.id_batch.is_empty() {
            self.replenish_id_batch();
        }
        self.id_batch.pop_front().unwrap_or(1)
    }

    /// Get a random TTL value (32-128)
    #[inline]
    pub fn ttl(&mut self) -> u8 {
        if self.ttl_batch.is_empty() {
            self.replenish_ttl_batch();
        }
        self.ttl_batch.pop_front().unwrap_or(64)
    }

    /// Get a random TCP window size
    pub fn window_size(&mut self) -> u16 {
        if self.window_batch.is_empty() {
            self.replenish_window_batch();
        }
        self.window_batch.pop_front().unwrap_or(8192)
    }

    /// Get a random IPv6 flow label (20-bit)
    pub fn flow_label(&mut self) -> u32 {
        if self.flow_batch.is_empty() {
            self.replenish_flow_batch();
        }
        self.flow_batch.pop_front().unwrap_or(0)
    }

    /// Get a random byte for payload generation
    pub fn byte(&mut self) -> u8 {
        if self.byte_batch.is_empty() {
            self.replenish_byte_batch();
        }
        self.byte_batch.pop_front().unwrap_or(0)
    }

    /// Generate random payload of specified size
    pub fn payload(&mut self, size: usize) -> Vec<u8> {
        // For large payloads, generate directly instead of using byte batches
        if size > self.batch_size / 4 {
            let mut payload = vec![0u8; size];
            self.rng.fill(&mut payload[..]);
            return payload;
        }
        
        let mut payload = Vec::with_capacity(size);
        
        // Ensure we have enough bytes in batch
        while self.byte_batch.len() < size {
            self.replenish_byte_batch();
        }
        
        for _ in 0..size {
            payload.push(self.byte_batch.pop_front().unwrap_or(0));
        }
        
        payload
    }

    /// Generate random boolean with given probability
    #[inline(always)]
    pub fn bool_with_probability(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability)
    }

    /// Generate random value in range
    #[inline(always)]
    pub fn range(&mut self, min: usize, max: usize) -> usize {
        self.rng.gen_range(min..max)
    }

    /// Generate random float in range
    #[inline(always)]
    pub fn float_range(&mut self, min: f64, max: f64) -> f64 {
        self.rng.gen_range(min..max)
    }

    /// Replenish all batches when they get low
    fn replenish_all_batches(&mut self) {
        self.replenish_port_batch();
        self.replenish_sequence_batch();
        self.replenish_id_batch();
        self.replenish_ttl_batch();
        self.replenish_window_batch();
        self.replenish_flow_batch();
        self.replenish_byte_batch();
    }

    /// Replenish port number batch
    fn replenish_port_batch(&mut self) {
        for _ in 0..self.batch_size {
            self.port_batch.push_back(self.rng.gen_range(1024..65535));
        }
    }

    /// Replenish sequence number batch
    fn replenish_sequence_batch(&mut self) {
        for _ in 0..self.batch_size {
            self.sequence_batch.push_back(self.rng.gen_range(0..u32::MAX));
        }
    }

    /// Replenish identification batch
    fn replenish_id_batch(&mut self) {
        for _ in 0..self.batch_size {
            self.id_batch.push_back(self.rng.gen_range(0..u16::MAX));
        }
    }

    /// Replenish TTL batch
    fn replenish_ttl_batch(&mut self) {
        for _ in 0..self.batch_size {
            self.ttl_batch.push_back(self.rng.gen_range(32..128));
        }
    }

    /// Replenish window size batch
    fn replenish_window_batch(&mut self) {
        for _ in 0..self.batch_size {
            self.window_batch.push_back(self.rng.gen_range(1024..65535));
        }
    }

    /// Replenish IPv6 flow label batch
    fn replenish_flow_batch(&mut self) {
        for _ in 0..self.batch_size {
            self.flow_batch.push_back(self.rng.gen_range(0..=0xFFFFF));
        }
    }

    /// Replenish byte batch
    fn replenish_byte_batch(&mut self) {
        for _ in 0..self.batch_size {
            self.byte_batch.push_back(self.rng.gen_range(0..=255));
        }
    }

    /// Get remaining count for a specific batch (for testing/monitoring)
    pub fn batch_remaining(&self, value_type: RandomValueType) -> usize {
        match value_type {
            RandomValueType::Port => self.port_batch.len(),
            RandomValueType::Sequence => self.sequence_batch.len(),
            RandomValueType::Identification => self.id_batch.len(),
            RandomValueType::Ttl => self.ttl_batch.len(),
            RandomValueType::Window => self.window_batch.len(),
            RandomValueType::FlowLabel => self.flow_batch.len(),
            RandomValueType::Byte => self.byte_batch.len(),
        }
    }

    /// Get batch size configuration
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

impl Default for BatchedRng {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common use cases
impl BatchedRng {
    /// Get multiple random ports at once
    pub fn ports(&mut self, count: usize) -> Vec<u16> {
        let mut ports = Vec::with_capacity(count);
        for _ in 0..count {
            ports.push(self.port());
        }
        ports
    }

    /// Get multiple random TTL values at once
    pub fn ttls(&mut self, count: usize) -> Vec<u8> {
        let mut ttls = Vec::with_capacity(count);
        for _ in 0..count {
            ttls.push(self.ttl());
        }
        ttls
    }

    /// Check if any batch needs replenishing
    pub fn needs_replenishment(&self) -> bool {
        let threshold = self.batch_size / 4; // Replenish when 25% remain
        
        self.port_batch.len() < threshold ||
        self.sequence_batch.len() < threshold ||
        self.id_batch.len() < threshold ||
        self.ttl_batch.len() < threshold ||
        self.window_batch.len() < threshold ||
        self.flow_batch.len() < threshold ||
        self.byte_batch.len() < threshold
    }

    /// Proactively replenish batches that are running low
    pub fn replenish_if_needed(&mut self) {
        let threshold = self.batch_size / 4;
        
        if self.port_batch.len() < threshold { self.replenish_port_batch(); }
        if self.sequence_batch.len() < threshold { self.replenish_sequence_batch(); }
        if self.id_batch.len() < threshold { self.replenish_id_batch(); }
        if self.ttl_batch.len() < threshold { self.replenish_ttl_batch(); }
        if self.window_batch.len() < threshold { self.replenish_window_batch(); }
        if self.flow_batch.len() < threshold { self.replenish_flow_batch(); }
        if self.byte_batch.len() < threshold { self.replenish_byte_batch(); }
    }
}
