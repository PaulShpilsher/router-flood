//! Progress indicators and user interface enhancements
//!
//! This module provides progress bars, status indicators, and other UI elements
//! to improve the user experience during long-running operations.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

/// Simple progress indicator for operations
pub struct ProgressIndicator {
    message: String,
    start_time: Instant,
    running: Arc<AtomicBool>,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        print!("🔄 {} ", message);
        let _ = io::stdout().flush(); // Ignore flush errors for UI
        
        Self {
            message,
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Start the spinner animation
    pub async fn start_spinner(&self) {
        let running = self.running.clone();
        let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut i = 0;
        
        while running.load(Ordering::Relaxed) {
            print!("\r🔄 {} {}", self.message, chars[i % chars.len()]);
            let _ = io::stdout().flush(); // Ignore flush errors for UI
            i += 1;
            time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Complete the progress indicator with success
    pub fn complete_success(&self, result_message: Option<&str>) {
        self.running.store(false, Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();
        
        if let Some(msg) = result_message {
            println!("\r✅ {} - {} ({:.2}s)", self.message, msg, elapsed.as_secs_f64());
        } else {
            println!("\r✅ {} ({:.2}s)", self.message, elapsed.as_secs_f64());
        }
    }

    /// Complete the progress indicator with error
    pub fn complete_error(&self, error_message: &str) {
        self.running.store(false, Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();
        println!("\r❌ {} - {} ({:.2}s)", self.message, error_message, elapsed.as_secs_f64());
    }
}

impl Drop for ProgressIndicator {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        // Clear the line if not already completed
        print!("\r{}\r", " ".repeat(80));
        let _ = io::stdout().flush(); // Ignore flush errors for UI
    }
}

/// Real-time statistics display for packet flooding
pub struct StatsDisplay {
    packets_sent: Arc<AtomicU64>,
    packets_failed: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    start_time: Instant,
    last_update: Instant,
    last_packets: u64,
    last_bytes: u64,
}

impl StatsDisplay {
    /// Create a new statistics display
    pub fn new(
        packets_sent: Arc<AtomicU64>,
        packets_failed: Arc<AtomicU64>,
        bytes_sent: Arc<AtomicU64>,
    ) -> Self {
        Self {
            packets_sent,
            packets_failed,
            bytes_sent,
            start_time: Instant::now(),
            last_update: Instant::now(),
            last_packets: 0,
            last_bytes: 0,
        }
    }

    /// Update and display current statistics
    pub fn update_display(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time);
        let interval = now.duration_since(self.last_update);
        
        let current_packets = self.packets_sent.load(Ordering::Relaxed);
        let current_failed = self.packets_failed.load(Ordering::Relaxed);
        let current_bytes = self.bytes_sent.load(Ordering::Relaxed);
        
        // Calculate rates
        let packets_diff = current_packets.saturating_sub(self.last_packets);
        let bytes_diff = current_bytes.saturating_sub(self.last_bytes);
        
        let pps = if interval.as_secs_f64() > 0.0 {
            packets_diff as f64 / interval.as_secs_f64()
        } else {
            0.0
        };
        
        let mbps = if interval.as_secs_f64() > 0.0 {
            (bytes_diff as f64 * 8.0) / (interval.as_secs_f64() * 1_000_000.0)
        } else {
            0.0
        };
        
        let avg_pps = if elapsed.as_secs_f64() > 0.0 {
            current_packets as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        let success_rate = if current_packets + current_failed > 0 {
            (current_packets as f64 / (current_packets + current_failed) as f64) * 100.0
        } else {
            100.0
        };
        
        // Format display
        print!(
            "\r📊 Packets: {} | Failed: {} | Rate: {:.1} PPS ({:.2} Mbps) | Avg: {:.1} PPS | Success: {:.1}% | Time: {:.1}s",
            format_number(current_packets),
            format_number(current_failed),
            pps,
            mbps,
            avg_pps,
            success_rate,
            elapsed.as_secs_f64()
        );
        let _ = io::stdout().flush(); // Ignore flush errors for UI
        
        // Update tracking values
        self.last_update = now;
        self.last_packets = current_packets;
        self.last_bytes = current_bytes;
    }

    /// Clear the display line
    pub fn clear(&self) {
        print!("\r{}", " ".repeat(120));
        print!("\r");
        let _ = io::stdout().flush(); // Ignore flush errors for UI
    }
}

/// Format large numbers with appropriate suffixes
pub fn format_number(num: u64) -> String {
    if num >= 1_000_000_000 {
        format!("{:.1}B", num as f64 / 1_000_000_000.0)
    } else if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

/// Display a startup banner with version and safety information
pub fn display_startup_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("🚀 Router Flood v{} - Educational Network Stress Tester", version);
    println!("═══════════════════════════════════════════════════════════");
    println!("⚠️  EDUCATIONAL USE ONLY - Private Networks Only");
    println!("🛡️  Built-in Safety: IP Validation, Rate Limiting, Audit Logging");
    println!("📚 Use --help for examples and --dry-run for safe testing");
    println!();
}

/// Display a completion summary
pub fn display_completion_summary(
    packets_sent: u64,
    packets_failed: u64,
    bytes_sent: u64,
    duration: Duration,
    dry_run: bool,
) {
    println!();
    println!("📈 {} Summary:", if dry_run { "Simulation" } else { "Test" });
    println!("═══════════════════════════════════════");
    println!("📦 Packets Sent: {}", format_number(packets_sent));
    println!("❌ Packets Failed: {}", format_number(packets_failed));
    println!("📊 Data Transmitted: {}", format_bytes(bytes_sent));
    println!("⏱️  Duration: {:.2} seconds", duration.as_secs_f64());
    
    if packets_sent > 0 {
        let avg_pps = packets_sent as f64 / duration.as_secs_f64();
        let avg_mbps = (bytes_sent as f64 * 8.0) / (duration.as_secs_f64() * 1_000_000.0);
        println!("📈 Average Rate: {:.1} PPS ({:.2} Mbps)", avg_pps, avg_mbps);
        
        let success_rate = (packets_sent as f64 / (packets_sent + packets_failed) as f64) * 100.0;
        println!("✅ Success Rate: {:.1}%", success_rate);
    }
    
    if dry_run {
        println!("🔍 DRY-RUN: No actual packets were sent");
    }
    println!();
}

/// Format bytes with appropriate units
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.2} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} bytes", bytes)
    }
}

