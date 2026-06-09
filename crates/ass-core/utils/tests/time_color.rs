//! Time and color string parsing tests for the utils module.

use crate::utils::{parse_color_string, parse_time_string};

#[cfg(not(feature = "std"))]
#[test]
fn time_parsing_valid_formats() {
    // Test standard H:MM:SS.CC format
    assert!(parse_time_string("0:00:30.50").is_ok());
    assert!(parse_time_string("1:23:45.67").is_ok());
    assert!(parse_time_string("12:34:56.78").is_ok());

    // Test single digit components
    assert!(parse_time_string("0:01:02.03").is_ok());

    // Test edge cases
    assert!(parse_time_string("0:00:00.00").is_ok());
    assert!(parse_time_string("23:59:59.99").is_ok());
}

#[test]
fn time_parsing_invalid_formats() {
    // Missing components
    assert!(parse_time_string("").is_err());
    assert!(parse_time_string("1:23").is_err());
    assert!(parse_time_string("1:23:45").is_err());

    // Invalid characters
    assert!(parse_time_string("a:23:45.67").is_err());
    assert!(parse_time_string("1:ab:45.67").is_err());
    assert!(parse_time_string("1:23:cd.67").is_err());
    assert!(parse_time_string("1:23:45.ef").is_err());

    // Out of range values
    assert!(parse_time_string("1:60:45.67").is_err());
    assert!(parse_time_string("1:23:60.67").is_err());
    assert!(parse_time_string("1:23:45.100").is_err());
}

#[test]
fn time_parsing_precision() {
    let result = parse_time_string("1:23:45.67").unwrap();
    assert_eq!(result.hours, 1);
    assert_eq!(result.minutes, 23);
    assert_eq!(result.seconds, 45);
    assert_eq!(result.centiseconds, 67);
}

#[test]
fn color_parsing_hex_formats() {
    // Standard hex colors
    assert!(parse_color_string("#FF0000").is_ok());
    assert!(parse_color_string("#00FF00").is_ok());
    assert!(parse_color_string("#0000FF").is_ok());

    // ASS format colors
    assert!(parse_color_string("&H0000FF&").is_ok());
    assert!(parse_color_string("&HFF0000&").is_ok());
    assert!(parse_color_string("&H00FF00&").is_ok());

    // Without alpha
    assert!(parse_color_string("FF0000").is_ok());
    assert!(parse_color_string("00FF00").is_ok());

    // With alpha
    assert!(parse_color_string("&H80FF0000&").is_ok());
}

#[test]
fn color_parsing_invalid_formats() {
    // Invalid hex characters
    assert!(parse_color_string("&HGGGGGG&").is_err());
    assert!(parse_color_string("#ZZZZZZ").is_err());

    // Wrong length
    assert!(parse_color_string("&HFF&").is_err());
    assert!(parse_color_string("#FF").is_err());
    assert!(parse_color_string("FFFFFFF").is_err());

    // Malformed ASS format
    assert!(parse_color_string("&HFF0000").is_err());
    assert!(parse_color_string("FF0000&").is_err());
}
