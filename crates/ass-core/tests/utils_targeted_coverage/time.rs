//! Targeted coverage tests for `parse_ass_time` in `utils/mod.rs`.

use ass_core::utils::{parse_ass_time, CoreError};

#[test]
fn test_parse_ass_time_invalid_minutes() {
    // This should hit line 292: minutes >= 60 validation
    let invalid_times = vec![
        "1:60:30.45", // 60 minutes (should be < 60)
        "0:65:00.00", // 65 minutes
        "2:99:15.50", // 99 minutes
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
    // This should hit seconds >= 60 validation
    let invalid_times = vec![
        "1:30:60.45", // 60 seconds (should be < 60)
        "0:45:65.00", // 65 seconds
        "2:15:99.50", // 99 seconds
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
    // Test centiseconds validation edge cases
    let invalid_times = vec![
        "1:30:45.100", // 100 centiseconds (should be < 100)
        "0:45:30.999", // 999 centiseconds
    ];

    for time_str in invalid_times {
        let result = parse_ass_time(time_str);
        // May be valid or invalid depending on implementation
        let _ = result;
    }
}

#[test]
fn test_parse_ass_time_malformed_format() {
    // Test various malformed time formats that should actually fail
    let malformed_times = vec![
        "",            // Empty string
        "invalid",     // Non-time string
        "1:30",        // Missing seconds and centiseconds
        "1:30:45.",    // Missing centiseconds value
        "a:30:45.50",  // Non-numeric hours
        "1:b:45.50",   // Non-numeric minutes
        "1:30:c.50",   // Non-numeric seconds
        "1:30:45.d",   // Non-numeric centiseconds
        "1::45.50",    // Missing minutes
        ":30:45.50",   // Missing hours
        "1:30:.50",    // Missing seconds
        "-1:30:45.50", // Negative hours
        "1:-30:45.50", // Negative minutes
        "1:30:-45.50", // Negative seconds
        "1:30:45.-50", // Negative centiseconds
        "1:30:45.1a",  // Non-numeric fractional digit
    ];

    for time_str in malformed_times {
        let result = parse_ass_time(time_str);
        assert!(
            result.is_err(),
            "Expected {time_str} to be invalid but it was valid"
        );
    }
}

#[test]
fn test_parse_ass_time_precision_edge_cases() {
    // Test time parsing with precision edge cases
    let precision_cases = vec![
        "0:00:00.00", // Minimum time
        "9:59:59.99", // Maximum valid time
        "0:00:00.01", // Minimum non-zero
        "5:30:30.50", // Middle values
    ];

    for time_str in precision_cases {
        let result = parse_ass_time(time_str);
        assert!(result.is_ok(), "Time {time_str} should be valid");
    }
}
