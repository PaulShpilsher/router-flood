//! String interning system for reducing memory allocations
//!
//! This module provides string interning to avoid repeated allocations
//! of common strings used throughout the application.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Interned string that uses reference counting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternedString {
    inner: Arc<str>,
}

impl InternedString {
    /// Create a new interned string
    fn new(s: &str) -> Self {
        Self {
            inner: Arc::from(s),
        }
    }
    
    /// Get the string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }
    
    /// Get the length
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    /// Compare with a string slice
    pub fn eq_str(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for InternedString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

/// String interner for deduplicating strings
pub struct StringInterner {
    strings: RwLock<HashMap<Arc<str>, InternedString>>,
}

impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            strings: RwLock::new(HashMap::new()),
        }
    }
    
    /// Intern a string
    pub fn intern(&self, s: &str) -> InternedString {
        // First try to find existing string with read lock
        {
            let strings = self.strings.read().unwrap();
            if let Some(interned) = strings.get(s) {
                return interned.clone();
            }
        }
        
        // Not found, acquire write lock and insert
        let mut strings = self.strings.write().unwrap();
        
        // Double-check in case another thread inserted it
        if let Some(interned) = strings.get(s) {
            return interned.clone();
        }
        
        let interned = InternedString::new(s);
        strings.insert(Arc::clone(&interned.inner), interned.clone());
        interned
    }
    
    /// Get the number of interned strings
    pub fn len(&self) -> usize {
        self.strings.read().unwrap().len()
    }
    
    /// Check if the interner is empty
    pub fn is_empty(&self) -> bool {
        self.strings.read().unwrap().is_empty()
    }
    
    /// Clear all interned strings
    pub fn clear(&self) {
        self.strings.write().unwrap().clear();
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Global string interner for common protocol names and error messages
pub struct GlobalStringInterner {
    interner: StringInterner,
}

impl GlobalStringInterner {
    /// Create a new global interner with pre-populated common strings
    pub fn new() -> Self {
        let interner = StringInterner::new();
        
        // Pre-populate with common protocol names
        let _ = interner.intern("UDP");
        let _ = interner.intern("TCP");
        let _ = interner.intern("ICMP");
        let _ = interner.intern("IPv6");
        let _ = interner.intern("ARP");
        let _ = interner.intern("TCP-SYN");
        let _ = interner.intern("TCP-ACK");
        let _ = interner.intern("IPv6-UDP");
        let _ = interner.intern("IPv6-TCP");
        let _ = interner.intern("IPv6-ICMP");
        
        // Pre-populate with common error messages
        let _ = interner.intern("Invalid IP address format");
        let _ = interner.intern("Private range required");
        let _ = interner.intern("Thread count must be greater than 0");
        let _ = interner.intern("Packet rate must be greater than 0");
        let _ = interner.intern("Failed to create packet");
        let _ = interner.intern("Failed to send packet");
        let _ = interner.intern("Network interface not found");
        let _ = interner.intern("Permission denied");
        
        // Pre-populate with common field names
        let _ = interner.intern("target");
        let _ = interner.intern("threads");
        let _ = interner.intern("packet_rate");
        let _ = interner.intern("ports");
        let _ = interner.intern("protocol_mix");
        let _ = interner.intern("duration");
        let _ = interner.intern("dry_run");
        let _ = interner.intern("export");
        let _ = interner.intern("format");
        
        Self { interner }
    }
    
    /// Intern a string using the global interner
    pub fn intern(&self, s: &str) -> InternedString {
        self.interner.intern(s)
    }
    
    /// Get statistics about the global interner
    pub fn stats(&self) -> InternerStats {
        InternerStats {
            total_strings: self.interner.len(),
        }
    }
}

impl Default for GlobalStringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about string interning
#[derive(Debug, Clone)]
pub struct InternerStats {
    pub total_strings: usize,
}

/// Global instance of the string interner
static GLOBAL_INTERNER: std::sync::OnceLock<GlobalStringInterner> = std::sync::OnceLock::new();

/// Get the global string interner
pub fn global_interner() -> &'static GlobalStringInterner {
    GLOBAL_INTERNER.get_or_init(GlobalStringInterner::new)
}

