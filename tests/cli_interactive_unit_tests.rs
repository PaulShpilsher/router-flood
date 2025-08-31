//! Unit tests for interactive CLI module
//!
//! These tests were moved from src/cli/interactive.rs to maintain
//! separation between implementation and test code.

use router_flood::cli::enhanced::InteractiveMode;
use router_flood::security::Capabilities;
use router_flood::error::Result;

#[test]
fn test_interactive_cli_creation() -> Result<()> {
    let capability_manager = Capabilities::new()?;
    let _cli = InteractiveMode::new(capability_manager);
    // Should not panic - just test that creation works
    Ok(())
}

#[test]
fn test_interactive_config() -> Result<()> {
    use router_flood::cli::enhanced::InteractiveConfig;
    
    // Test InteractiveConfig creation with sample values
    let config = InteractiveConfig {
        target_ip: "192.168.1.1".to_string(),
        ports: "80,443".to_string(),
        threads: "4".to_string(),
        rate: "100".to_string(),
        duration: "30".to_string(),
        dry_run: true,
        cpu_affinity: false,
        export_format: None,
    };
    
    // Basic validation
    assert_eq!(config.target_ip, "192.168.1.1");
    assert_eq!(config.ports, "80,443");
    assert!(config.dry_run);
    
    Ok(())
}

#[tokio::test]
async fn test_interactive_mode_creation() -> Result<()> {
    let capability_manager = Capabilities::new()?;
    let cli = InteractiveMode::new(capability_manager);
    
    // Just ensure creation works - actual run() method would need user input
    // so we can't test it easily
    assert!(std::mem::size_of_val(&cli) > 0);
    
    Ok(())
}