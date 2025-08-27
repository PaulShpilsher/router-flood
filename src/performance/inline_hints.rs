//! Inline hints for performance-critical functions
//!
//! This module provides optimized versions of hot-path functions with
//! appropriate inline hints for better performance.

/// Inline hint for very hot functions (called millions of times)
pub const INLINE_ALWAYS: &str = "always";

/// Inline hint for hot functions (called thousands of times)  
pub const INLINE_HOT: &str = "hot";

/// Macro to add inline hints based on usage frequency
macro_rules! inline_hot {
    () => {
        #[inline]
    };
}

macro_rules! inline_always {
    () => {
        #[inline(always)]
    };
}

pub(crate) use inline_hot;
pub(crate) use inline_always;