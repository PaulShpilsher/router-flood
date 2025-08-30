//! Unit tests for interactive CLI module
//!
//! These tests were moved from src/cli/interactive.rs to maintain
//! separation between implementation and test code.

use router_flood::cli::enhanced::{InteractiveCli};

#[test]
fn test_interactive_cli_creation() {
    let _cli = InteractiveCli::default(); // Using new InteractiveCli type
    // Should not panic - just test that creation works
    // Note: Cannot access private fields, so just ensure no panic
}

#[test]
fn test_command_building() {
    let cmd = InteractiveCli::build_command(); // Using new InteractiveCli type
    
    // Should have subcommands
    let subcommands: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();
    assert!(subcommands.contains(&"run"));
    assert!(subcommands.contains(&"config"));
    assert!(subcommands.contains(&"system"));
    assert!(subcommands.contains(&"interactive"));
}

#[tokio::test]
async fn test_list_templates() {
    let cli = InteractiveCli::default();
    let result = cli.handle_list_templates().await;
    assert!(result.is_ok());
}