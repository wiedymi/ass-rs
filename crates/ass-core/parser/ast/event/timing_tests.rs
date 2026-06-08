//! Tests for [`Event`] centisecond time parsing and duration.

use super::*;

#[test]
fn event_time_parsing() {
    let event = Event {
        start: "0:01:30.50",
        end: "0:01:35.00",
        ..Event::default()
    };

    // Test start time parsing
    assert_eq!(event.start_time_cs().unwrap(), 9050); // 1*60*100 + 30*100 + 50

    // Test end time parsing
    assert_eq!(event.end_time_cs().unwrap(), 9500); // 1*60*100 + 35*100

    // Test duration calculation
    assert_eq!(event.duration_cs().unwrap(), 450); // 9500 - 9050
}

#[test]
fn event_time_parsing_edge_cases() {
    // Test zero time
    let zero_event = Event {
        start: "0:00:00.00",
        end: "0:00:00.00",
        ..Event::default()
    };
    assert_eq!(zero_event.start_time_cs().unwrap(), 0);
    assert_eq!(zero_event.end_time_cs().unwrap(), 0);
    assert_eq!(zero_event.duration_cs().unwrap(), 0);

    // Test negative duration (end before start)
    let negative_event = Event {
        start: "0:01:00.00",
        end: "0:00:30.00",
        ..Event::default()
    };
    assert_eq!(negative_event.duration_cs().unwrap(), 0); // saturating_sub returns 0
}

#[test]
fn event_time_parsing_errors() {
    // Test invalid time formats
    let invalid_start = Event {
        start: "invalid",
        end: "0:00:05.00",
        ..Event::default()
    };
    assert!(invalid_start.start_time_cs().is_err());
    assert!(invalid_start.duration_cs().is_err());

    let invalid_end = Event {
        start: "0:00:00.00",
        end: "invalid",
        ..Event::default()
    };
    assert!(invalid_end.end_time_cs().is_err());
    assert!(invalid_end.duration_cs().is_err());
}
