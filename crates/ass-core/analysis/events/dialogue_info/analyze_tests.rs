//! Tests for dialogue analysis construction and timing validation.

use super::*;
use crate::parser::ast::{Event, EventType, Span};
#[cfg(not(feature = "std"))]
#[test]
fn dialogue_info_analyze_valid() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Hello world!",
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
    assert_eq!(info.duration_ms(), 5000);
    assert_eq!(info.duration_cs(), 500);
    assert_eq!(info.start_time_cs(), 0);
    assert_eq!(info.end_time_cs(), 500);
}

#[test]
fn dialogue_info_analyze_with_override_tags() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Hello {\\b1}bold{\\b0} world",
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
    assert_eq!(info.duration_ms(), 5000);
    assert_eq!(info.duration_cs(), 500);
    assert_eq!(info.start_time_cs(), 0);
    assert_eq!(info.end_time_cs(), 500);
    assert!(!info.text_analysis().override_tags().is_empty());
}

#[test]
fn dialogue_info_analyze_invalid_timing_start_after_end() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:05.00",
        end: "0:00:02.00",
        text: "Invalid timing",
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

    assert!(DialogueInfo::analyze(&event).is_err());
}

#[test]
fn dialogue_info_analyze_invalid_timing_equal() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:03.00",
        end: "0:00:03.00",
        text: "Zero duration",
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

    assert!(DialogueInfo::analyze(&event).is_err());
}

#[test]
fn dialogue_info_analyze_invalid_start_time() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "invalid_time",
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

    assert!(DialogueInfo::analyze(&event).is_err());
}

#[test]
fn dialogue_info_analyze_invalid_end_time() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "invalid_time",
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

    assert!(DialogueInfo::analyze(&event).is_err());
}
