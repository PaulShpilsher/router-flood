//! Fuzz testing for configuration parsing
//!
//! This fuzzer tests YAML configuration parsing with malformed inputs
//! to ensure the parser handles all edge cases gracefully.

#![no_main]

use libfuzzer_sys::fuzz_target;
use router_flood::config::*;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, handling invalid UTF-8 gracefully
    let yaml_str = String::from_utf8_lossy(data);
    
    // Test YAML parsing - should never panic
    let _ = serde_yaml::from_str::<Config>(&yaml_str);
    
    // Test with common YAML prefixes to increase coverage
    let prefixes = [
        "target:\n",
        "attack:\n",
        "safety:\n",
        "monitoring:\n",
        "export:\n",
        "---\n",
        "# Comment\n",
    ];
    
    for prefix in &prefixes {
        let prefixed_yaml = format!("{}{}", prefix, yaml_str);
        let _ = serde_yaml::from_str::<Config>(&prefixed_yaml);
    }
    
    // Test configuration validation if parsing succeeds
    if let Ok(config) = serde_yaml::from_str::<Config>(&yaml_str) {
        let _ = ConfigSchema::validate(&config);
    }
    
    // Test with common configuration patterns
    let patterns = [
        format!("target:\n  ip: \"{}\"\n", yaml_str.chars().take(15).collect::<String>()),
        format!("attack:\n  threads: {}\n", yaml_str.len() % 1000),
        format!("safety:\n  dry_run: {}\n", yaml_str.len() % 2 == 0),
    ];
    
    for pattern in &patterns {
        let _ = serde_yaml::from_str::<Config>(pattern);
    }
});