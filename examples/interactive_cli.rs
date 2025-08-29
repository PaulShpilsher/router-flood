//! Example demonstrating interactive CLI functionality
//!
//! This example shows how the prompt system could be used for
//! interactive configuration and operation.

// Note: CliPrompts is internal, we'll demonstrate the concept
use router_flood::error::Result;

fn main() -> Result<()> {
    println!("=== Interactive CLI Example ===\n");
    
    // Example of how prompts would be used in an interactive mode
    println!("This example demonstrates the CLI prompt system.");
    println!("In a real interactive mode, you would be able to:");
    println!();
    println!("1. Choose attack parameters interactively");
    println!("2. Select protocols from a menu");
    println!("3. Configure safety settings");
    println!("4. Monitor progress in real-time");
    println!();
    
    // Display section headers (demonstration)
    display_section("Configuration");
    println!("  - Target IP: 192.168.1.1");
    println!("  - Ports: [80, 443]");
    println!("  - Protocol: UDP");
    println!();
    
    display_section("Attack Parameters");
    println!("  - Threads: 4");
    println!("  - Packet rate: 1000 pps");
    println!("  - Duration: 60 seconds");
    println!();
    
    // Example of what a choice prompt would look like
    println!("Example choice prompt (not interactive in this demo):");
    println!("Select protocol:");
    println!("  1. UDP");
    println!("  2. TCP SYN");
    println!("  3. ICMP");
    println!("  4. Mixed");
    println!("Enter choice [1]: _");
    println!();
    
    println!("Note: To use interactive mode, the prompt_choice() function");
    println!("would be called to get user input. This is disabled in the");
    println!("example to prevent blocking.");
    
    Ok(())
}

/// Display a section header (demonstration function)
fn display_section(title: &str) {
    println!();
    println!("🎯 {}", title);
    println!("==========================================");
    println!();
}