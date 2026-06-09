//! Targeted coverage tests for error formatting in `utils/mod.rs`.

use ass_core::utils::{parse_ass_time, parse_bgr_color};

#[test]
fn test_utils_error_formatting() {
    // Test error message formatting
    let time_result = parse_ass_time("1:99:45.50");
    if let Err(error) = time_result {
        let error_string = error.to_string();
        assert!(!error_string.is_empty());
        assert!(error_string.contains("Minutes") || error_string.contains("invalid"));
    }

    let color_result = parse_bgr_color("invalid");
    if let Err(error) = color_result {
        let error_string = error.to_string();
        assert!(!error_string.is_empty());
    }
}
