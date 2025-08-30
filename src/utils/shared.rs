//! Shared utilities and common patterns extracted from across the codebase
//!
//! This module consolidates repeated patterns and provides reusable components
//! to reduce code duplication and improve maintainability.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

/// Common atomic counter pattern used throughout the codebase
#[derive(Debug)]
pub struct AtomicCounter {
    value: AtomicU64,
    name: &'static str,
}

impl AtomicCounter {
    pub fn new(name: &'static str) -> Self {
        Self {
            value: AtomicU64::new(0),
            name,
        }
    }
    
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
    
    pub fn add(&self, amount: u64) -> u64 {
        self.value.fetch_add(amount, Ordering::Relaxed)
    }
    
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
    
    pub fn reset(&self) -> u64 {
        self.value.swap(0, Ordering::Relaxed)
    }
    
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Clone for AtomicCounter {
    fn clone(&self) -> Self {
        Self {
            value: AtomicU64::new(self.value.load(Ordering::Relaxed)),
            name: self.name,
        }
    }
}

/// Common running flag pattern used across workers and managers
#[derive(Debug)]
pub struct RunningFlag {
    inner: Arc<AtomicBool>,
}

impl RunningFlag {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AtomicBool::new(true)),
        }
    }
    
    pub fn is_running(&self) -> bool {
        self.inner.load(Ordering::Relaxed)
    }
    
    pub fn stop(&self) {
        self.inner.store(false, Ordering::Relaxed);
    }
    
    pub fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
    
    /// Get the underlying Arc<AtomicBool> for compatibility with existing code
    pub fn as_arc(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.inner)
    }
}

impl Default for RunningFlag {
    fn default() -> Self {
        Self::new()
    }
}

/// Common rate calculation pattern
#[derive(Debug)]
pub struct RateCalculator {
    start_time: Instant,
    last_count: AtomicU64,
    last_time: std::sync::Mutex<Instant>,
}

impl RateCalculator {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_count: AtomicU64::new(0),
            last_time: std::sync::Mutex::new(now),
        }
    }
    
    pub fn calculate_rate(&self, current_count: u64) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time).as_secs_f64();
        
        if elapsed > 0.0 {
            current_count as f64 / elapsed
        } else {
            0.0
        }
    }
    
    pub fn calculate_instantaneous_rate(&self, current_count: u64) -> f64 {
        let now = Instant::now();
        let mut last_time = self.last_time.lock().unwrap();
        let last_count = self.last_count.swap(current_count, Ordering::Relaxed);
        
        let time_diff = now.duration_since(*last_time).as_secs_f64();
        *last_time = now;
        
        if time_diff > 0.0 && current_count >= last_count {
            (current_count - last_count) as f64 / time_diff
        } else {
            0.0
        }
    }
}

impl Default for RateCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Common jitter application pattern for timing randomization
pub struct JitterApplier {
    min_factor: f64,
    max_factor: f64,
}

impl JitterApplier {
    pub fn new(min_factor: f64, max_factor: f64) -> Self {
        Self { min_factor, max_factor }
    }
    
    pub fn apply_jitter(&self, base_duration: Duration) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let factor = rng.gen_range(self.min_factor..self.max_factor);
        Duration::from_nanos((base_duration.as_nanos() as f64 * factor) as u64)
    }
    
    pub async fn sleep_with_jitter(&self, base_duration: Duration) {
        let jittered_duration = self.apply_jitter(base_duration);
        time::sleep(jittered_duration).await;
    }
}

impl Default for JitterApplier {
    fn default() -> Self {
        Self::new(0.8, 1.2) // ±20% jitter
    }
}

