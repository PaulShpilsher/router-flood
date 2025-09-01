//! Graceful shutdown integration tests

use router_flood::Stats;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

#[test]
fn test_graceful_shutdown_with_stats() {
    let stats = Arc::new(Stats::new(None));
    let shutdown = Arc::new(AtomicBool::new(false));
    
    let stats_clone = Arc::clone(&stats);
    let shutdown_clone = Arc::clone(&shutdown);
    
    let handle = thread::spawn(move || {
        while !shutdown_clone.load(Ordering::Relaxed) {
            stats_clone.increment_sent(1000, "test");
            thread::sleep(Duration::from_millis(10));
        }
    });
    
    // Let it run for a bit
    thread::sleep(Duration::from_millis(100));
    
    // Signal shutdown
    shutdown.store(true, Ordering::Relaxed);
    
    // Wait for thread to finish
    handle.join().unwrap();
    
    // Verify stats were collected
    assert!(stats.packets_sent() > 0);
    assert!(stats.bytes_sent() > 0);
}

#[test]
fn test_resource_cleanup_on_shutdown() {
    let resources_cleaned = Arc::new(AtomicBool::new(false));
    let resources_clone = Arc::clone(&resources_cleaned);
    
    // Simulate resource that needs cleanup
    struct Resource {
        cleaned: Arc<AtomicBool>,
    }
    
    impl Drop for Resource {
        fn drop(&mut self) {
            self.cleaned.store(true, Ordering::Relaxed);
        }
    }
    
    {
        let _resource = Resource {
            cleaned: resources_clone,
        };
        // Resource goes out of scope
    }
    
    // Verify cleanup happened
    assert!(resources_cleaned.load(Ordering::Relaxed));
}

#[test]
fn test_concurrent_shutdown() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];
    
    // Start multiple workers
    for i in 0..5 {
        let shutdown_clone = Arc::clone(&shutdown);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            while !shutdown_clone.load(Ordering::Relaxed) {
                count += 1;
                thread::sleep(Duration::from_millis(5));
            }
            (i, count)
        }));
    }
    
    // Let them run
    thread::sleep(Duration::from_millis(50));
    
    // Signal shutdown
    shutdown.store(true, Ordering::Relaxed);
    
    // Collect results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.join().unwrap());
    }
    
    // Verify all threads stopped
    assert_eq!(results.len(), 5);
    for (id, count) in results {
        assert!(count > 0, "Thread {} didn't do any work", id);
    }
}