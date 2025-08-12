use pnet::datalink::{self, NetworkInterface};

/// Network interface management
pub fn list_network_interfaces() -> Vec<NetworkInterface> {
    datalink::interfaces()
}

pub fn find_interface_by_name(name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == name)
}

pub fn get_default_interface() -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
}
