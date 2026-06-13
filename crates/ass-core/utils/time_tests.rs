//! Tests for ASS time parsing and formatting.

use super::*;

#[test]
fn parse_ass_times() {
    assert_eq!(parse_ass_time("0:00:00.00").unwrap(), 0);
    assert_eq!(parse_ass_time("0:00:01.00").unwrap(), 100);
    assert_eq!(parse_ass_time("0:01:00.00").unwrap(), 6000);
    assert_eq!(parse_ass_time("1:00:00.00").unwrap(), 360_000);
    assert_eq!(parse_ass_time("0:01:30.50").unwrap(), 9050);
}

#[test]
fn parse_ass_times_invalid() {
    assert!(parse_ass_time("invalid").is_err());
    assert!(parse_ass_time("0:60:00.00").is_err()); // Invalid minutes
    assert!(parse_ass_time("0:00:60.00").is_err()); // Invalid seconds
    assert!(parse_ass_time("0:00:00.xx").is_err()); // Non-numeric fraction
}

#[test]
fn parse_ass_time_millisecond_precision() {
    // libass and many real files use 3-digit (millisecond) fractional seconds;
    // they parse as truncated centiseconds rather than erroring.
    assert_eq!(parse_ass_time("0:00:00.100").unwrap(), 10); // 100ms = 10cs
    assert_eq!(parse_ass_time("0:00:27.021").unwrap(), 2702); // 27.021s
    assert_eq!(parse_ass_time("0:00:00.098").unwrap(), 9); // 98ms -> 9cs (truncated)
    assert_eq!(parse_ass_time("0:00:00.5").unwrap(), 50); // tenths
    assert_eq!(parse_ass_time("0:00:00.05").unwrap(), 5); // centiseconds
}

#[test]
fn parse_ass_time_ms_keeps_full_precision() {
    // Unlike parse_ass_time (centiseconds, truncating), this keeps milliseconds.
    assert_eq!(parse_ass_time_ms("0:00:00.00").unwrap(), 0);
    assert_eq!(parse_ass_time_ms("0:00:01.00").unwrap(), 1000);
    assert_eq!(parse_ass_time_ms("0:01:00.00").unwrap(), 60_000);
    assert_eq!(parse_ass_time_ms("1:00:00.00").unwrap(), 3_600_000);

    // Fractional scaling by digit count: tenths, centiseconds, milliseconds.
    assert_eq!(parse_ass_time_ms("0:00:00.5").unwrap(), 500); // tenths
    assert_eq!(parse_ass_time_ms("0:00:00.05").unwrap(), 50); // centiseconds
    assert_eq!(parse_ass_time_ms("0:00:00.098").unwrap(), 98); // milliseconds, no truncation
    assert_eq!(parse_ass_time_ms("0:00:27.021").unwrap(), 27_021);

    // The sub-centisecond remainder parse_ass_time would drop is preserved here:
    // 0.098s is 98ms, not 90ms (9cs).
    assert_eq!(parse_ass_time("0:00:00.098").unwrap() * 10, 90);
    assert_eq!(parse_ass_time_ms("0:00:00.098").unwrap(), 98);
}

#[test]
fn parse_ass_time_ms_invalid() {
    assert!(parse_ass_time_ms("invalid").is_err());
    assert!(parse_ass_time_ms("0:60:00.00").is_err()); // Invalid minutes
    assert!(parse_ass_time_ms("0:00:60.00").is_err()); // Invalid seconds
    assert!(parse_ass_time_ms("0:00:00.xx").is_err()); // Non-numeric fraction
    assert!(parse_ass_time_ms("0:00").is_err()); // Missing component
}

#[test]
fn format_ass_times() {
    assert_eq!(format_ass_time(0), "0:00:00.00");
    assert_eq!(format_ass_time(100), "0:00:01.00");
    assert_eq!(format_ass_time(6000), "0:01:00.00");
    assert_eq!(format_ass_time(360_000), "1:00:00.00");
    assert_eq!(format_ass_time(9050), "0:01:30.50");
}

#[test]
fn parse_ass_time_edge_cases() {
    // Test maximum valid values
    assert!(parse_ass_time("23:59:59.99").is_ok());

    // Test zero padding variations
    assert_eq!(parse_ass_time("0:0:0.0").unwrap(), 0);
    assert_eq!(parse_ass_time("0:00:00.0").unwrap(), 0);
    assert_eq!(parse_ass_time("0:00:00.00").unwrap(), 0);

    // Test missing components
    assert!(parse_ass_time("0:00").is_err());
    assert!(parse_ass_time("0").is_err());
    assert!(parse_ass_time("").is_err());

    // Test extra components
    assert!(parse_ass_time("0:0:0:0.0").is_err());
    // Note: parse_ass_time("0:0:0.0.0") actually succeeds by taking first decimal part
    assert!(parse_ass_time("0:0:0.0.0").is_ok());

    // Test negative values
    assert!(parse_ass_time("-1:00:00.00").is_err());
    assert!(parse_ass_time("0:-1:00.00").is_err());
    assert!(parse_ass_time("0:00:-1.00").is_err());
    assert!(parse_ass_time("0:00:00.-1").is_err());

    // Test non-numeric values
    assert!(parse_ass_time("a:00:00.00").is_err());
    assert!(parse_ass_time("0:b:00.00").is_err());
    assert!(parse_ass_time("0:00:c.00").is_err());
    assert!(parse_ass_time("0:00:00.d").is_err());

    // Test boundary values that should fail
    assert!(parse_ass_time("0:60:00.00").is_err()); // 60 minutes
    assert!(parse_ass_time("0:00:60.00").is_err()); // 60 seconds
    assert_eq!(parse_ass_time("0:00:00.100").unwrap(), 10); // 100ms = 10cs, not invalid
}

#[test]
fn format_ass_time_edge_cases() {
    // Test very large values
    assert_eq!(format_ass_time(u32::MAX), "11930:27:52.95");

    // Test boundary values
    assert_eq!(format_ass_time(99), "0:00:00.99");
    assert_eq!(format_ass_time(5999), "0:00:59.99");
    assert_eq!(format_ass_time(359_999), "0:59:59.99");

    // Test values requiring padding
    assert_eq!(format_ass_time(1), "0:00:00.01");
    assert_eq!(format_ass_time(10), "0:00:00.10");
    assert_eq!(format_ass_time(601), "0:00:06.01");
    assert_eq!(format_ass_time(3661), "0:00:36.61");
}
