//! In-place statistics display with terminal control
//!
//! This module provides functionality to update statistics in place
//! instead of scrolling, creating a cleaner display.

use super::SystemStats;
use crate::constants::stats as stats_constants;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

/// ANSI escape codes for terminal control
mod ansi {
    pub const CLEAR_LINE: &str = "\x1b[2K";     // Clear entire line
    pub const CURSOR_UP: &str = "\x1b[A";       // Move cursor up one line
    pub const HIDE_CURSOR: &str = "\x1b[?25l";  // Hide cursor
    pub const SHOW_CURSOR: &str = "\x1b[?25h";  // Show cursor
    pub const RESET: &str = "\x1b[0m";          // Reset all attributes
    pub const BOLD: &str = "\x1b[1m";           // Bold text
    pub const DIM: &str = "\x1b[2m";            // Dim text
    pub const GREEN: &str = "\x1b[32m";         // Green color
    pub const YELLOW: &str = "\x1b[33m";        // Yellow color
    pub const BLUE: &str = "\x1b[34m";          // Blue color
    pub const CYAN: &str = "\x1b[36m";          // Cyan color
    pub const RED: &str = "\x1b[31m";           // Red color
}

/// Stats display that updates in place
pub struct StatsDisplay {
    lines_printed: AtomicU64,
    first_print: AtomicBool,
    enabled: bool,
}

impl StatsDisplay {
    /// Create a new stats display
    pub fn new(enabled: bool) -> Self {
        Self {
            lines_printed: AtomicU64::new(0),
            first_print: AtomicBool::new(true),
            enabled,
        }
    }
    
    /// Clear the previous stats display
    fn clear_previous_lines(&self, count: u64) {
        if count == 0 {
            return;
        }
        
        for _ in 0..count {
            print!("{}{}", ansi::CURSOR_UP, ansi::CLEAR_LINE);
        }
        print!("\r");
        let _ = io::stdout().flush();
    }
    
    /// Display stats with in-place update
    pub fn display_stats(
        &self,
        packets_sent: u64,
        packets_failed: u64,
        bytes_sent: u64,
        elapsed_secs: f64,
        protocol_stats: &HashMap<String, AtomicU64>,
        system_stats: Option<&SystemStats>,
    ) {
        if !self.enabled {
            return;
        }
        
        // Calculate metrics
        let pps = packets_sent as f64 / elapsed_secs;
        let mbps = (bytes_sent as f64 * 8.0) / (elapsed_secs * stats_constants::MEGABITS_DIVISOR);
        
        // Clear previous display if not first print
        if !self.first_print.load(Ordering::Relaxed) {
            let lines = self.lines_printed.load(Ordering::Relaxed);
            self.clear_previous_lines(lines);
        } else {
            self.first_print.store(false, Ordering::Relaxed);
            // Hide cursor for cleaner display
            print!("{}", ansi::HIDE_CURSOR);
        }
        
        let mut lines = 0;
        
        // Print main stats line with colors
        println!(
            "{}📊 Stats{} - Sent: {}{}{}, Failed: {}{}{}, Rate: {}{:.1}{} pps, {}{:.2}{} Mbps",
            ansi::BOLD, ansi::RESET,
            ansi::GREEN, packets_sent, ansi::RESET,
            ansi::RED, packets_failed, ansi::RESET,
            ansi::CYAN, pps, ansi::RESET,
            ansi::BLUE, mbps, ansi::RESET
        );
        lines += 1;
        
        // Protocol breakdown with colors
        let protocols = ["UDP", "TCP", "ICMP", "IPv6", "ARP"];
        let mut protocol_line = String::new();
        let mut has_protocols = false;
        
        for protocol in &protocols {
            if let Some(counter) = protocol_stats.get(*protocol) {
                let count = counter.load(Ordering::Relaxed);
                if count > 0 {
                    if has_protocols {
                        protocol_line.push_str(" | ");
                    }
                    protocol_line.push_str(&format!(
                        "{}{}: {}{} packets{}",
                        ansi::DIM, protocol, ansi::YELLOW, count, ansi::RESET
                    ));
                    has_protocols = true;
                }
            }
        }
        
        if has_protocols {
            println!("   {}", protocol_line);
            lines += 1;
        }
        
        // System stats with colors
        if let Some(sys_stats) = system_stats {
            let cpu_color = if sys_stats.cpu_usage > 80.0 {
                ansi::RED
            } else if sys_stats.cpu_usage > 50.0 {
                ansi::YELLOW
            } else {
                ansi::GREEN
            };
            
            let mem_mb = sys_stats.memory_usage / stats_constants::BYTES_TO_MB_DIVISOR;
            let mem_color = if mem_mb > 10000 {
                ansi::RED
            } else if mem_mb > 5000 {
                ansi::YELLOW
            } else {
                ansi::GREEN
            };
            
            println!(
                "   {}System:{} CPU {}{:.1}%{}, Memory: {}{:.1} MB{}",
                ansi::DIM, ansi::RESET,
                cpu_color, sys_stats.cpu_usage, ansi::RESET,
                mem_color, mem_mb, ansi::RESET
            );
            lines += 1;
        }
        
        // Progress bar
        if packets_sent > 0 {
            let success_rate = ((packets_sent - packets_failed) as f64 / packets_sent as f64) * 100.0;
            let bar_width = 30;
            let filled = ((success_rate / 100.0) * bar_width as f64) as usize;
            let empty = bar_width - filled;
            
            let bar_color = if success_rate > 95.0 {
                ansi::GREEN
            } else if success_rate > 80.0 {
                ansi::YELLOW
            } else {
                ansi::RED
            };
            
            println!(
                "   {}Success:{} {}[{}{}]{}  {:.1}%",
                ansi::DIM, ansi::RESET,
                bar_color,
                "█".repeat(filled),
                "░".repeat(empty),
                ansi::RESET,
                success_rate
            );
            lines += 1;
        }
        
        self.lines_printed.store(lines, Ordering::Relaxed);
        let _ = io::stdout().flush();
    }
    
    /// Clear the display and show cursor on drop
    pub fn clear(&self) {
        if self.enabled && !self.first_print.load(Ordering::Relaxed) {
            let lines = self.lines_printed.load(Ordering::Relaxed);
            self.clear_previous_lines(lines);
            // Show cursor again
            print!("{}", ansi::SHOW_CURSOR);
            let _ = io::stdout().flush();
        }
    }
}

impl Drop for StatsDisplay {
    fn drop(&mut self) {
        // Ensure cursor is shown when display is dropped
        print!("{}", ansi::SHOW_CURSOR);
        let _ = io::stdout().flush();
    }
}

/// Global stats display instance
static STATS_DISPLAY: once_cell::sync::OnceCell<Arc<StatsDisplay>> = once_cell::sync::OnceCell::new();

/// Initialize the global stats display
pub fn init_display(enabled: bool) -> Arc<StatsDisplay> {
    STATS_DISPLAY.get_or_init(|| Arc::new(StatsDisplay::new(enabled))).clone()
}

/// Get the global stats display
pub fn get_display() -> Option<Arc<StatsDisplay>> {
    STATS_DISPLAY.get().cloned()
}