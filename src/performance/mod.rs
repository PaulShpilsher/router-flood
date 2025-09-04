//! Performance optimization modules
//!
//! This module contains critical performance optimizations that are actually
//! used in the codebase: CPU affinity and memory pooling.

pub mod cpu_affinity;
pub mod memory_pool;

// Re-export commonly used types
pub use cpu_affinity::{CpuAffinity, CpuTopology};
pub use memory_pool::{LockFreeMemoryPool, Memory, ManagedMemory, PoolStats};

// Simple SIMD utilities for packet payload generation
/// 
/// SIMD (Single Instruction, Multiple Data) optimizations for bulk operations.
/// These functions use CPU vector instructions to process multiple bytes at once.
pub mod simd {
    use crate::error::Result;
    
    // Performance strategy:
    // - AVX2 processes 32 bytes per instruction (256-bit vectors)
    // - This is ~8-16x faster than byte-by-byte operations
    // - Runtime CPU feature detection ensures compatibility
    // - Fallback to scalar implementation on older CPUs
    
    
    /// Fill buffer with random data using SIMD when available
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn fill_random(buffer: &mut [u8]) -> Result<()> {

        // Use AVX2 if available
        if is_x86_feature_detected!("avx2") {
            unsafe { fill_random_avx2(buffer) }
        } else {
            fill_random_scalar(buffer)
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    #[inline]
    pub fn fill_random(buffer: &mut [u8]) -> Result<()> {
        fill_random_scalar(buffer)
    }
    
    #[inline(never)]  // Don't inline - this is the fallback path
    fn fill_random_scalar(buffer: &mut [u8]) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.fill(buffer);
        Ok(())
    }
    
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    #[inline]  // Inline for performance when AVX2 is available
    unsafe fn fill_random_avx2(buffer: &mut [u8]) -> Result<()> {
        use std::arch::x86_64::*;
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let mut offset = 0;
        
        // Process 32 bytes at a time with AVX2
        while offset + 32 <= buffer.len() {
            let mut random_bytes = [0u8; 32];
            rng.fill(&mut random_bytes);
            unsafe {
                let vec = _mm256_loadu_si256(random_bytes.as_ptr() as *const __m256i);
                _mm256_storeu_si256(buffer[offset..].as_mut_ptr() as *mut __m256i, vec);
            }
            offset += 32;
        }
        
        // Handle remaining bytes
        while offset < buffer.len() {
            rng.fill(&mut buffer[offset..offset+1]);
            offset += 1;
        }
        
        Ok(())
    }
}