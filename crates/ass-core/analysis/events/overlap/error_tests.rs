//! Error-path tests for sweep-line overlap detection on malformed input.

use super::detection::find_overlaps_generic;
use super::*;
use crate::parser::{ast::Span, Event};
use crate::utils::CoreError;
#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

fn create_test_event(start: &'static str, end: &'static str) -> Event<'static> {
    Event {
        event_type: crate::parser::ast::EventType::Dialogue,
        start,
        end,
        text: "Test",
        layer: "0",
        style: "Default",
        name: "",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        margin_t: None,
        margin_b: None,
        effect: "",
        span: Span::new(0, 0, 0, 0),
    }
}

#[test]
fn test_invalid_start_time_format_error() {
    let events = vec![
        create_test_event("invalid_time", "0:00:05.00"),
        create_test_event("0:00:03.00", "0:00:08.00"),
    ];
    let result = find_overlapping_events(&events);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid start time format"));
}

#[test]
fn test_invalid_end_time_format_error() {
    let events = vec![
        create_test_event("0:00:00.00", "invalid_time"),
        create_test_event("0:00:03.00", "0:00:08.00"),
    ];
    let result = find_overlapping_events(&events);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid end time format"));
}

#[test]
fn test_invalid_time_formats_in_multiple_events() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:05.00"), // Valid
        create_test_event("bad_start", "0:00:08.00"),  // Invalid start
        create_test_event("0:00:03.00", "bad_end"),    // Invalid end
    ];
    let result = find_overlapping_events(&events);
    assert!(result.is_err());
    // Should fail on first invalid time encountered
}

#[test]
fn test_count_overlapping_events_with_invalid_times() {
    let events = vec![
        create_test_event("invalid_time", "0:00:05.00"),
        create_test_event("0:00:03.00", "0:00:08.00"),
    ];
    let result = count_overlapping_events(&events);
    assert!(result.is_err());
}

#[test]
fn test_find_overlapping_event_refs_with_invalid_times() {
    let event1 = create_test_event("invalid_start", "0:00:05.00");
    let event2 = create_test_event("0:00:03.00", "0:00:08.00");
    let event_refs = vec![&event1, &event2];
    let result = find_overlapping_event_refs(&event_refs);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid start time format"));
}

#[test]
fn test_find_overlaps_generic_error_propagation() {
    let events = ["valid", "invalid"];
    let result = find_overlaps_generic(events.iter(), |_index, event| {
        if *event == "invalid" {
            Err(CoreError::parse("Test error"))
        } else {
            Ok((1000u32, 2000u32))
        }
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Test error"));
}

#[test]
fn test_malformed_time_formats_various() {
    let test_cases = vec![
        ("invalid", "0:00:05.00"),       // Non-time format
        ("0:60:00.00", "0:00:05.00"),    // Invalid minute (60)
        ("0:00:60.00", "0:00:05.00"),    // Invalid second (60)
        ("0:00:00.100", "0:00:05.00"),   // Invalid centiseconds (100)
        ("0:00:00.00", "invalid"),       // Invalid end time
        ("0:00:00.00", "0:60:00.00"),    // Invalid end minute
        ("abc:def:gh.ij", "0:00:05.00"), // Non-numeric components
    ];

    for (start, end) in test_cases {
        // Need at least 2 events to trigger time parsing in overlap detection
        let events = vec![
            create_test_event(start, end), // Event with invalid time
            create_test_event("0:00:10.00", "0:00:15.00"), // Valid event for comparison
        ];
        let result = find_overlapping_events(&events);
        assert!(
            result.is_err(),
            "Expected error for time format: {start} -> {end}, got: {result:?}"
        );
    }
}

#[test]
fn test_find_overlapping_event_refs_with_invalid_end_time() {
    let event1 = create_test_event("0:00:00.00", "invalid_end");
    let event2 = create_test_event("0:00:03.00", "0:00:08.00");
    let event_refs = vec![&event1, &event2];
    let result = find_overlapping_event_refs(&event_refs);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid end time format"));
}
