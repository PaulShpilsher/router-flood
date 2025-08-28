//! Simple system abstraction for testability

/// Trait for system operations - allows mocking in tests
pub trait SystemProvider: Send + Sync {
    fn is_root(&self) -> bool;
    fn effective_uid(&self) -> u32;
    fn is_tty(&self) -> bool;
    fn cpu_count(&self) -> usize;
}

/// Default implementation using libc
pub struct DefaultSystemProvider;

impl SystemProvider for DefaultSystemProvider {
    fn is_root(&self) -> bool {
        unsafe { libc::geteuid() == 0 }
    }
    
    fn effective_uid(&self) -> u32 {
        unsafe { libc::geteuid() }
    }
    
    fn is_tty(&self) -> bool {
        unsafe { libc::isatty(libc::STDIN_FILENO) != 0 }
    }
    
    fn cpu_count(&self) -> usize {
        num_cpus::get()
    }
}