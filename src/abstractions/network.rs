//! Simple network abstraction for testability

/// Trait for network operations - allows mocking in tests
pub trait NetworkProvider: Send + Sync {
    fn interfaces(&self) -> Vec<pnet::datalink::NetworkInterface>;
    fn find_by_name(&self, name: &str) -> Option<pnet::datalink::NetworkInterface>;
    fn default_interface(&self) -> Option<pnet::datalink::NetworkInterface>;
}

/// Default implementation using pnet
pub struct PnetProvider;

impl NetworkProvider for PnetProvider {
    fn interfaces(&self) -> Vec<pnet::datalink::NetworkInterface> {
        pnet::datalink::interfaces()
    }
    
    fn find_by_name(&self, name: &str) -> Option<pnet::datalink::NetworkInterface> {
        self.interfaces().into_iter().find(|i| i.name == name)
    }
    
    fn default_interface(&self) -> Option<pnet::datalink::NetworkInterface> {
        self.interfaces()
            .into_iter()
            .find(|i| i.is_up() && !i.is_loopback() && !i.ips.is_empty())
    }
}