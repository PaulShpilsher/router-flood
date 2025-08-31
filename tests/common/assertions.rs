//! Custom test assertions and helpers

use router_flood::error::{RouterFloodError, ValidationError};
use std::fmt::Debug;

/// Assert that a Result contains a validation error with specific field
pub fn assert_validation_error<T: Debug>(result: Result<T, RouterFloodError>, expected_field: &str) {
    match result {
        Err(RouterFloodError::Validation(msg)) => {
            assert!(
                msg.contains(expected_field),
                "Expected validation error containing '{}', got '{}'",
                expected_field, msg
            );
        }
        Ok(val) => panic!("Expected validation error, got Ok({:?})", val),
        Err(e) => panic!("Expected validation error, got {:?}", e),
    }
}

/// Assert that a Result contains any validation error
pub fn assert_is_validation_error<T: Debug>(result: Result<T, RouterFloodError>) {
    match result {
        Err(RouterFloodError::Validation(_)) => {}
        Ok(val) => panic!("Expected validation error, got Ok({:?})", val),
        Err(e) => panic!("Expected validation error, got {:?}", e),
    }
}

/// Assert that a Result is Ok
pub fn assert_ok<T: Debug, E: Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(val) => val,
        Err(e) => panic!("Expected Ok, got Err({:?})", e),
    }
}

/// Assert that a Result is Err
pub fn assert_err<T: Debug, E: Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(val) => panic!("Expected Err, got Ok({:?})", val),
        Err(e) => e,
    }
}

/// Assert that two floating point values are approximately equal
pub fn assert_approx_eq(left: f64, right: f64, epsilon: f64) {
    let diff = (left - right).abs();
    assert!(
        diff < epsilon,
        "Values not approximately equal: {} != {} (diff: {})",
        left,
        right,
        diff
    );
}

/// Assert that a value is within a range
pub fn assert_in_range<T: PartialOrd + Debug>(value: T, min: T, max: T) {
    assert!(
        value >= min && value <= max,
        "Value {:?} not in range [{:?}, {:?}]",
        value,
        min,
        max
    );
}

/// Assert that a collection contains an element
pub fn assert_contains<T: PartialEq + Debug>(collection: &[T], element: &T) {
    assert!(
        collection.contains(element),
        "Collection does not contain {:?}",
        element
    );
}

/// Assert that a collection does not contain an element
pub fn assert_not_contains<T: PartialEq + Debug>(collection: &[T], element: &T) {
    assert!(
        !collection.contains(element),
        "Collection unexpectedly contains {:?}",
        element
    );
}

/// Helper to run a closure multiple times and ensure consistent results
pub fn assert_consistent<F, T>(times: usize, mut f: F) -> T
where
    F: FnMut() -> T,
    T: PartialEq + Debug + Clone,
{
    let first_result = f();
    for i in 1..times {
        let result = f();
        assert_eq!(
            result, first_result,
            "Inconsistent results on iteration {}: {:?} != {:?}",
            i, result, first_result
        );
    }
    first_result
}

/// Helper to test thread safety by running concurrent operations
pub fn test_concurrent<F>(threads: usize, iterations: usize, f: F)
where
    F: Fn() + Send + Sync + 'static + Clone,
{
    use std::thread;
    
    let mut handles = vec![];
    
    for _ in 0..threads {
        let f = f.clone();
        let handle = thread::spawn(move || {
            for _ in 0..iterations {
                f();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

/// Assert that a duration is within expected bounds
pub fn assert_duration_in_range(
    actual: std::time::Duration,
    expected: std::time::Duration,
    tolerance_percent: u32,
) {
    let tolerance = expected.as_millis() * tolerance_percent as u128 / 100;
    let min = expected.as_millis().saturating_sub(tolerance);
    let max = expected.as_millis() + tolerance;
    let actual_ms = actual.as_millis();
    
    assert!(
        actual_ms >= min && actual_ms <= max,
        "Duration {}ms not within {}% of expected {}ms (range: {}ms - {}ms)",
        actual_ms,
        tolerance_percent,
        expected.as_millis(),
        min,
        max
    );
}