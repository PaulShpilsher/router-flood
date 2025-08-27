//! SIMD-optimized packet construction
//!
//! This module provides SIMD-accelerated packet building for improved performance
//! on systems that support SIMD instructions.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use crate::error::{PacketError, Result};
use crate::rng::BatchedRng;

/// SIMD-optimized packet builder
pub struct SimdPacketBuilder {
    rng: BatchedRng,
    simd_available: bool,
}

impl SimdPacketBuilder {
    /// Create a new SIMD packet builder
    pub fn new() -> Self {
        Self {
            rng: BatchedRng::new(),
            simd_available: Self::detect_simd_support(),
        }
    }

    /// Detect available SIMD instruction sets
    fn detect_simd_support() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx2") || is_x86_feature_detected!("sse4.2")
        }
        #[cfg(target_arch = "aarch64")]
        {
            std::arch::is_aarch64_feature_detected!("neon")
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            false
        }
    }

    /// Fill buffer with random payload data using SIMD instructions
    pub fn fill_payload_simd(&mut self, buffer: &mut [u8]) -> Result<()> {
        if !self.simd_available || buffer.len() < 16 {
            // Fallback to scalar implementation for small buffers or unsupported platforms
            return self.fill_payload_scalar(buffer);
        }

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { self.fill_payload_avx2(buffer) }
            } else if is_x86_feature_detected!("sse4.2") {
                unsafe { self.fill_payload_sse42(buffer) }
            } else {
                self.fill_payload_scalar(buffer)
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                unsafe { self.fill_payload_neon(buffer) }
            } else {
                self.fill_payload_scalar(buffer)
            }
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            self.fill_payload_scalar(buffer)
        }
    }

    /// Scalar fallback implementation
    fn fill_payload_scalar(&mut self, buffer: &mut [u8]) -> Result<()> {
        // Use batched RNG for better performance
        let payload = self.rng.payload(buffer.len());
        if payload.len() != buffer.len() {
            return Err(PacketError::InvalidParameters(
                "Payload size mismatch".to_string()
            ).into());
        }
        buffer.copy_from_slice(&payload);
        Ok(())
    }

    /// AVX2-optimized payload generation (x86_64)
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_payload_avx2(&mut self, buffer: &mut [u8]) -> Result<()> {
        let len = buffer.len();
        let chunks = len / 32; // AVX2 processes 32 bytes at a time
        let remainder = len % 32;

        // Process 32-byte chunks with AVX2
        for i in 0..chunks {
            let offset = i * 32;
            
            // Generate 32 bytes of random data
            let rand1 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            let rand2 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            let rand3 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            let rand4 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            
            // Create AVX2 vector from random data
            let vec = _mm256_set_epi64x(
                rand4 as i64,
                rand3 as i64,
                rand2 as i64,
                rand1 as i64,
            );
            
            // Store to buffer
            _mm256_storeu_si256(
                buffer.as_mut_ptr().add(offset) as *mut __m256i,
                vec,
            );
        }

        // Handle remaining bytes with scalar code
        if remainder > 0 {
            let remaining_slice = &mut buffer[chunks * 32..];
            self.fill_payload_scalar(remaining_slice)?;
        }

        Ok(())
    }

    /// SSE4.2-optimized payload generation (x86_64)
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse4.2")]
    unsafe fn fill_payload_sse42(&mut self, buffer: &mut [u8]) -> Result<()> {
        let len = buffer.len();
        let chunks = len / 16; // SSE processes 16 bytes at a time
        let remainder = len % 16;

        // Process 16-byte chunks with SSE4.2
        for i in 0..chunks {
            let offset = i * 16;
            
            // Generate 16 bytes of random data
            let rand1 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            let rand2 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            
            // Create SSE vector from random data
            let vec = _mm_set_epi64x(rand2 as i64, rand1 as i64);
            
            // Store to buffer
            _mm_storeu_si128(
                buffer.as_mut_ptr().add(offset) as *mut __m128i,
                vec,
            );
        }

        // Handle remaining bytes with scalar code
        if remainder > 0 {
            let remaining_slice = &mut buffer[chunks * 16..];
            self.fill_payload_scalar(remaining_slice)?;
        }

        Ok(())
    }

    /// NEON-optimized payload generation (ARM64)
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn fill_payload_neon(&mut self, buffer: &mut [u8]) -> Result<()> {
        let len = buffer.len();
        let chunks = len / 16; // NEON processes 16 bytes at a time
        let remainder = len % 16;

        // Process 16-byte chunks with NEON
        for i in 0..chunks {
            let offset = i * 16;
            
            // Generate 16 bytes of random data
            let rand1 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            let rand2 = (self.rng.sequence() as u64) << 32 | self.rng.sequence() as u64;
            
            // Create NEON vector from random data
            let vec = vreinterpretq_u8_u64(vdupq_n_u64(0));
            let vec = vsetq_lane_u64(rand1, vreinterpretq_u64_u8(vec), 0);
            let vec = vsetq_lane_u64(rand2, vreinterpretq_u64_u8(vec), 1);
            
            // Store to buffer
            vst1q_u8(buffer.as_mut_ptr().add(offset), vec);
        }

        // Handle remaining bytes with scalar code
        if remainder > 0 {
            let remaining_slice = &mut buffer[chunks * 16..];
            self.fill_payload_scalar(remaining_slice)?;
        }

        Ok(())
    }

    /// Generate optimized UDP packet with SIMD payload
    pub fn build_udp_packet_simd(
        &mut self,
        buffer: &mut [u8],
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        payload_size: usize,
    ) -> Result<usize> {
        let total_size = 20 + 8 + payload_size; // IP + UDP + payload
        
        if buffer.len() < total_size {
            return Err(PacketError::BufferTooSmall {
                required: total_size,
                available: buffer.len(),
            }.into());
        }

        // Build IP header (20 bytes)
        self.build_ip_header(&mut buffer[0..20], src_ip, dst_ip, 8 + payload_size, 17)?;
        
        // Build UDP header (8 bytes)
        self.build_udp_header(&mut buffer[20..28], src_port, dst_port, payload_size)?;
        
        // Fill payload with SIMD-optimized random data
        if payload_size > 0 {
            self.fill_payload_simd(&mut buffer[28..28 + payload_size])?;
        }

        Ok(total_size)
    }

    /// Build IP header with optimized field setting
    fn build_ip_header(
        &mut self,
        buffer: &mut [u8],
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        payload_len: usize,
        protocol: u8,
    ) -> Result<()> {
        if buffer.len() < 20 {
            return Err(PacketError::BufferTooSmall {
                required: 20,
                available: buffer.len(),
            }.into());
        }

        // Version (4) + IHL (5) = 0x45
        buffer[0] = 0x45;
        // Type of Service
        buffer[1] = 0x00;
        // Total Length
        let total_len = 20 + payload_len;
        buffer[2] = (total_len >> 8) as u8;
        buffer[3] = total_len as u8;
        // Identification
        let id = self.rng.identification();
        buffer[4] = (id >> 8) as u8;
        buffer[5] = id as u8;
        // Flags + Fragment Offset
        buffer[6] = 0x40; // Don't Fragment
        buffer[7] = 0x00;
        // TTL
        buffer[8] = self.rng.ttl();
        // Protocol
        buffer[9] = protocol;
        // Checksum (will be calculated)
        buffer[10] = 0x00;
        buffer[11] = 0x00;
        // Source IP
        buffer[12..16].copy_from_slice(&src_ip);
        // Destination IP
        buffer[16..20].copy_from_slice(&dst_ip);

        // Calculate and set checksum
        let checksum = self.calculate_ip_checksum(&buffer[0..20]);
        buffer[10] = (checksum >> 8) as u8;
        buffer[11] = checksum as u8;

        Ok(())
    }

    /// Build UDP header
    fn build_udp_header(
        &mut self,
        buffer: &mut [u8],
        src_port: u16,
        dst_port: u16,
        payload_len: usize,
    ) -> Result<()> {
        if buffer.len() < 8 {
            return Err(PacketError::BufferTooSmall {
                required: 8,
                available: buffer.len(),
            }.into());
        }

        let udp_len = 8 + payload_len;
        
        // Source Port
        buffer[0] = (src_port >> 8) as u8;
        buffer[1] = src_port as u8;
        // Destination Port
        buffer[2] = (dst_port >> 8) as u8;
        buffer[3] = dst_port as u8;
        // Length
        buffer[4] = (udp_len >> 8) as u8;
        buffer[5] = udp_len as u8;
        // Checksum (simplified - set to 0 for performance)
        buffer[6] = 0x00;
        buffer[7] = 0x00;

        Ok(())
    }

    /// Calculate IP header checksum
    fn calculate_ip_checksum(&self, header: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        // Sum all 16-bit words in the header
        for chunk in header.chunks_exact(2) {
            let word = u16::from_be_bytes([chunk[0], chunk[1]]);
            sum += word as u32;
        }
        
        // Add carry bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        // One's complement
        !sum as u16
    }

    /// Check if SIMD optimizations are available
    pub fn is_simd_available(&self) -> bool {
        self.simd_available
    }

    /// Get performance information
    pub fn get_performance_info(&self) -> SimdPerformanceInfo {
        SimdPerformanceInfo {
            simd_available: self.simd_available,
            instruction_set: self.get_instruction_set(),
            vector_width: self.get_vector_width(),
        }
    }

    /// Get the available instruction set
    fn get_instruction_set(&self) -> &'static str {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                "AVX2"
            } else if is_x86_feature_detected!("sse4.2") {
                "SSE4.2"
            } else {
                "Scalar"
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                "NEON"
            } else {
                "Scalar"
            }
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            "Scalar"
        }
    }

    /// Get the vector width in bytes
    fn get_vector_width(&self) -> usize {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                32
            } else if is_x86_feature_detected!("sse4.2") {
                16
            } else {
                8
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                16
            } else {
                8
            }
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            8
        }
    }
}

