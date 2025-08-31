//! Basic usage examples for Router Flood

#![allow(clippy::uninlined_format_args)]
//!
//! This example demonstrates the basic functionality of Router Flood
//! in a safe, educational context.

use router_flood::{
    config::{ConfigBuilder, ConfigTemplates},
    security::Capabilities,
    performance::CpuAffinity,
    error::Result,
};
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Router Flood - Basic Usage Examples");
    println!("======================================");

    // Example 1: Security Context Check
    security_context_example()?;

    // Example 2: Configuration Building
    configuration_example()?;

    // Example 3: Performance Analysis
    performance_example()?;

    // Example 4: Template Usage
    template_example()?;

    println!("\n✅ All examples completed successfully!");
    Ok(())
}

/// Example 1: Check security context and capabilities
fn security_context_example() -> Result<()> {
    println!("\n🔒 Example 1: Security Context Analysis");
    println!("---------------------------------------");

    let capability_manager = Capabilities::new()?;
    let context = capability_manager.security_context();

    println!("Process ID: {}", context.process_id);
    println!("Effective UID: {}", context.effective_uid);
    println!("Real UID: {}", context.real_uid);
    println!("Capabilities Available: {}", context.capabilities_available);
    println!("CAP_NET_RAW: {}", if context.has_net_raw { "✅" } else { "❌" });
    println!("CAP_NET_ADMIN: {}", if context.has_net_admin { "✅" } else { "❌" });

    // Check if we can run in dry-run mode (always safe)
    match capability_manager.has_required_capabilities(true) {
        Ok(()) => println!("✅ Dry-run mode: Available"),
        Err(e) => println!("❌ Dry-run mode: {}", e),
    }

    // Check if we can run with actual packets
    match capability_manager.has_required_capabilities(false) {
        Ok(()) => println!("✅ Live mode: Available"),
        Err(e) => println!("❌ Live mode: {}", e),
    }

    Ok(())
}

/// Example 2: Configuration building and validation
fn configuration_example() -> Result<()> {
    println!("\n⚙️  Example 2: Configuration Management");
    println!("-------------------------------------");

    // Method 1: Using ConfigBuilder (fluent API)
    let config1 = ConfigBuilder::new()
        .target_ip("192.168.1.100")
        .target_ports(vec![80, 443, 8080])
        .threads(4)
        .packet_rate(200)
        .duration(Some(60))
        .dry_run(true)
        .build()?;

    println!("✅ Configuration built with fluent API");
    println!("   Target: {}:{:?}", config1.target.ip, config1.target.ports);
    println!("   Threads: {}, Rate: {} PPS", config1.attack.threads, config1.attack.packet_rate);
    println!("   Dry run: {}", config1.safety.dry_run);

    // Method 2: Using templates
    let web_config = ConfigTemplates::template("web_server")
        .ok_or_else(|| router_flood::error::ConfigError::InvalidValue {
            field: "template".to_string(),
            value: "web_server".to_string(),
            reason: "Template not found".to_string(),
        })?;

    println!("\n✅ Web server template loaded");
    println!("   Target: {}:{:?}", web_config.target.ip, web_config.target.ports);
    println!("   Protocol mix: UDP:{:.1}% TCP-SYN:{:.1}% TCP-ACK:{:.1}%",
        web_config.target.protocol_mix.udp_ratio * 100.0,
        web_config.target.protocol_mix.tcp_syn_ratio * 100.0,
        web_config.target.protocol_mix.tcp_ack_ratio * 100.0
    );

    // Method 3: Validate configuration
    router_flood::config::ConfigSchema::validate(&config1)?;
    println!("✅ Configuration validation passed");

    Ok(())
}

/// Example 3: Performance analysis and CPU affinity
fn performance_example() -> Result<()> {
    println!("\n⚡ Example 3: Performance Analysis");
    println!("----------------------------------");

    let cpu_manager = CpuAffinity::new()?;
    let topology = cpu_manager.topology();

    println!("CPU Topology:");
    println!("  Total CPUs: {}", topology.total_cpus);
    println!("  NUMA Nodes: {}", topology.numa_nodes.len());
    println!("  Hyperthreading: {}", if topology.hyperthreading_enabled { "Enabled" } else { "Disabled" });

    // Analyze performance for different worker counts
    for workers in [1, 2, 4, 8] {
        if workers <= topology.total_cpus {
            let recommendations = cpu_manager.get_performance_recommendations(workers);
            println!("\n{} Workers:", workers);
            if recommendations.is_empty() {
                println!("  ✅ Optimal configuration");
            } else {
                for rec in recommendations {
                    println!("  💡 {}", rec);
                }
            }
        }
    }

    // Show CPU assignments for 4 workers
    let mut cpu_manager_mut = CpuAffinity::new()?;
    let assignments = cpu_manager_mut.assign_workers(4)?;
    
    println!("\nProposed CPU Assignments (4 workers):");
    for assignment in assignments {
        println!("  Worker {} → CPU {} (NUMA Node {})", 
            assignment.worker_id, assignment.cpu_id, assignment.numa_node);
    }

    Ok(())
}

