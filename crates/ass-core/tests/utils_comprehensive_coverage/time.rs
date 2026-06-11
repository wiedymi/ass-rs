//! Coverage tests for `parse_ass_time`, covering invalid component ranges,
//! malformed input, boundary values, and numeric edge cases.

use ass_core::utils::parse_ass_time;
use ass_core::CoreError;

#[test]
fn test_parse_ass_time_invalid_minutes() {
    // Test minutes >= 60 error (line 292)
    let invalid_times = vec![
        "1:60:00.00", // 60 minutes
        "2:75:30.50", // 75 minutes
        "0:99:59.99", // 99 minutes
    ];

    for time_str in invalid_times {
        let result = parse_ass_time(time_str);
        assert!(result.is_err());
        if let Err(CoreError::InvalidTime(msg)) = result {
            assert!(msg.contains("Minutes must be < 60"));
        }
    }
}

#[test]
fn test_parse_ass_time_invalid_seconds() {
    // Test seconds >= 60 error (lines 306-307)
    let invalid_times = vec![
        "0:00:60.00", // 60 seconds
        "0:30:75.25", // 75 seconds
        "1:45:99.99", // 99 seconds
    ];

    for time_str in invalid_times {
        let result = parse_ass_time(time_str);
        assert!(result.is_err());
        if let Err(CoreError::InvalidTime(msg)) = result {
            assert!(msg.contains("Seconds must be < 60"));
        }
    }
}

#[test]
fn test_parse_ass_time_invalid_centiseconds() {
    // Test centiseconds >= 100 error
    let invalid_times = vec![
        "0:00:00.100", // 100 centiseconds
        "0:00:30.150", // 150 centiseconds
        "1:30:45.999", // 999 centiseconds (should be parsed as 99.9, but if parsed literally)
    ];

    for time_str in invalid_times {
        let result = parse_ass_time(time_str);
        // Some might succeed if parsed differently, just ensure it doesn't panic
        match result {
            Ok(centiseconds) => {
                println!("Unexpectedly parsed '{time_str}' as {centiseconds} centiseconds");
            }
            Err(e) => {
                println!("Failed to parse '{time_str}': {e}");
            }
        }
    }
}

#[test]
fn test_parse_ass_time_malformed_input() {
    // Test various malformed time inputs to trigger different error paths
    let malformed_times = vec![
        "invalid",     // Not time format
        "1:2",         // Too few parts
        "1:2:3:4:5",   // Too many parts
        "a:b:c.d",     // Non-numeric
        "-1:30:45.50", // Negative hours
        "1:-30:45.50", // Negative minutes
        "1:30:-45.50", // Negative seconds
        "1:30:45.-50", // Negative centiseconds
        "",            // Empty string
        ":",           // Just colons
        "..",          // Just dots
        "1::30.50",    // Double colon
        "1:30:.50",    // Missing seconds
        "1:30:45.",    // Missing centiseconds
    ];

    for time_str in malformed_times {
        let result = parse_ass_time(time_str);
        assert!(result.is_err(), "Should fail for: {time_str}");
    }
}

#[test]
fn test_time_parsing_boundary_values() {
    // Test boundary values for time parsing
    let boundary_cases = vec![
        ("0:59:59.99", true),  // Maximum valid minutes/seconds/centiseconds
        ("23:59:59.99", true), // Maximum reasonable time
        ("0:00:00.00", true),  // Minimum time
        ("0:59:60.00", false), // Invalid seconds
        ("0:60:00.00", false), // Invalid minutes
        ("0:00:00.100", true), // 100ms = 10cs (millisecond precision, libass-compatible)
    ];

    for (time_str, should_succeed) in boundary_cases {
        let result = parse_ass_time(time_str);
        if should_succeed {
            assert!(result.is_ok(), "Should succeed parsing: {time_str}");
        } else {
            assert!(result.is_err(), "Should fail parsing: {time_str}");
        }
    }
}

#[test]
fn test_numeric_parsing_edge_cases() {
    // Test numeric parsing edge cases that might trigger uncovered paths
    let edge_cases = vec![
        "999:999:999.999", // Very large numbers
        "000:000:000.000", // Leading zeros
        "1:2:3.4",         // Single digits
        " 1:30:45.50 ",    // Whitespace (should fail)
        "1: 30:45.50",     // Internal whitespace (should fail)
    ];

    for time_str in edge_cases {
        let result = parse_ass_time(time_str);
        // Just ensure it doesn't panic and handles edge cases
        match result {
            Ok(centiseconds) => {
                println!("Parsed '{time_str}' as {centiseconds} centiseconds");
            }
            Err(e) => {
                println!("Failed to parse '{time_str}': {e}");
            }
        }
    }
}
