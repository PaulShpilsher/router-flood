//! User interface components and utilities
//!
//! This module provides enhanced user experience components including
//! progress indicators, status displays, and formatted output.

pub mod progress;

pub use progress::{
    ProgressIndicator, 
    StatsDisplay, 
    display_startup_banner, 
    display_completion_summary
};