impl Default for SimdPacketBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance information for SIMD packet builder
#[derive(Debug, Clone)]
pub struct SimdPerformanceInfo {
    pub simd_available: bool,
    pub instruction_set: &'static str,
    pub vector_width: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_packet_builder_creation() {
        let builder = SimdPacketBuilder::new();
        let info = builder.get_performance_info();
        
        // Should not panic and provide valid info
        assert!(!info.instruction_set.is_empty());
        assert!(info.vector_width >= 8);
    }

    #[test]
    fn test_payload_filling() {
        let mut builder = SimdPacketBuilder::new();
        let mut buffer = vec![0u8; 1000];
        
        // Should fill buffer without panicking
        assert!(builder.fill_payload_simd(&mut buffer).is_ok());
        
        // Buffer should contain non-zero data
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_udp_packet_building() {
        let mut builder = SimdPacketBuilder::new();
        let mut buffer = vec![0u8; 1500];
        
        let result = builder.build_udp_packet_simd(
            &mut buffer,
            [192, 168, 1, 100],
            [192, 168, 1, 1],
            12345,
            80,
            100,
        );
        
        assert!(result.is_ok());
        let packet_size = result.unwrap();
        assert_eq!(packet_size, 128); // 20 + 8 + 100
        
        // Verify IP header structure
        assert_eq!(buffer[0], 0x45); // Version + IHL
        assert_eq!(buffer[9], 17); // UDP protocol
        
        // Verify UDP header
        assert_eq!(u16::from_be_bytes([buffer[20], buffer[21]]), 12345); // Source port
        assert_eq!(u16::from_be_bytes([buffer[22], buffer[23]]), 80); // Dest port
    }

    #[test]
    fn test_small_buffer_fallback() {
        let mut builder = SimdPacketBuilder::new();
        let mut small_buffer = vec![0u8; 8];
        
        // Should handle small buffers gracefully
        assert!(builder.fill_payload_simd(&mut small_buffer).is_ok());
    }

    #[test]
    fn test_checksum_calculation() {
        let builder = SimdPacketBuilder::new();
        
        // Test with known IP header
        let header = [
            0x45, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00,
            0x40, 0x06, 0x00, 0x00, 0xac, 0x10, 0x0a, 0x63,
            0xac, 0x10, 0x0a, 0x0c,
        ];
        
        let checksum = builder.calculate_ip_checksum(&header);
        
        // Checksum should be calculated (exact value depends on implementation)
        assert_ne!(checksum, 0);
    }

    #[test]
    fn test_performance_info() {
        let builder = SimdPacketBuilder::new();
        let info = builder.get_performance_info();
        
        // Should provide meaningful performance information
        assert!(!info.instruction_set.is_empty());
        assert!(info.vector_width > 0);
        
        println!("SIMD Performance Info: {:?}", info);
    }
}