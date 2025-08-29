//! Comprehensive tests for abstraction layers

use router_flood::abstractions::{NetworkProvider, SystemProvider};
use pnet::datalink::NetworkInterface;
use pnet::ipnetwork::IpNetwork;
use std::net::{Ipv4Addr, Ipv6Addr};

/// Complete mock network provider for testing
struct TestNetworkProvider {
    interfaces: Vec<NetworkInterface>,
    find_by_name_called: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl TestNetworkProvider {
    fn new() -> Self {
        use std::sync::Arc;
        use std::sync::atomic::AtomicUsize;
        
        let interfaces = vec![
            NetworkInterface {
                name: "lo".to_string(),
                description: "Loopback".to_string(),
                index: 1,
                mac: None,
                ips: vec![
                    IpNetwork::V4(Ipv4Addr::new(127, 0, 0, 1).into()),
                    IpNetwork::V6(Ipv6Addr::LOCALHOST.into()),
                ],
                flags: 0,
            },
            NetworkInterface {
                name: "eth0".to_string(),
                description: "Ethernet".to_string(),
                index: 2,
                mac: None,
                ips: vec![
                    IpNetwork::V4(Ipv4Addr::new(192, 168, 1, 100).into()),
                    IpNetwork::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1).into()),
                ],
                flags: 0x1043, // IFF_UP | IFF_BROADCAST | IFF_RUNNING | IFF_MULTICAST
            },
            NetworkInterface {
                name: "wlan0".to_string(),
                description: "Wireless".to_string(),
                index: 3,
                mac: None,
                ips: vec![
                    IpNetwork::V4(Ipv4Addr::new(192, 168, 1, 101).into()),
                ],
                flags: 0,
            },
        ];
        
