//! Unit tests for the dialogue event utility helpers.

use super::*;
use crate::analysis::events::dialogue_info::DialogueInfo;
use crate::parser::{ast::Span, Event};
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
fn create_test_dialogue_info(start: &'static str, end: &'static str) -> DialogueInfo<'static> {
    // Create a static event for the lifetime requirement
    // Using Box::leak is acceptable in tests for simplicity
    let event = Box::leak(Box::new(Event {
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
    }));
    DialogueInfo::analyze(event).unwrap()
}

#[cfg(feature = "std")]
fn create_test_dialogue_info(start: &'static str, end: &'static str) -> DialogueInfo<'static> {
    // Create a static event for the lifetime requirement
    // Using Box::leak is acceptable in tests for simplicity
    let event = Box::leak(Box::new(Event {
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
    }));
    DialogueInfo::analyze(event).unwrap()
}

#[test]
fn empty_events_no_overlaps() {
    let events = vec![];
    assert_eq!(find_overlapping_dialogue_events(&events).len(), 0);
    assert_eq!(count_overlapping_dialogue_events(&events), 0);
}

#[test]
fn calculate_duration_empty() {
    let events = vec![];
    assert_eq!(calculate_total_duration(&events), None);
    assert_eq!(calculate_average_duration(&events), None);
}

#[test]
fn calculate_duration_single_event() {
    let events = vec![create_test_dialogue_info("0:00:00.00", "0:00:05.00")];
    assert_eq!(calculate_total_duration(&events), Some(500)); // 5 seconds = 500cs
    assert_eq!(calculate_average_duration(&events), Some(500));
}

#[test]
fn calculate_duration_multiple_events() {
    let events = vec![
        create_test_dialogue_info("0:00:00.00", "0:00:05.00"),
        create_test_dialogue_info("0:00:03.00", "0:00:08.00"),
        create_test_dialogue_info("0:00:10.00", "0:00:15.00"),
    ];

    // Total: 0:00:00.00 to 0:00:15.00 = 1500cs
    assert_eq!(calculate_total_duration(&events), Some(1500));

    // Average: (500 + 500 + 500) / 3 = 500cs
    assert_eq!(calculate_average_duration(&events), Some(500));
}

#[test]
fn sort_events_maintains_order() {
    let mut events = vec![
        create_test_dialogue_info("0:00:05.00", "0:00:10.00"),
        create_test_dialogue_info("0:00:00.00", "0:00:05.00"),
        create_test_dialogue_info("0:00:02.00", "0:00:07.00"),
    ];

    sort_events_by_time(&mut events);

    assert_eq!(events[0].start_time_cs(), 0); // 0:00:00.00
    assert_eq!(events[1].start_time_cs(), 200); // 0:00:02.00
    assert_eq!(events[2].start_time_cs(), 500); // 0:00:05.00
}

#[test]
fn find_events_in_range_filters_correctly() {
    let events = vec![
        create_test_dialogue_info("0:00:00.00", "0:00:05.00"), // 0-500cs
        create_test_dialogue_info("0:00:03.00", "0:00:08.00"), // 300-800cs
        create_test_dialogue_info("0:00:10.00", "0:00:15.00"), // 1000-1500cs
    ];

    let indices = find_events_in_range(&events, 250, 600); // 2.5s to 6s
    assert_eq!(indices, vec![0, 1]); // First two events overlap this range
}

#[test]
fn sort_events_same_start_time() {
    // Test sorting when events have same start time (covers line 121)
    let mut events = vec![
        create_test_dialogue_info("0:00:05.00", "0:00:10.00"), // Same start, longer duration
        create_test_dialogue_info("0:00:05.00", "0:00:08.00"), // Same start, shorter duration
        create_test_dialogue_info("0:00:05.00", "0:00:12.00"), // Same start, longest duration
    ];

    sort_events_by_time(&mut events);

    // All should have same start time but be sorted by end time
    assert_eq!(events[0].start_time_cs(), 500);
    assert_eq!(events[0].end_time_cs(), 800); // Shortest duration first
    assert_eq!(events[1].start_time_cs(), 500);
    assert_eq!(events[1].end_time_cs(), 1000); // Medium duration second
    assert_eq!(events[2].start_time_cs(), 500);
    assert_eq!(events[2].end_time_cs(), 1200); // Longest duration last
}
