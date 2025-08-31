//! High-performance memory pool system
//!
//! This module provides optimized memory pools for reducing allocations
//! and improving cache locality in hot paths.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;

/// Branch prediction hint for unlikely conditions
#[inline(always)]
fn unlikely(b: bool) -> bool {
    #[cold]
    fn cold() {}
    
    if b {
        cold();
    }
    b
}

/// Memory block in the pool
struct MemoryBlock {
    data: NonNull<u8>,
    size: usize,
    next: Option<NonNull<MemoryBlock>>,
}

impl MemoryBlock {
    /// Create a new memory block
    fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, 8).ok()?;
        let data = NonNull::new(unsafe { alloc(layout) })?;
        
        Some(Self {
            data,
            size,
            next: None,
        })
    }
    
    /// Get the data pointer
    fn as_ptr(&self) -> *mut u8 {
        self.data.as_ptr()
    }
    
    // size method removed as it was unused
}

impl Drop for MemoryBlock {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.size, 8);
            dealloc(self.data.as_ptr(), layout);
        }
    }
}

/// Lock-free memory pool using a stack-based free list
pub struct LockFreeMemoryPool {
    free_list: AtomicPtr<MemoryBlock>,
    block_size: usize,
    allocated_count: AtomicUsize,
    max_blocks: usize,
}

impl LockFreeMemoryPool {
    /// Create a new lock-free memory pool
    pub fn new(block_size: usize, initial_blocks: usize, max_blocks: usize) -> Self {
        let pool = Self {
            free_list: AtomicPtr::new(ptr::null_mut()),
            block_size,
            allocated_count: AtomicUsize::new(0),
            max_blocks,
        };
        
        // Pre-allocate initial blocks
        for _ in 0..initial_blocks {
            if let Some(block) = MemoryBlock::new(block_size) {
                pool.add_block_to_free_list(Box::into_raw(Box::new(block)));
            }
        }
        
        pool
    }
    
    /// Allocate a memory block from the pool
    #[inline]
    pub fn allocate(&self) -> Option<PooledMemory<'_>> {
        // Try to get a block from the free list
        loop {
            let head = self.free_list.load(Ordering::Acquire);
            if head.is_null() {
                break;
            }
            
            let block = unsafe { &*head };
            let next = block.next.map_or(ptr::null_mut(), |n| n.as_ptr());
            
            if self.free_list.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                self.allocated_count.fetch_add(1, Ordering::Relaxed);
                return Some(PooledMemory {
                    data: block.as_ptr(),
                    size: block.size,
                    pool: self,
                    block: NonNull::new(head)?,
                });
            }
        }
        
        // No free blocks, try to allocate a new one
        if self.allocated_count.load(Ordering::Relaxed) < self.max_blocks {
            if let Some(block) = MemoryBlock::new(self.block_size) {
                let block_ptr = Box::into_raw(Box::new(block));
                self.allocated_count.fetch_add(1, Ordering::Relaxed);
                return Some(PooledMemory {
                    data: unsafe { (*block_ptr).as_ptr() },
                    size: self.block_size,
                    pool: self,
                    block: NonNull::new(block_ptr)?,
                });
            }
        }
        
        None
    }
    
    /// Add a block to the free list (used during initialization)
    fn add_block_to_free_list(&self, block: *mut MemoryBlock) {
        unsafe {
            loop {
                let head = self.free_list.load(Ordering::Acquire);
                (*block).next = NonNull::new(head);
                
                if self.free_list.compare_exchange_weak(
                    head,
                    block,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    break;
                }
            }
        }
        // Don't decrement allocated_count during initialization
    }
    
    /// Return a block to the pool (performance-optimized with safety)
    #[inline]
    fn return_block(&self, block: *mut MemoryBlock) {
        self.add_block_to_free_list(block);
        
        // Performance-optimized defensive approach:
        // Fast path: assume no underflow (99.99% of cases)
        let old_count = self.allocated_count.fetch_sub(1, Ordering::Relaxed);
        
        // Unlikely branch: handle underflow case
        if unlikely(old_count == 0) {
            // Restore count to prevent negative values
            self.allocated_count.store(0, Ordering::Relaxed);
            self.handle_underflow_error();
        }
        
        // Debug-only additional validation (zero cost in release)
        debug_assert!(old_count > 0, "Memory pool double-free detected");
    }
    
    /// Handle underflow error (cold path)
    #[cold]
    #[inline(never)]
    fn handle_underflow_error(&self) {
        #[cfg(debug_assertions)]
        {
            panic!("Memory pool underflow: possible double-free or corruption detected");
        }
        
        #[cfg(not(debug_assertions))]
        {
            // In release mode, log error but continue execution
            eprintln!("WARNING: Memory pool underflow detected - possible double-free");
            // Could also increment a global error counter for monitoring
        }
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let allocated = self.allocated_count.load(Ordering::Relaxed);
        let mut free_count = 0;
        
        let mut current = self.free_list.load(Ordering::Acquire);
        while !current.is_null() {
            free_count += 1;
            current = unsafe { (*current).next.map_or(ptr::null_mut(), |n| n.as_ptr()) };
        }
        
        PoolStats {
            block_size: self.block_size,
            allocated_blocks: allocated,
            free_blocks: free_count,
            max_blocks: self.max_blocks,
        }
    }
}

impl Drop for LockFreeMemoryPool {
    fn drop(&mut self) {
        // Clean up all blocks in the free list
        let mut current = self.free_list.load(Ordering::Acquire);
        while !current.is_null() {
            unsafe {
                let block = Box::from_raw(current);
                current = block.next.map_or(ptr::null_mut(), |n| n.as_ptr());
            }
        }
    }
}

