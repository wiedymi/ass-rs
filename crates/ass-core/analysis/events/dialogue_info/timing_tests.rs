//! Tests for timing-relationship detection between dialogue events.

use super::*;
use crate::parser::ast::{Event, EventType, Span};

#[test]
fn dialogue_info_timing_relation_no_overlap() {
    let event1 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "First event",
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

    let event2 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:06.00",
        end: "0:00:10.00",
        text: "Second event",
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

    let info1 = DialogueInfo::analyze(&event1).unwrap();
    let info2 = DialogueInfo::analyze(&event2).unwrap();

    assert_eq!(info1.timing_relation(&info2), TimingRelation::NoOverlap);
    assert_eq!(info2.timing_relation(&info1), TimingRelation::NoOverlap);
}

#[test]
fn dialogue_info_timing_relation_partial_overlap() {
    let event1 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:02.50",
        end: "0:00:07.50",
        text: "Event on layer 1",
        layer: "1",
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

    let event2 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:02.00",
        end: "0:00:04.00",
        text: "Second",
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

    let info1 = DialogueInfo::analyze(&event1).unwrap();
    let info2 = DialogueInfo::analyze(&event2).unwrap();

    assert_eq!(
        info1.timing_relation(&info2),
        TimingRelation::PartialOverlap
    );
    assert_eq!(
        info2.timing_relation(&info1),
        TimingRelation::PartialOverlap
    );
}

#[test]
fn dialogue_info_timing_relation_full_overlap() {
    let event1 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Complex {\\b1}bold{\\b0} text",
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

    let event2 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:01.00",
        end: "0:00:03.00",
        text: "Inner",
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

    let info1 = DialogueInfo::analyze(&event1).unwrap();
    let info2 = DialogueInfo::analyze(&event2).unwrap();

    assert_eq!(info1.timing_relation(&info2), TimingRelation::FullOverlap);
    assert_eq!(info2.timing_relation(&info1), TimingRelation::FullOverlap);
}

#[test]
fn dialogue_info_timing_relation_identical() {
    let event1 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Container",
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

    let event2 = Event {
        event_type: EventType::Dialogue,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Second",
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

    let info1 = DialogueInfo::analyze(&event1).unwrap();
    let info2 = DialogueInfo::analyze(&event2).unwrap();

    assert_eq!(info1.timing_relation(&info2), TimingRelation::Identical);
    assert_eq!(info2.timing_relation(&info1), TimingRelation::Identical);
}
