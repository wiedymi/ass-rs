//! Behavioural tests for sweep-line overlap detection on valid inputs.

use super::*;
use crate::parser::{ast::Span, Event};
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

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
fn no_overlaps_empty_list() {
    let events = vec![];
    let result = find_overlapping_events(&events).unwrap();
    assert!(result.is_empty());
}

#[test]
fn no_overlaps_single_event() {
    let events = vec![create_test_event("0:00:00.00", "0:00:05.00")];
    let result = find_overlapping_events(&events).unwrap();
    assert!(result.is_empty());
}

#[test]
fn no_overlaps_sequential_events() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:05.00"),
        create_test_event("0:00:05.00", "0:00:10.00"),
        create_test_event("0:00:10.00", "0:00:15.00"),
    ];
    let result = find_overlapping_events(&events).unwrap();
    assert!(result.is_empty());
}

#[test]
fn simple_overlap() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:05.00"),
        create_test_event("0:00:03.00", "0:00:08.00"),
    ];
    let result = find_overlapping_events(&events).unwrap();
    assert_eq!(result, vec![(0, 1)]);
}

#[test]
fn multiple_overlaps() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:10.00"),
        create_test_event("0:00:02.00", "0:00:05.00"),
        create_test_event("0:00:03.00", "0:00:08.00"),
    ];
    let result = find_overlapping_events(&events).unwrap();
    assert_eq!(result, vec![(0, 1), (0, 2), (1, 2)]);
}

#[test]
fn count_overlaps() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:05.00"),
        create_test_event("0:00:03.00", "0:00:08.00"),
    ];
    let count = count_overlapping_events(&events).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn find_overlapping_event_refs_test() {
    let event1 = create_test_event("0:00:00.00", "0:00:05.00");
    let event2 = create_test_event("0:00:03.00", "0:00:08.00");
    let event3 = create_test_event("0:00:10.00", "0:00:15.00");

    let event_refs = vec![&event1, &event2, &event3];
    let result = find_overlapping_event_refs(&event_refs).unwrap();
    assert_eq!(result, vec![(0, 1)]);
}

#[test]
fn find_overlapping_event_refs_empty() {
    let event_refs: Vec<&Event> = vec![];
    let result = find_overlapping_event_refs(&event_refs).unwrap();
    assert!(result.is_empty());
}

#[test]
fn find_overlapping_event_refs_single() {
    let event1 = create_test_event("0:00:00.00", "0:00:05.00");
    let event_refs = vec![&event1];
    let result = find_overlapping_event_refs(&event_refs).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_overlaps_with_identical_times() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:05.00"),
        create_test_event("0:00:00.00", "0:00:05.00"),
    ];
    let result = find_overlapping_events(&events).unwrap();
    assert_eq!(result, vec![(0, 1)]);
}

#[test]
fn test_adjacent_events_no_overlap() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:05.00"),
        create_test_event("0:00:05.00", "0:00:10.00"),
    ];
    let result = find_overlapping_events(&events).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_complex_overlap_scenario() {
    let events = vec![
        create_test_event("0:00:00.00", "0:00:10.00"), // Event 0
        create_test_event("0:00:02.00", "0:00:05.00"), // Event 1 - overlaps with 0
        create_test_event("0:00:03.00", "0:00:08.00"), // Event 2 - overlaps with 0, 1
        create_test_event("0:00:06.00", "0:00:09.00"), // Event 3 - overlaps with 0, 2
        create_test_event("0:00:11.00", "0:00:15.00"), // Event 4 - no overlaps
    ];
    let result = find_overlapping_events(&events).unwrap();

    // Expected overlaps: (0,1), (0,2), (0,3), (1,2), (2,3)
    assert_eq!(result.len(), 5);
    assert!(result.contains(&(0, 1)));
    assert!(result.contains(&(0, 2)));
    assert!(result.contains(&(0, 3)));
    assert!(result.contains(&(1, 2)));
    assert!(result.contains(&(2, 3)));
}
