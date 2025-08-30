//! Tests demonstrating the value of abstractions

use router_flood::abstractions::{NetworkProvider, SystemProvider};

/// Mock network provider for testing
struct MockNetworkProvider {
    interfaces: Vec<pnet::datalink::NetworkInterface>,
}

impl NetworkProvider for MockNetworkProvider {
    fn interfaces(&self) -> Vec<pnet::datalink::NetworkInterface> {
        self.interfaces.clone()
    }
    
    fn find_by_name(&self, name: &str) -> Option<pnet::datalink::NetworkInterface> {
        self.interfaces.iter().find(|i| i.name == name).cloned()
    }
    
    fn default_interface(&self) -> Option<pnet::datalink::NetworkInterface> {
        self.interfaces.iter()
            .find(|i| i.is_up() && !i.is_loopback())
            .cloned()
    }
}

/// Mock system provider for testing
struct MockSystemProvider {
    is_root: bool,
    uid: u32,
}

impl SystemProvider for MockSystemProvider {
    fn is_root(&self) -> bool {
        self.is_root
    }
    
    fn effective_uid(&self) -> u32 {
        self.uid
    }
    
    fn is_tty(&self) -> bool {
        false
    }
    
    fn cpu_count(&self) -> usize {
        4
    }
}

#[test]
fn test_network_provider_abstraction() {
    // This test demonstrates how we can mock network interfaces
    // without actual network access
    let mock = MockNetworkProvider {
        interfaces: vec![],
    };
    
    assert!(mock.interfaces().is_empty());
    assert!(mock.default_interface().is_none());
    assert!(mock.find_by_name("eth0").is_none());
}

#[test]
fn test_system_provider_abstraction() {
    // Test with non-root user
    let mock = MockSystemProvider {
        is_root: false,
        uid: 1000,
    };
    
    assert!(!mock.is_root());
    assert_eq!(mock.effective_uid(), 1000);
    assert_eq!(mock.cpu_count(), 4);
    
    // Test with root user
    let root_mock = MockSystemProvider {
        is_root: true,
        uid: 0,
    };
    
    assert!(root_mock.is_root());
    assert_eq!(root_mock.effective_uid(), 0);
}

#[test]
fn test_abstraction_allows_testing_without_privileges() {
    // This test demonstrates that we can test privilege-checking code
    // without actually needing root privileges
    
    fn requires_root<S: SystemProvider>(system: &S) -> Result<(), String> {
        if system.is_root() {
            Ok(())
        } else {
            Err("This operation requires root privileges")
        }
    }
    
    let non_root = MockSystemProvider { is_root: false, uid: 1000 };
    assert!(requires_root(&non_root).is_err());
    
    let root = MockSystemProvider { is_root: true, uid: 0 };
    assert!(requires_root(&root).is_ok());
}