/// Convenience function to intern a string globally
pub fn intern(s: &str) -> InternedString {
    global_interner().intern(s)
}

/// Macro for interning string literals at compile time
#[macro_export]
macro_rules! intern_str {
    ($s:literal) => {
        $crate::performance::string_interning::intern($s)
    };
}

/// Common interned strings for protocols
pub mod protocols {
    use super::*;
    
    /// Get interned protocol names
    pub fn udp() -> InternedString { intern("UDP") }
    pub fn tcp() -> InternedString { intern("TCP") }
    pub fn icmp() -> InternedString { intern("ICMP") }
    pub fn ipv6() -> InternedString { intern("IPv6") }
    pub fn arp() -> InternedString { intern("ARP") }
    pub fn tcp_syn() -> InternedString { intern("TCP-SYN") }
    pub fn tcp_ack() -> InternedString { intern("TCP-ACK") }
    pub fn ipv6_udp() -> InternedString { intern("IPv6-UDP") }
    pub fn ipv6_tcp() -> InternedString { intern("IPv6-TCP") }
    pub fn ipv6_icmp() -> InternedString { intern("IPv6-ICMP") }
}

/// Common interned strings for error messages
pub mod errors {
    use super::*;
    
    pub fn invalid_ip_format() -> InternedString { intern("Invalid IP address format") }
    pub fn private_range_required() -> InternedString { intern("Private range required") }
    pub fn thread_count_zero() -> InternedString { intern("Thread count must be greater than 0") }
    pub fn packet_rate_zero() -> InternedString { intern("Packet rate must be greater than 0") }
    pub fn packet_creation_failed() -> InternedString { intern("Failed to create packet") }
    pub fn packet_send_failed() -> InternedString { intern("Failed to send packet") }
    pub fn interface_not_found() -> InternedString { intern("Network interface not found") }
    pub fn permission_denied() -> InternedString { intern("Permission denied") }
}

/// Common interned strings for field names
pub mod fields {
    use super::*;
    
    pub fn target() -> InternedString { intern("target") }
    pub fn threads() -> InternedString { intern("threads") }
    pub fn packet_rate() -> InternedString { intern("packet_rate") }
    pub fn ports() -> InternedString { intern("ports") }
    pub fn protocol_mix() -> InternedString { intern("protocol_mix") }
    pub fn duration() -> InternedString { intern("duration") }
    pub fn dry_run() -> InternedString { intern("dry_run") }
    pub fn export() -> InternedString { intern("export") }
    pub fn format() -> InternedString { intern("format") }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interned_string() {
        let s1 = InternedString::new("hello");
        let s2 = InternedString::new("hello");
        
        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s1.len(), 5);
        assert!(!s1.is_empty());
        assert!(s1.eq_str("hello"));
        assert_eq!(s1, s2);
    }
    
    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new();
        
        let s1 = interner.intern("test");
        let s2 = interner.intern("test");
        let s3 = interner.intern("other");
        
        // Same string should return the same interned instance
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
        
        // Should have 2 unique strings
        assert_eq!(interner.len(), 2);
    }
    
    #[test]
    fn test_global_interner() {
        let s1 = intern("global_test");
        let s2 = intern("global_test");
        
        assert_eq!(s1, s2);
        assert_eq!(s1.as_str(), "global_test");
    }
    
    #[test]
    fn test_protocol_strings() {
        let udp1 = protocols::udp();
        let udp2 = protocols::udp();
        
        assert_eq!(udp1, udp2);
        assert_eq!(udp1.as_str(), "UDP");
    }
    
    #[test]
    fn test_error_strings() {
        let err1 = errors::invalid_ip_format();
        let err2 = errors::invalid_ip_format();
        
        assert_eq!(err1, err2);
        assert_eq!(err1.as_str(), "Invalid IP address format");
    }
    
    #[test]
    fn test_field_strings() {
        let field1 = fields::target();
        let field2 = fields::target();
        
        assert_eq!(field1, field2);
        assert_eq!(field1.as_str(), "target");
    }
    
    #[test]
    fn test_interner_stats() {
        let interner = GlobalStringInterner::new();
        let stats = interner.stats();
        
        // Should have pre-populated strings
        assert!(stats.total_strings > 0);
    }
}