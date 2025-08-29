//! Unit tests for UI progress module
//!
//! These tests were moved from src/ui/progress.rs to maintain
//! separation between implementation and test code.

use router_flood::ui::progress::*;

#[test]
fn test_format_number() {
    assert_eq!(format_number(500), "500");
    assert_eq!(format_number(1_500), "1.5K");
    assert_eq!(format_number(1_500_000), "1.5M");
    assert_eq!(format_number(1_500_000_000), "1.5B");
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(500), "500 bytes");
    assert_eq!(format_bytes(1_536), "1.50 KB");
    assert_eq!(format_bytes(1_572_864), "1.50 MB");
    assert_eq!(format_bytes(1_610_612_736), "1.50 GB");
}

#[tokio::test]
async fn test_progress_indicator() {
    let progress = ProgressIndicator::new("Testing");
    
    // Test completion - should not panic with our improved error handling
    progress.complete_success(Some("Test completed"));
    
    // Should not panic
}