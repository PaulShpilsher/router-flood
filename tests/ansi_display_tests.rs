//! Tests for ANSI escape code functionality in stats display
//!
//! Tests the ANSI terminal control codes used for in-place updates,
//! colors, and cursor manipulation.

// Test ANSI escape codes directly
mod ansi_codes {
    pub const CLEAR_LINE: &str = "\x1b[2K";
    pub const CURSOR_UP: &str = "\x1b[A";
    pub const HIDE_CURSOR: &str = "\x1b[?25l";
    pub const SHOW_CURSOR: &str = "\x1b[?25h";
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED: &str = "\x1b[31m";
}

#[test]
fn test_ansi_clear_line() {
    // Test that clear line code is correct
    assert_eq!(ansi_codes::CLEAR_LINE, "\x1b[2K");
    assert_eq!(ansi_codes::CLEAR_LINE.len(), 4);
}

#[test]
fn test_ansi_cursor_movement() {
    // Test cursor movement codes
    assert_eq!(ansi_codes::CURSOR_UP, "\x1b[A");
    assert!(ansi_codes::CURSOR_UP.starts_with("\x1b["));
}

#[test]
fn test_ansi_cursor_visibility() {
    // Test cursor hide/show codes
    assert_eq!(ansi_codes::HIDE_CURSOR, "\x1b[?25l");
    assert_eq!(ansi_codes::SHOW_CURSOR, "\x1b[?25h");
    
    // Both should start with ESC
    assert!(ansi_codes::HIDE_CURSOR.starts_with('\x1b'));
    assert!(ansi_codes::SHOW_CURSOR.starts_with('\x1b'));
}

#[test]
fn test_ansi_color_codes() {
    // Test color codes
    assert_eq!(ansi_codes::GREEN, "\x1b[32m");
    assert_eq!(ansi_codes::YELLOW, "\x1b[33m");
    assert_eq!(ansi_codes::RED, "\x1b[31m");
    assert_eq!(ansi_codes::RESET, "\x1b[0m");
    
    // All color codes should be escape sequences
    assert!(ansi_codes::GREEN.starts_with("\x1b["));
    assert!(ansi_codes::YELLOW.starts_with("\x1b["));
    assert!(ansi_codes::RED.starts_with("\x1b["));
}

#[test]
fn test_ansi_text_attributes() {
    // Test text attribute codes
    assert_eq!(ansi_codes::BOLD, "\x1b[1m");
    assert_eq!(ansi_codes::RESET, "\x1b[0m");
}

#[test]
fn test_colored_text_formatting() {
    // Test combining colors with text
    let green_text = format!("{}Success{}", ansi_codes::GREEN, ansi_codes::RESET);
    assert!(green_text.contains(ansi_codes::GREEN));
    assert!(green_text.contains(ansi_codes::RESET));
    assert!(green_text.contains("Success"));
    
    let red_text = format!("{}Failed{}", ansi_codes::RED, ansi_codes::RESET);
    assert!(red_text.contains(ansi_codes::RED));
    assert!(red_text.contains("Failed"));
    
    let bold_text = format!("{}Important{}", ansi_codes::BOLD, ansi_codes::RESET);
    assert!(bold_text.contains(ansi_codes::BOLD));
    assert!(bold_text.contains("Important"));
}

#[test]
fn test_progress_bar_characters() {
    // Test Unicode characters used in progress bar
    let filled = "█";
    let empty = "░";
    
    // Verify they are valid UTF-8
    assert_eq!(filled.len(), 3); // UTF-8 encoded length
    assert_eq!(empty.len(), 3);  // UTF-8 encoded length
    
    // Test progress bar construction
    let progress = format!("[{}{}]", filled.repeat(5), empty.repeat(5));
    assert!(progress.starts_with('['));
    assert!(progress.ends_with(']'));
    assert_eq!(progress.matches('█').count(), 5);
    assert_eq!(progress.matches('░').count(), 5);
}

