//! Zero-copy packet processing optimizations
//!
//! This module implements zero-copy operations to reduce memory allocations
//! and improve performance in hot paths.

use std::mem::MaybeUninit;
use std::ptr;
use std::slice;

/// Zero-copy buffer for packet construction
pub struct ZeroCopyBuffer {
    data: Box<[MaybeUninit<u8>]>,
    capacity: usize,
    len: usize,
}

impl ZeroCopyBuffer {
    /// Create a new zero-copy buffer with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let data = unsafe {
            let layout = std::alloc::Layout::array::<MaybeUninit<u8>>(capacity).unwrap();
            let ptr = std::alloc::alloc(layout) as *mut MaybeUninit<u8>;
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(slice::from_raw_parts_mut(ptr, capacity))
        };
        
        Self {
            data,
            capacity,
            len: 0,
        }
    }
    
    /// Get the current length of valid data
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Clear the buffer without deallocating
    pub fn clear(&mut self) {
        self.len = 0;
    }
    
    /// Reserve additional capacity if needed
    pub fn reserve(&mut self, additional: usize) {
        if self.len + additional > self.capacity {
            let new_capacity = (self.len + additional).next_power_of_two();
            self.resize(new_capacity);
        }
    }
    
    /// Resize the buffer to the new capacity
    fn resize(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }
        
        let new_data = unsafe {
            let layout = std::alloc::Layout::array::<MaybeUninit<u8>>(new_capacity).unwrap();
            let ptr = std::alloc::alloc(layout) as *mut MaybeUninit<u8>;
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            
            // Copy existing data
            ptr::copy_nonoverlapping(self.data.as_ptr(), ptr, self.len);
            
            Box::from_raw(slice::from_raw_parts_mut(ptr, new_capacity))
        };
        
        self.data = new_data;
        self.capacity = new_capacity;
    }
    
    /// Write data to the buffer without bounds checking
    /// 
    /// # Safety
    /// The caller must ensure that `offset + data.len() <= self.capacity`
    pub unsafe fn write_unchecked(&mut self, offset: usize, data: &[u8]) {
        let dst = self.data.as_mut_ptr().add(offset) as *mut u8;
        ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
        self.len = self.len.max(offset + data.len());
    }
    
    /// Write data to the buffer with bounds checking
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), &'static str> {
        if offset + data.len() > self.capacity {
            return Err("Write would exceed buffer capacity");
        }
        
        unsafe {
            self.write_unchecked(offset, data);
        }
        Ok(())
    }
    
    /// Append data to the end of the buffer
    pub fn push(&mut self, data: &[u8]) -> Result<(), &'static str> {
        self.reserve(data.len());
        unsafe {
            self.write_unchecked(self.len, data);
        }
        Ok(())
    }
    
    /// Write a single byte at the specified offset
    pub fn write_u8(&mut self, offset: usize, value: u8) -> Result<(), &'static str> {
        if offset >= self.capacity {
            return Err("Write would exceed buffer capacity");
        }
        
        unsafe {
            let dst = self.data.as_mut_ptr().add(offset) as *mut u8;
            *dst = value;
            self.len = self.len.max(offset + 1);
        }
        Ok(())
    }
    
    /// Write a u16 in network byte order
    pub fn write_u16_be(&mut self, offset: usize, value: u16) -> Result<(), &'static str> {
        let bytes = value.to_be_bytes();
        self.write(offset, &bytes)
    }
    
    /// Write a u32 in network byte order
    pub fn write_u32_be(&mut self, offset: usize, value: u32) -> Result<(), &'static str> {
        let bytes = value.to_be_bytes();
        self.write(offset, &bytes)
    }
    
    /// Get a slice of the valid data
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const u8, self.len)
        }
    }
    
    /// Get a mutable slice of the valid data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut u8, self.len)
        }
    }
    
    /// Get a slice of the entire buffer capacity
    pub fn as_full_slice(&self) -> &[MaybeUninit<u8>] {
        &self.data
    }
    
    /// Get a mutable slice of the entire buffer capacity
    pub fn as_full_mut_slice(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.data
    }
    
    /// Set the length directly (unsafe)
    /// 
    /// # Safety
    /// The caller must ensure that the first `len` bytes are initialized
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity);
        self.len = len;
    }
}

impl Drop for ZeroCopyBuffer {
    fn drop(&mut self) {
        // The Box will handle deallocation
    }
}