/// Example 4: Template usage and customization
fn template_example() -> Result<()> {
    println!("\n📋 Example 4: Configuration Templates");
    println!("------------------------------------");

    // List all available templates
    let templates = ConfigTemplates::list_templates();
    println!("Available templates:");
    for template_name in &templates {
        println!("  • {}", template_name);
    }

    // Load and customize each template
    for template_name in templates {
        if let Some(mut template) = ConfigTemplates::template(template_name) {
            println!("\n📝 Template: {}", template_name);
            
            // Customize for safe demonstration
            template.safety.dry_run = true;
            template.attack.duration = Some(10); // Short duration
            template.attack.packet_rate = 50; // Low rate
            
            // Validate the customized template
            match router_flood::config::ConfigSchema::validate(&template) {
                Ok(()) => {
                    println!("  ✅ Valid configuration");
                    println!("     Target: {}:{:?}", template.target.ip, template.target.ports);
                    println!("     Threads: {}, Rate: {} PPS, Duration: {:?}s",
                        template.attack.threads,
                        template.attack.packet_rate,
                        template.attack.duration
                    );
                }
                Err(e) => {
                    println!("  ❌ Invalid configuration: {}", e);
                }
            }
        }
    }

    // Example: Convert template to YAML
    if let Some(template) = ConfigTemplates::template("basic") {
        match ConfigTemplates::template_to_yaml(&template) {
            Ok(yaml) => {
                println!("\n📄 Basic template as YAML:");
                println!("{}", yaml);
            }
            Err(e) => {
                println!("❌ Failed to serialize template: {}", e);
            }
        }
    }

    Ok(())
}

/// Example 5: IP validation demonstration
#[allow(dead_code)]
fn ip_validation_example() -> Result<()> {
    println!("\n🌐 Example 5: IP Address Validation");
    println!("-----------------------------------");

    let test_ips = [
        "192.168.1.1",    // Private - should pass
        "10.0.0.1",       // Private - should pass
        "172.16.0.1",     // Private - should pass
        "8.8.8.8",        // Public - should fail
        "127.0.0.1",      // Loopback - should fail
        "invalid-ip",     // Invalid - should fail
    ];

    for ip_str in &test_ips {
        match ip_str.parse::<IpAddr>() {
            Ok(ip) => {
                match router_flood::security::validation::validate_target_ip(&ip) {
                    Ok(()) => println!("  ✅ {} - Valid private IP", ip_str),
                    Err(e) => println!("  ❌ {} - {}", ip_str, e),
                }
            }
            Err(_) => {
                println!("  ❌ {} - Invalid IP format", ip_str);
            }
        }
    }

    Ok(())
}

/// Example 6: Safe simulation demonstration
#[allow(dead_code)]
async fn simulation_example() -> Result<()> {
    println!("\n🎯 Example 6: Safe Simulation");
    println!("-----------------------------");

    // Create a safe configuration for demonstration
    let config = ConfigBuilder::new()
        .target_ip("192.168.1.1")
        .target_ports(vec![80])
        .threads(1)
        .packet_rate(10)
        .duration(Some(5))
        .dry_run(true) // Safe mode - no actual packets
        .build()?;

    println!("Configuration:");
    println!("  Target: {}:{:?}", config.target.ip, config.target.ports);
    println!("  Mode: {} (safe)", if config.safety.dry_run { "DRY-RUN" } else { "LIVE" });
    println!("  Duration: {:?} seconds", config.attack.duration);

    // In a real application, you would run the simulation here
    // For this example, we just validate the configuration
    router_flood::config::ConfigSchema::validate(&config)?;
    println!("✅ Configuration validated - ready for simulation");

    Ok(())
}