        Self {
            interfaces,
            find_by_name_called: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    fn with_empty() -> Self {
        use std::sync::Arc;
        use std::sync::atomic::AtomicUsize;
        
        Self {
            interfaces: vec![],
            find_by_name_called: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl NetworkProvider for TestNetworkProvider {
    fn interfaces(&self) -> Vec<NetworkInterface> {
        self.interfaces.clone()
    }
    
    fn find_by_name(&self, name: &str) -> Option<NetworkInterface> {
        use std::sync::atomic::Ordering;
        self.find_by_name_called.fetch_add(1, Ordering::Relaxed);
        self.interfaces.iter().find(|i| i.name == name).cloned()
    }
    
    fn default_interface(&self) -> Option<NetworkInterface> {
        self.interfaces.iter()
            .find(|i| i.is_up() && !i.is_loopback() && !i.ips.is_empty())
            .cloned()
    }
}

/// Complete mock system provider for testing
struct TestSystemProvider {
    is_root: bool,
    uid: u32,
    is_tty: bool,
    cpu_count: usize,
}

impl TestSystemProvider {
    fn new_root() -> Self {
        Self {
            is_root: true,
            uid: 0,
            is_tty: true,
            cpu_count: 8,
        }
    }
    
    fn new_user(uid: u32) -> Self {
        Self {
            is_root: false,
            uid,
            is_tty: false,
            cpu_count: 4,
        }
    }
}

impl SystemProvider for TestSystemProvider {
    fn is_root(&self) -> bool {
        self.is_root
    }
    
    fn effective_uid(&self) -> u32 {
        self.uid
    }
    
    fn is_tty(&self) -> bool {
        self.is_tty
    }
    
    fn cpu_count(&self) -> usize {
        self.cpu_count
    }
}

#[test]
fn test_network_provider_interfaces_list() {
    let provider = TestNetworkProvider::new();
    let interfaces = provider.interfaces();
    
    assert_eq!(interfaces.len(), 3);
    assert_eq!(interfaces[0].name, "lo");
    assert_eq!(interfaces[1].name, "eth0");
    assert_eq!(interfaces[2].name, "wlan0");
}

#[test]
fn test_network_provider_find_by_name() {
    let provider = TestNetworkProvider::new();
    
    // Find existing interface
    let eth0 = provider.find_by_name("eth0");
    assert!(eth0.is_some());
    assert_eq!(eth0.unwrap().name, "eth0");
    
    // Find non-existing interface
    let eth1 = provider.find_by_name("eth1");
    assert!(eth1.is_none());
    
    // Verify method was called
    use std::sync::atomic::Ordering;
    assert_eq!(provider.find_by_name_called.load(Ordering::Relaxed), 2);
}

#[test]
fn test_network_provider_empty_list() {
    let provider = TestNetworkProvider::with_empty();
    
    assert!(provider.interfaces().is_empty());
    assert!(provider.find_by_name("any").is_none());
    assert!(provider.default_interface().is_none());
}

#[test]
fn test_system_provider_root_detection() {
    let root = TestSystemProvider::new_root();
    assert!(root.is_root());
    assert_eq!(root.effective_uid(), 0);
    
    let user = TestSystemProvider::new_user(1000);
    assert!(!user.is_root());
    assert_eq!(user.effective_uid(), 1000);
}

#[test]
fn test_system_provider_environment_info() {
    let provider = TestSystemProvider::new_root();
    assert!(provider.is_tty());
    assert_eq!(provider.cpu_count(), 8);
    
    let user_provider = TestSystemProvider::new_user(1001);
    assert!(!user_provider.is_tty());
    assert_eq!(user_provider.cpu_count(), 4);
}

#[test]
fn test_privilege_checking_with_mock() {
    fn check_privileges<S: SystemProvider>(system: &S) -> Result<(), String> {
        if system.is_root() || system.effective_uid() == 0 {
            Ok(())
        } else {
            Err(format!("Requires root privileges (current uid: {})", system.effective_uid()))
        }
    }
    
    let root = TestSystemProvider::new_root();
    assert!(check_privileges(&root).is_ok());
    
    let user = TestSystemProvider::new_user(1000);
    assert!(check_privileges(&user).is_err());
}

#[test]
fn test_network_selection_with_mock() {
    fn select_network_interface<N: NetworkProvider>(
        network: &N,
        preferred: Option<&str>
    ) -> Option<NetworkInterface> {
        if let Some(name) = preferred {
            network.find_by_name(name)
        } else {
            network.default_interface()
        }
    }
    
    let provider = TestNetworkProvider::new();
    
    // Select specific interface
    let eth0 = select_network_interface(&provider, Some("eth0"));
    assert!(eth0.is_some());
    assert_eq!(eth0.unwrap().name, "eth0");
    
    // Select default interface (would be eth0 as first non-loopback)
    let default = select_network_interface(&provider, None);
    assert!(default.is_some());
}

#[test]
fn test_cpu_affinity_planning_with_mock() {
    fn plan_worker_distribution<S: SystemProvider>(
        system: &S,
        num_workers: usize
    ) -> Vec<usize> {
        let cpu_count = system.cpu_count();
        (0..num_workers)
            .map(|i| i % cpu_count)
            .collect()
    }
    
    let system = TestSystemProvider::new_root();
    let distribution = plan_worker_distribution(&system, 10);
    
    assert_eq!(distribution.len(), 10);
    assert_eq!(distribution[0], 0);
    assert_eq!(distribution[7], 7);
    assert_eq!(distribution[8], 0); // Wraps around
    assert_eq!(distribution[9], 1);
}

#[test]
fn test_combined_abstraction_usage() {
    fn setup_environment<N: NetworkProvider, S: SystemProvider>(
        network: &N,
        system: &S,
        interface_name: Option<&str>
    ) -> Result<(bool, Option<String>), String> {
        // Check privileges
        let has_privileges = system.is_root();
        
        // Select network interface
        let interface = if let Some(name) = interface_name {
            network.find_by_name(name)
        } else {
            network.default_interface()
        };
        
        let interface_name = interface.map(|i| i.name);
        
        Ok((has_privileges, interface_name))
    }
    
    let network = TestNetworkProvider::new();
    let system = TestSystemProvider::new_root();
    
    let result = setup_environment(&network, &system, Some("eth0"));
    assert!(result.is_ok());
    
    let (has_priv, iface) = result.unwrap();
    assert!(has_priv);
    assert_eq!(iface, Some("eth0".to_string()));
}

// Extension trait test
trait NetworkProviderExt: NetworkProvider {
    fn count_ipv4_interfaces(&self) -> usize {
        self.interfaces()
            .iter()
            .filter(|i| i.ips.iter().any(|ip| ip.is_ipv4()))
            .count()
    }
}

impl<T: NetworkProvider> NetworkProviderExt for T {}

#[test]
fn test_network_provider_extension() {
    let provider = TestNetworkProvider::new();
    assert_eq!(provider.count_ipv4_interfaces(), 3);
}