unsafe impl Send for LockFreeMemoryPool {}
unsafe impl Sync for LockFreeMemoryPool {}

/// Memory allocated from a pool
pub struct PooledMemory<'a> {
    data: *mut u8,
    size: usize,
    pool: &'a LockFreeMemoryPool,
    block: NonNull<MemoryBlock>,
}

impl<'a> PooledMemory<'a> {
    /// Get the data as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.size) }
    }
    
    /// Get the data as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.size) }
    }
    
    /// Get the size
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get the data pointer
    pub fn as_ptr(&self) -> *const u8 {
        self.data
    }
    
    /// Get the mutable data pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data
    }
}

impl<'a> Drop for PooledMemory<'a> {
    fn drop(&mut self) {
        self.pool.return_block(self.block.as_ptr());
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub block_size: usize,
    pub allocated_blocks: usize,
    pub free_blocks: usize,
    pub max_blocks: usize,
}

impl PoolStats {
    /// Calculate utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.max_blocks == 0 {
            0.0
        } else {
            (self.allocated_blocks as f64 / self.max_blocks as f64) * 100.0
        }
    }
    
    /// Calculate hit rate (free blocks / total requests)
    pub fn hit_rate(&self) -> f64 {
        let total_blocks = self.allocated_blocks + self.free_blocks;
        if total_blocks == 0 {
            0.0
        } else {
            (self.free_blocks as f64 / total_blocks as f64) * 100.0
        }
    }
}

/// Multi-size memory pool manager
pub struct MemoryManager {
    pools: Vec<Arc<LockFreeMemoryPool>>,
    size_classes: Vec<usize>,
}

impl MemoryManager {
    /// Create a new memory pool manager with standard size classes
    pub fn new() -> Self {
        let size_classes = vec![
            64,    // Small packets
            128,   // Medium packets
            256,   // Large packets
            512,   // Very large packets
            1024,  // Jumbo packets
            1500,  // MTU-sized packets
            2048,  // Large buffers
            4096,  // Page-sized buffers
        ];
        
        let pools = size_classes.iter().map(|&size| {
            Arc::new(LockFreeMemoryPool::new(size, 10, 100))
        }).collect();
        
        Self {
            pools,
            size_classes,
        }
    }
    
    /// Create with custom size classes
    pub fn with_size_classes(size_classes: Vec<usize>) -> Self {
        let pools = size_classes.iter().map(|&size| {
            Arc::new(LockFreeMemoryPool::new(size, 10, 100))
        }).collect();
        
        Self {
            pools,
            size_classes,
        }
    }
    
    /// Allocate memory of the specified size
    pub fn allocate(&self, size: usize) -> Option<ManagedMemory<'_>> {
        // Find the best-fit size class
        let pool_index = self.find_size_class(size)?;
        let pool = &self.pools[pool_index];
        
        pool.allocate().map(|memory| ManagedMemory {
            inner: MemoryType::Pooled(memory),
        })
    }
    
    /// Find the appropriate size class for the given size
    fn find_size_class(&self, size: usize) -> Option<usize> {
        self.size_classes.iter().position(|&class_size| class_size >= size)
    }
    
    /// Get statistics for all pools
    pub fn stats(&self) -> Vec<PoolStats> {
        self.pools.iter().map(|pool| pool.stats()).collect()
    }
    
    /// Get total memory usage
    pub fn total_memory_usage(&self) -> usize {
        self.pools.iter().zip(&self.size_classes).map(|(pool, &size)| {
            let stats = pool.stats();
            (stats.allocated_blocks + stats.free_blocks) * size
        }).sum()
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory that can be either pooled or heap-allocated
pub struct ManagedMemory<'a> {
    inner: MemoryType<'a>,
}

enum MemoryType<'a> {
    Pooled(PooledMemory<'a>),
    Heap(Vec<u8>),
}

impl<'a> ManagedMemory<'a> {
    /// Create heap-allocated memory
    pub fn heap(size: usize) -> Self {
        Self {
            inner: MemoryType::Heap(vec![0; size]),
        }
    }
    
    /// Get the data as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        match &mut self.inner {
            MemoryType::Pooled(memory) => memory.as_mut_slice(),
            MemoryType::Heap(vec) => vec.as_mut_slice(),
        }
    }
    
    /// Get the data as a slice
    pub fn as_slice(&self) -> &[u8] {
        match &self.inner {
            MemoryType::Pooled(memory) => memory.as_slice(),
            MemoryType::Heap(vec) => vec.as_slice(),
        }
    }
    
    /// Get the size
    pub fn size(&self) -> usize {
        match &self.inner {
            MemoryType::Pooled(memory) => memory.size(),
            MemoryType::Heap(vec) => vec.len(),
        }
    }
    
    /// Check if this is pooled memory
    pub fn is_pooled(&self) -> bool {
        matches!(self.inner, MemoryType::Pooled(_))
    }
}

/// Global memory pool manager
static GLOBAL_POOL_MANAGER: std::sync::OnceLock<MemoryManager> = std::sync::OnceLock::new();

/// Get the global memory pool manager
pub fn global_pool_manager() -> &'static MemoryManager {
    GLOBAL_POOL_MANAGER.get_or_init(MemoryManager::new)
}

/// Allocate memory from the global pool
pub fn allocate(size: usize) -> Option<ManagedMemory<'static>> {
    // This is a bit of a hack to work around lifetime issues
    // In practice, you'd want to use a different approach
    unsafe {
        let manager = global_pool_manager() as *const MemoryManager;
        (*manager).allocate(size).map(|memory| std::mem::transmute(memory))
    }
}

// Tests moved to tests/ directory