#[test]
fn test_multi_line_clear_sequence() {
    // Test clearing multiple lines
    let lines_to_clear = 3;
    let mut clear_sequence = String::new();
    
    for _ in 0..lines_to_clear {
        clear_sequence.push_str(ansi_codes::CURSOR_UP);
        clear_sequence.push_str(ansi_codes::CLEAR_LINE);
    }
    
    // Should have 3 cursor ups and 3 clear lines
    assert_eq!(clear_sequence.matches(ansi_codes::CURSOR_UP).count(), 3);
    assert_eq!(clear_sequence.matches(ansi_codes::CLEAR_LINE).count(), 3);
}

#[test]
fn test_color_based_on_value() {
    // Test color selection based on thresholds
    fn get_cpu_color(cpu: f64) -> &'static str {
        if cpu > 80.0 {
            ansi_codes::RED
        } else if cpu > 50.0 {
            ansi_codes::YELLOW
        } else {
            ansi_codes::GREEN
        }
    }
    
    assert_eq!(get_cpu_color(85.0), ansi_codes::RED);
    assert_eq!(get_cpu_color(60.0), ansi_codes::YELLOW);
    assert_eq!(get_cpu_color(30.0), ansi_codes::GREEN);
    assert_eq!(get_cpu_color(80.0), ansi_codes::YELLOW); // Exactly 80
    assert_eq!(get_cpu_color(50.0), ansi_codes::GREEN);  // Exactly 50
}

#[test]
fn test_formatted_stats_string() {
    // Test building a formatted stats string with colors
    let sent = 1000u64;
    let failed = 50u64;
    let rate = 100.5f64;
    
    let formatted = format!(
        "{}Sent: {}{}{}, {}Failed: {}{}{}, Rate: {}{:.1}{}",
        ansi_codes::BOLD,
        ansi_codes::GREEN, sent, ansi_codes::RESET,
        ansi_codes::BOLD,
        ansi_codes::RED, failed, ansi_codes::RESET,
        ansi_codes::BOLD, rate, ansi_codes::RESET
    );
    
    // Verify all components are present
    assert!(formatted.contains(&sent));
    assert!(formatted.contains(&failed));
    assert!(formatted.contains("100.5"));
    assert!(formatted.contains(ansi_codes::GREEN));
    assert!(formatted.contains(ansi_codes::RED));
    assert!(formatted.contains(ansi_codes::BOLD));
}

#[test]
fn test_escape_sequence_stripping() {
    // Test removing ANSI codes for plain text
    fn strip_ansi(text: &str) -> String {
        // Simple ANSI stripping for testing
        let mut result = text;
        let codes = [
            ansi_codes::CLEAR_LINE,
            ansi_codes::CURSOR_UP,
            ansi_codes::HIDE_CURSOR,
            ansi_codes::SHOW_CURSOR,
            ansi_codes::RESET,
            ansi_codes::BOLD,
            ansi_codes::GREEN,
            ansi_codes::YELLOW,
            ansi_codes::RED,
        ];
        
        for code in &codes {
            result = result.replace(code, "");
        }
        result
    }
    
    let colored = format!("{}Success: {}100%{}", ansi_codes::GREEN, ansi_codes::BOLD, ansi_codes::RESET);
    let plain = strip_ansi(&colored);
    
    assert_eq!(plain, "Success: 100%");
    assert!(!plain.contains('\x1b'));
}

#[test]
fn test_terminal_restoration_sequence() {
    // Test sequence for restoring terminal state
    let restore_sequence = format!("{}{}", ansi_codes::SHOW_CURSOR, ansi_codes::RESET);
    
    assert!(restore_sequence.contains(ansi_codes::SHOW_CURSOR));
    assert!(restore_sequence.contains(ansi_codes::RESET));
    assert_eq!(restore_sequence.len(), ansi_codes::SHOW_CURSOR.len() + ansi_codes::RESET.len());
}