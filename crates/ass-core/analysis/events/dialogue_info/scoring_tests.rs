//! Tests for scoring, overlap queries, and event-reference accessors.

use super::*;
use crate::parser::ast::{Event, EventType, Span};

#[test]
fn dialogue_info_overlaps_time_range() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:02.00",
        end: "0:00:05.00",
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
    };

    let info = DialogueInfo::analyze(&event).unwrap();

    // Event is 200-500 cs
    assert!(info.overlaps_time_range(100, 300)); // Overlaps start
    assert!(info.overlaps_time_range(400, 600)); // Overlaps end
    assert!(info.overlaps_time_range(300, 400)); // Contained within
    assert!(info.overlaps_time_range(100, 600)); // Contains event
    assert!(!info.overlaps_time_range(0, 200)); // Before event
    assert!(!info.overlaps_time_range(500, 600)); // After event
}

#[test]
fn dialogue_info_animation_and_complexity_scoring() {
    let simple_event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Simple text",
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
    };

    let complex_event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Complex {\\b1}bold{\\b0} {\\i1}italic{\\i0} {\\u1}underline{\\u0} text",
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
    };

    let simple_info = DialogueInfo::analyze(&simple_event).unwrap();
    let complex_info = DialogueInfo::analyze(&complex_event).unwrap();

    assert!(simple_info.animation_score() < complex_info.animation_score());
    assert!(simple_info.complexity_score() < complex_info.complexity_score());
}

#[test]
fn dialogue_info_override_count() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "{\\b1}Bold {\\k50}ka{\\k100}ra{\\k75}o{\\i1}ke {\\b0}text{\\i0}",
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
    };

    let info = DialogueInfo::analyze(&event).unwrap();
    assert_eq!(info.override_count(), 7);
}

#[test]
fn dialogue_info_performance_impact() {
    let low_impact_event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:02.00",
        text: "Hi",
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
    };

    let info = DialogueInfo::analyze(&low_impact_event).unwrap();
    // Performance impact depends on complexity score
    let _impact = info.performance_impact();
}

#[test]
fn dialogue_info_event_reference() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Fast {\\b1}dynamic{\\b0} {\\i1}text{\\i0}!",
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
    };

    let info = DialogueInfo::analyze(&event).unwrap();
    assert_eq!(
        info.event().text,
        "Fast {\\b1}dynamic{\\b0} {\\i1}text{\\i0}!"
    );
    assert_eq!(info.event().start, "0:00:00.00");
    assert_eq!(info.event().end, "0:00:05.00");
}
