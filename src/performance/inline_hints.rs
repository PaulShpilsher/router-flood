//! Inline hints for performance-critical functions
//!
//! This module provides optimized versions of hot-path functions with
//! appropriate inline hints for better performance.

/// Inline hint for very hot functions (called millions of times)
pub const INLINE_ALWAYS: &str = "always";

/// Inline hint for hot functions (called thousands of times)  
pub const INLINE_HOT: &str = "hot";

/// Macro to add inline hints based on usage frequency
/// - inline_hot: For frequently called functions
/// - inline_cold: For rarely called functions
#[allow(unused_macros)]
macro_rules! inline_hot {
    () => {
        #[inline(always)]
    };
}

#[allow(unused_macros)]
macro_rules! inline_always {
    () => {
        #[inline(always)]
    };
}

#[allow(unused_imports)]
pub(crate) use inline_hot;
#[allow(unused_imports)]
pub(crate) use inline_always;