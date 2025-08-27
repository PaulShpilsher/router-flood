//! Terminal control utilities for clean signal handling
//!
//! This module provides utilities to control terminal behavior,
//! particularly for hiding control character echoes like ^C.

use std::io;
use termios::{Termios, TCSANOW, tcflag_t};

// Terminal control flags
const ECHOCTL: tcflag_t = 0o001000; // Echo control characters as ^X

/// Terminal controller for managing terminal settings
#[derive(Debug)]
pub struct TerminalController {
    original_termios: Option<Termios>,
    stdin_fd: i32,
}

impl Default for TerminalController {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalController {
    /// Create a new terminal controller
    pub fn new() -> Self {
        Self {
            original_termios: None,
            stdin_fd: libc::STDIN_FILENO,
        }
    }

    /// Disable control character echo (like ^C)
    pub fn disable_ctrl_echo(&mut self) -> io::Result<()> {
        // Get current terminal settings
        let mut termios = Termios::from_fd(self.stdin_fd)
            .map_err(|e| io::Error::other(format!("Failed to get terminal settings: {}", e)))?;
        
        // Store original settings for restoration
        self.original_termios = Some(termios);
        
        // Disable control character echo
        termios.c_lflag &= !(ECHOCTL);
        
        // Apply the new settings
        termios::tcsetattr(self.stdin_fd, TCSANOW, &termios)
            .map_err(|e| io::Error::other(format!("Failed to set terminal settings: {}", e)))?;
        
        Ok(())
    }

    /// Restore original terminal settings
    pub fn restore(&mut self) -> io::Result<()> {
        if let Some(original) = &self.original_termios {
            termios::tcsetattr(self.stdin_fd, TCSANOW, original)
                .map_err(|e| io::Error::other(format!("Failed to restore terminal settings: {}", e)))?;
            self.original_termios = None;
        }
        Ok(())
    }

    /// Check if we're running in a TTY (terminal)
    pub fn is_tty() -> bool {
        unsafe { libc::isatty(libc::STDIN_FILENO) != 0 }
    }

    /// Get the stdin file descriptor
    pub fn stdin_fd(&self) -> i32 {
        self.stdin_fd
    }

    /// Check if original termios is stored
    pub fn has_original_termios(&self) -> bool {
        self.original_termios.is_some()
    }
}

impl Drop for TerminalController {
    fn drop(&mut self) {
        // Ensure terminal settings are restored when the controller is dropped
        let _ = self.restore();
    }
}

/// RAII guard for terminal control
pub struct TerminalGuard {
    controller: TerminalController,
}

impl TerminalGuard {
    /// Create a new terminal guard and disable control character echo
    pub fn new() -> io::Result<Self> {
        let mut controller = TerminalController::new();
        
        // Only modify terminal settings if we're in a TTY
        if TerminalController::is_tty() {
            controller.disable_ctrl_echo()?;
        }
        
        Ok(Self { controller })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Restore terminal settings when guard is dropped
        let _ = self.controller.restore();
    }
}
