//! Fuzz testing for CLI argument parsing
//!
//! This fuzzer tests CLI parsing with malformed inputs to ensure
//! the argument parser handles all edge cases gracefully.

#![no_main]

use libfuzzer-sys::fuzz_target;
use router_flood::cli::*;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, handling invalid UTF-8 gracefully
    let input_str = String::from_utf8_lossy(data);
    
    // Test port parsing with various inputs
    let _ = parse_ports(&input_str);
    
    // Test with common port patterns
    let port_patterns = [
        input_str.clone(),
        format!("80,{}", input_str),
        format!("{},443", input_str),
        format!("80,{},443", input_str),
        input_str.replace(" ", ","),
        input_str.replace("\n", ","),
    ];
    
    for pattern in &port_patterns {
        let _ = parse_ports(pattern);
    }
    
    // Test positive number parsing
    let _ = parse_positive_number::<u64>(&input_str, "test_field");
    let _ = parse_positive_number::<usize>(&input_str, "test_field");
    let _ = parse_positive_number::<u16>(&input_str, "test_field");
    
    // Test export format parsing
    let _ = parse_export_format(&input_str);
    
    // Test with common format variations
    let format_variations = [
        input_str.to_lowercase(),
        input_str.to_uppercase(),
        input_str.trim().to_string(),
        format!(" {} ", input_str),
    ];
    
    for variation in &format_variations {
        let _ = parse_export_format(variation);
    }
    
    // Test with numeric inputs of various formats
    if let Ok(num_str) = std::str::from_utf8(data) {
        // Test various numeric parsing scenarios
        let _ = num_str.parse::<u64>();
        let _ = num_str.parse::<i64>();
        let _ = num_str.parse::<f64>();
        
        // Test with modified numeric strings
        let modified_nums = [
            format!("0{}", num_str),
            format!("-{}", num_str),
            format!("{}.0", num_str),
            format!("{}e10", num_str),
        ];
        
        for modified in &modified_nums {
            let _ = parse_positive_number::<u64>(modified, "fuzz_field");
        }
    }
});