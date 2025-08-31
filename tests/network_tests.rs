//! Network module tests
//!
//! Tests for network interface detection and management.

use router_flood::core::network::*;

#[test]
fn test_list_network_interfaces() {
    let interfaces = list_network_interfaces();
    
    // Should always have at least loopback interface
    assert!(!interfaces.is_empty(), "Should find at least one network interface (loopback)");
    
    // Check that we have a loopback interface
    let has_loopback = interfaces.iter().any(|iface| {
        iface.is_loopback() || iface.name == "lo" || iface.name.starts_with("lo")
    });
    assert!(has_loopback, "Should find loopback interface");
    
    // Verify interface properties
    for iface in &interfaces {
        // Name should not be empty
        assert!(!iface.name.is_empty(), "Interface name should not be empty");
        
        // Should have some basic properties we can check
        // Note: is_up() and is_loopback() are boolean methods that should always work
        let _is_up = iface.is_up();
        let _is_loopback = iface.is_loopback();
    }
}

#[test]
fn test_default_interface() {
    let default_iface = default_interface();
    
    // Depending on the system, we might or might not have a suitable default interface
    // But the function should not panic
    if let Some(iface) = default_iface {
        // If we found a default interface, it should meet our criteria
        assert!(iface.is_up(), "Default interface should be up");
        assert!(!iface.is_loopback(), "Default interface should not be loopback");
        assert!(!iface.ips.is_empty(), "Default interface should have IP addresses");
        assert!(!iface.name.is_empty(), "Default interface should have a name");
    }
}

#[test]
fn test_find_interface_by_name() {
    let interfaces = list_network_interfaces();
    
    if let Some(first_iface) = interfaces.first() {
        let found = find_interface_by_name(&first_iface.name);
        assert!(found.is_some(), "Should find interface by its own name");
        assert_eq!(found.unwrap().name, first_iface.name);
    }
    
    // Test with non-existent interface name
    let not_found = find_interface_by_name("definitely_nonexistent_interface_name_12345");
    assert!(not_found.is_none(), "Should not find non-existent interface");
}

#[test]
fn test_find_interface_by_name_exact_match() {
    // Test that we get exact matches, not partial matches
    let interfaces = list_network_interfaces();
    
    // Find an interface with a short name if possible
    if let Some(iface) = interfaces.iter().find(|i| i.name.len() <= 5) {
        let exact_match = find_interface_by_name(&iface.name);
        assert!(exact_match.is_some());
        assert_eq!(exact_match.unwrap().name, iface.name);
        
        // Test that a partial name doesn't match
        if iface.name.len() > 1 {
            let partial_name = &iface.name[..iface.name.len()-1];
            let partial_match = find_interface_by_name(partial_name);
            // Should either be None or match a different interface with that exact name
            if let Some(matched) = partial_match {
                assert_eq!(matched.name, partial_name);
            }
        }
    }
}

#[test]
fn test_interface_properties() {
    let interfaces = list_network_interfaces();
    
    for iface in interfaces {
        // Test that basic properties can be accessed without panicking
        let _name = &iface.name;
        let _desc = &iface.description;
        let _ips = &iface.ips;
        let _is_up = iface.is_up();
        let _is_loopback = iface.is_loopback();
        
        // Name should never be empty
        assert!(!iface.name.is_empty());
        
        // Description can be empty, but should be a valid string
        // Length is always >= 0 for String, so just verify it's accessible
        let _ = iface.description.len();
        
        // IPs should be a valid vector (can be empty)
        // Length is always >= 0 for Vec, so just verify it's accessible
        let _ = iface.ips.len();
        
        // If interface is up and not loopback, it might have IPs
        if iface.is_up() && !iface.is_loopback() {
            // This is a hint, but not a requirement as some interfaces might be up but not configured
        }
    }
}

#[test]
fn test_loopback_interface_detection() {
    let interfaces = list_network_interfaces();
    
    // Find loopback interfaces
    let loopback_interfaces: Vec<_> = interfaces.iter()
        .filter(|iface| iface.is_loopback())
        .collect();
    
    // Should have at least one loopback interface on most systems
    if !loopback_interfaces.is_empty() {
        for lo_iface in loopback_interfaces {
            assert!(lo_iface.is_loopback());
            // Loopback interfaces often have names like "lo", "lo0", etc.
            let _name_lower = lo_iface.name.to_lowercase();
            // This is a common pattern but not guaranteed across all systems
        }
    }
}

#[test]
fn test_interface_ip_addresses() {
    let interfaces = list_network_interfaces();
    
    for iface in interfaces {
        for ip_network in &iface.ips {
            // Each IP network should be valid
            match ip_network {
                pnet::ipnetwork::IpNetwork::V4(ipv4_net) => {
                    // Test IPv4 network
                    let _ip = ipv4_net.ip();
                    let _prefix = ipv4_net.prefix();
                    assert!(ipv4_net.prefix() <= 32, "IPv4 prefix should be <= 32");
                }
                pnet::ipnetwork::IpNetwork::V6(ipv6_net) => {
                    // Test IPv6 network
                    let _ip = ipv6_net.ip();
                    let _prefix = ipv6_net.prefix();
                    assert!(ipv6_net.prefix() <= 128, "IPv6 prefix should be <= 128");
                }
            }
        }
    }
}

#[test]
fn test_interface_name_uniqueness() {
    let interfaces = list_network_interfaces();
    let mut names = std::collections::HashSet::new();
    
    for iface in interfaces {
        // Interface names should be unique
        assert!(names.insert(iface.name.clone()), 
                "Interface name '{}' should be unique", iface.name);
    }
}

#[test]
fn test_find_suitable_interfaces() {
    let interfaces = list_network_interfaces();
    
    // Find interfaces that would be suitable for our purposes
    let suitable_interfaces: Vec<_> = interfaces.iter()
        .filter(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
        .collect();
    
    // Test that our filtering logic works
    for iface in suitable_interfaces {
        assert!(iface.is_up());
        assert!(!iface.is_loopback());
        assert!(!iface.ips.is_empty());
    }
}

#[test]
fn test_interface_comparison_with_default() {
    let default_iface = default_interface();
    let all_interfaces = list_network_interfaces();
    
    if let Some(default) = default_iface {
        // The default interface should be in our list of all interfaces
        let found_in_list = all_interfaces.iter()
            .any(|iface| iface.name == default.name);
        assert!(found_in_list, "Default interface should be in the complete list");
        
        // Default interface should meet our criteria
        assert!(default.is_up());
        assert!(!default.is_loopback());
        assert!(!default.ips.is_empty());
    }
}