/// Zero-copy string operations
pub struct ZeroCopyStr<'a> {
    data: &'a [u8],
}

impl<'a> ZeroCopyStr<'a> {
    /// Create a new zero-copy string from bytes
    pub fn from_bytes(data: &'a [u8]) -> Self {
        Self { data }
    }
    
    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.data
    }
    
    /// Convert to a string slice if valid UTF-8
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.data)
    }
    
    /// Compare with another string without allocation
    pub fn eq_str(&self, other: &str) -> bool {
        self.data == other.as_bytes()
    }
    
    /// Check if starts with prefix
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.data.starts_with(prefix)
    }
    
    /// Check if ends with suffix
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        self.data.ends_with(suffix)
    }
}

/// Pool of zero-copy buffers for reuse
pub struct ZeroCopyBufferPool {
    buffers: Vec<ZeroCopyBuffer>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl ZeroCopyBufferPool {
    /// Create a new buffer pool
    pub fn new(buffer_size: usize, initial_count: usize, max_pool_size: usize) -> Self {
        let mut buffers = Vec::with_capacity(initial_count);
        for _ in 0..initial_count {
            buffers.push(ZeroCopyBuffer::with_capacity(buffer_size));
        }
        
        Self {
            buffers,
            buffer_size,
            max_pool_size,
        }
    }
    
    /// Get a buffer from the pool or create a new one
    pub fn get(&mut self) -> ZeroCopyBuffer {
        self.buffers.pop().unwrap_or_else(|| {
            ZeroCopyBuffer::with_capacity(self.buffer_size)
        })
    }
    
    /// Return a buffer to the pool
    pub fn put(&mut self, mut buffer: ZeroCopyBuffer) {
        if self.buffers.len() < self.max_pool_size {
            buffer.clear();
            self.buffers.push(buffer);
        }
        // Otherwise, let it drop
    }
    
    /// Get current pool size
    pub fn pool_size(&self) -> usize {
        self.buffers.len()
    }
}

/// Optimized packet header builder using zero-copy operations
pub struct ZeroCopyPacketBuilder {
    pub buffer: ZeroCopyBuffer,
}

impl ZeroCopyPacketBuilder {
    /// Create a new packet builder
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: ZeroCopyBuffer::with_capacity(capacity),
        }
    }
    
    /// Reset the builder for reuse
    pub fn reset(&mut self) {
        self.buffer.clear();
    }
    
    /// Build an Ethernet header
    pub fn ethernet_header(
        &mut self,
        dst_mac: &[u8; 6],
        src_mac: &[u8; 6],
        ethertype: u16,
    ) -> Result<(), &'static str> {
        self.buffer.write(0, dst_mac)?;
        self.buffer.write(6, src_mac)?;
        self.buffer.write_u16_be(12, ethertype)?;
        Ok(())
    }
    
    /// Build an IPv4 header
    pub fn ipv4_header(
        &mut self,
        offset: usize,
        src_ip: u32,
        dst_ip: u32,
        protocol: u8,
        total_length: u16,
    ) -> Result<(), &'static str> {
        // Version (4) + IHL (5) = 0x45
        self.buffer.write_u8(offset, 0x45)?;
        // ToS
        self.buffer.write_u8(offset + 1, 0)?;
        // Total length
        self.buffer.write_u16_be(offset + 2, total_length)?;
        // ID
        self.buffer.write_u16_be(offset + 4, 0)?;
        // Flags + Fragment offset
        self.buffer.write_u16_be(offset + 6, 0x4000)?; // Don't fragment
        // TTL
        self.buffer.write_u8(offset + 8, 64)?;
        // Protocol
        self.buffer.write_u8(offset + 9, protocol)?;
        // Checksum (will be calculated later)
        self.buffer.write_u16_be(offset + 10, 0)?;
        // Source IP
        self.buffer.write_u32_be(offset + 12, src_ip)?;
        // Destination IP
        self.buffer.write_u32_be(offset + 16, dst_ip)?;
        
        Ok(())
    }
    
    /// Build a UDP header
    pub fn udp_header(
        &mut self,
        offset: usize,
        src_port: u16,
        dst_port: u16,
        length: u16,
    ) -> Result<(), &'static str> {
        self.buffer.write_u16_be(offset, src_port)?;
        self.buffer.write_u16_be(offset + 2, dst_port)?;
        self.buffer.write_u16_be(offset + 4, length)?;
        self.buffer.write_u16_be(offset + 6, 0)?; // Checksum
        Ok(())
    }
    
    /// Build a TCP header
    pub fn tcp_header(
        &mut self,
        offset: usize,
        src_port: u16,
        dst_port: u16,
        seq: u32,
        ack: u32,
        flags: u8,
    ) -> Result<(), &'static str> {
        self.buffer.write_u16_be(offset, src_port)?;
        self.buffer.write_u16_be(offset + 2, dst_port)?;
        self.buffer.write_u32_be(offset + 4, seq)?;
        self.buffer.write_u32_be(offset + 8, ack)?;
        // Data offset (5) + reserved (0) = 0x50
        self.buffer.write_u8(offset + 12, 0x50)?;
        // Flags
        self.buffer.write_u8(offset + 13, flags)?;
        // Window size
        self.buffer.write_u16_be(offset + 14, 8192)?;
        // Checksum
        self.buffer.write_u16_be(offset + 16, 0)?;
        // Urgent pointer
        self.buffer.write_u16_be(offset + 18, 0)?;
        Ok(())
    }
    
    /// Get the built packet
    pub fn packet(&self) -> &[u8] {
        self.buffer.as_slice()
    }
    
    /// Get the buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_zero_copy_buffer() {
        let mut buffer = ZeroCopyBuffer::with_capacity(100);
        assert_eq!(buffer.capacity(), 100);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        
        // Test writing data
        let data = b"hello";
        buffer.write(0, data).unwrap();
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_slice(), data);
        
        // Test appending data
        buffer.push(b" world").unwrap();
        assert_eq!(buffer.len(), 11);
        assert_eq!(buffer.as_slice(), b"hello world");
        
        // Test clearing
        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }
    
    #[test]
    fn test_zero_copy_buffer_numeric_writes() {
        let mut buffer = ZeroCopyBuffer::with_capacity(100);
        
        buffer.write_u8(0, 0x42).unwrap();
        buffer.write_u16_be(1, 0x1234).unwrap();
        buffer.write_u32_be(3, 0x56789ABC).unwrap();
        
        let expected = [0x42, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        assert_eq!(buffer.as_slice(), &expected);
    }
    
    #[test]
    fn test_zero_copy_str() {
        let data = b"hello world";
        let zc_str = ZeroCopyStr::from_bytes(data);
        
        assert_eq!(zc_str.len(), 11);
        assert!(!zc_str.is_empty());
        assert_eq!(zc_str.as_bytes(), data);
        assert_eq!(zc_str.as_str().unwrap(), "hello world");
        assert!(zc_str.eq_str("hello world"));
        assert!(zc_str.starts_with(b"hello"));
        assert!(zc_str.ends_with(b"world"));
    }
    
    #[test]
    fn test_buffer_pool() {
        let mut pool = ZeroCopyBufferPool::new(64, 2, 5);
        assert_eq!(pool.pool_size(), 2);
        
        // Get buffers from pool
        let mut buf1 = pool.get();
        let mut buf2 = pool.get();
        let buf3 = pool.get(); // This should create a new one
        
        assert_eq!(pool.pool_size(), 0);
        
        // Use the buffers
        buf1.push(b"test1").unwrap();
        buf2.push(b"test2").unwrap();
        
        // Return buffers to pool
        pool.put(buf1);
        pool.put(buf2);
        pool.put(buf3);
        
        assert_eq!(pool.pool_size(), 3);
    }
    
    #[test]
    fn test_packet_builder() {
        let mut builder = ZeroCopyPacketBuilder::new(1500);
        
        // Build Ethernet + IPv4 + UDP packet
        let dst_mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let src_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        
        builder.ethernet_header(&dst_mac, &src_mac, 0x0800).unwrap();
        builder.ipv4_header(14, 0xC0A80001, 0xC0A80002, 17, 28).unwrap();
        builder.udp_header(34, 12345, 80, 8).unwrap();
        
        assert_eq!(builder.len(), 42);
        
        let packet = builder.packet();
        assert_eq!(&packet[0..6], &dst_mac);
        assert_eq!(&packet[6..12], &src_mac);
        assert_eq!(u16::from_be_bytes([packet[12], packet[13]]), 0x0800);
    }
}