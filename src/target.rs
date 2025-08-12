use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Multi-port target manager
pub struct MultiPortTarget {
    ports: Vec<u16>,
    current_index: Arc<AtomicUsize>,
}

impl MultiPortTarget {
    pub fn new(ports: Vec<u16>) -> Self {
        Self {
            ports,
            current_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn next_port(&self) -> u16 {
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.ports.len();
        self.ports[index]
    }

    pub fn get_ports(&self) -> &[u16] {
        &self.ports
    }
}