/// Common percentage calculation pattern
pub fn calculate_percentage(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

/// Common success rate calculation pattern
pub fn calculate_success_rate(successful: u64, failed: u64) -> f64 {
    let total = successful + failed;
    if total == 0 {
        100.0
    } else {
        calculate_percentage(successful, total)
    }
}

/// Common bandwidth calculation pattern
pub fn calculate_bandwidth_mbps(bytes: u64, duration_secs: f64) -> f64 {
    if duration_secs > 0.0 {
        (bytes as f64 * 8.0) / (duration_secs * 1_000_000.0)
    } else {
        0.0
    }
}

/// Common memory size formatting
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Common duration formatting
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Common validation patterns
pub mod validation {
    /// Validate that a value is within a range
    pub fn validate_range<T: PartialOrd + Copy + std::fmt::Debug>(
        value: T,
        min: T,
        max: T,
        field_name: &str,
    ) -> Result<(), String> {
        if value < min || value > max {
            Err(format!("{} must be between {:?} and {:?}", field_name, min, max))
        } else {
            Ok(())
        }
    }
    
    /// Validate that a value is positive
    pub fn validate_positive<T: PartialOrd + Default + Copy>(
        value: T,
        field_name: &str,
    ) -> Result<(), String> {
        if value <= T::default() {
            Err(format!("{} must be greater than 0", field_name))
        } else {
            Ok(())
        }
    }
    
    /// Validate that a collection is not empty
    pub fn validate_not_empty<T>(
        collection: &[T],
        field_name: &str,
    ) -> Result<(), String> {
        if collection.is_empty() {
            Err(format!("{} cannot be empty", field_name))
        } else {
            Ok(())
        }
    }
}

/// Common retry patterns
pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_factor: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = config.base_delay;
    
    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt == config.max_attempts - 1 {
                    return Err(error);
                }
                
                time::sleep(delay).await;
                delay = std::cmp::min(
                    Duration::from_nanos((delay.as_nanos() as f64 * config.backoff_factor) as u64),
                    config.max_delay,
                );
            }
        }
    }
    
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new("test");
        assert_eq!(counter.get(), 0);
        
        counter.increment();
        assert_eq!(counter.get(), 1);
        
        counter.add(5);
        assert_eq!(counter.get(), 6);
        
        let old_value = counter.reset();
        assert_eq!(old_value, 6);
        assert_eq!(counter.get(), 0);
    }
    
    #[test]
    fn test_running_flag() {
        let flag = RunningFlag::new();
        assert!(flag.is_running());
        
        flag.stop();
        assert!(!flag.is_running());
        
        let cloned = flag.clone();
        assert!(!cloned.is_running());
    }
    
    #[test]
    fn test_rate_calculator() {
        let calc = RateCalculator::new();
        
        // Test basic rate calculation
        std::thread::sleep(Duration::from_millis(10));
        let rate = calc.calculate_rate(100);
        assert!(rate > 0.0);
    }
    
    #[test]
    fn test_percentage_calculations() {
        assert_eq!(calculate_percentage(50, 100), 50.0);
        assert_eq!(calculate_percentage(0, 100), 0.0);
        assert_eq!(calculate_percentage(100, 0), 0.0);
        
        assert_eq!(calculate_success_rate(90, 10), 90.0);
        assert_eq!(calculate_success_rate(0, 0), 100.0);
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }
    
    #[test]
    fn test_validation() {
        use validation::*;
        
        assert!(validate_range(5, 1, 10, "value").is_ok());
        assert!(validate_range(0, 1, 10, "value").is_err());
        assert!(validate_range(11, 1, 10, "value").is_err());
        
        assert!(validate_positive(1, "value").is_ok());
        assert!(validate_positive(0, "value").is_err());
        
        assert!(validate_not_empty(&[1, 2, 3], "list").is_ok());
        assert!(validate_not_empty::<i32>(&[], "list").is_err());
    }
    
    #[tokio::test]
    async fn test_retry_with_backoff() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_factor: 2.0,
        };
        
        let mut attempt_count = 0;
        let result = retry_with_backoff(&config, || {
            attempt_count += 1;
            async move {
                if attempt_count < 3 {
                    Err("not ready")
                } else {
                    Ok("success")
                }
            }
        }).await;
        
        assert_eq!(result, Ok("success"));
        assert_eq!(attempt_count, 3);
    }
}