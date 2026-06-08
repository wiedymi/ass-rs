//! Tests for [`Event`] construction, predicates, and equality.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn event_dialogue_check() {
    let dialogue = Event {
        event_type: EventType::Dialogue,
        ..Event::default()
    };
    assert!(dialogue.is_dialogue());
    assert!(!dialogue.is_comment());

    let comment = Event {
        event_type: EventType::Comment,
        ..Event::default()
    };
    assert!(!comment.is_dialogue());
    assert!(comment.is_comment());
}

#[test]
fn event_default() {
    let event = Event::default();
    assert_eq!(event.event_type, EventType::Dialogue);
    assert_eq!(event.layer, "0");
    assert_eq!(event.start, "0:00:00.00");
    assert_eq!(event.end, "0:00:00.00");
    assert_eq!(event.style, "Default");
    assert_eq!(event.text, "");
}

#[test]
fn event_clone_eq() {
    let event = Event::default();
    let cloned = event.clone();
    assert_eq!(event, cloned);
}

#[test]
fn event_all_types() {
    // Test all event types
    let dialogue = Event {
        event_type: EventType::Dialogue,
        ..Event::default()
    };
    assert!(dialogue.is_dialogue());
    assert!(!dialogue.is_comment());

    let comment = Event {
        event_type: EventType::Comment,
        ..Event::default()
    };
    assert!(!comment.is_dialogue());
    assert!(comment.is_comment());

    let picture = Event {
        event_type: EventType::Picture,
        ..Event::default()
    };
    assert!(!picture.is_dialogue());
    assert!(!picture.is_comment());

    let sound = Event {
        event_type: EventType::Sound,
        ..Event::default()
    };
    assert!(!sound.is_dialogue());
    assert!(!sound.is_comment());

    let movie = Event {
        event_type: EventType::Movie,
        ..Event::default()
    };
    assert!(!movie.is_dialogue());
    assert!(!movie.is_comment());

    let command = Event {
        event_type: EventType::Command,
        ..Event::default()
    };
    assert!(!command.is_dialogue());
    assert!(!command.is_comment());
}

#[test]
fn event_comprehensive_creation() {
    let event = Event {
        event_type: EventType::Dialogue,
        layer: "5",
        start: "0:02:15.75",
        end: "0:02:20.25",
        style: "MainStyle",
        name: "Character",
        margin_l: "10",
        margin_r: "20",
        margin_v: "15",
        margin_t: None,
        margin_b: None,
        effect: "fadeIn",
        text: "Hello, world!",
        span: Span::new(0, 0, 0, 0),
    };

    assert_eq!(event.event_type, EventType::Dialogue);
    assert_eq!(event.layer, "5");
    assert_eq!(event.start, "0:02:15.75");
    assert_eq!(event.end, "0:02:20.25");
    assert_eq!(event.style, "MainStyle");
    assert_eq!(event.name, "Character");
    assert_eq!(event.margin_l, "10");
    assert_eq!(event.margin_r, "20");
    assert_eq!(event.margin_v, "15");
    assert_eq!(event.effect, "fadeIn");
    assert_eq!(event.text, "Hello, world!");
}

#[test]
fn event_debug_output() {
    let event = Event {
        event_type: EventType::Dialogue,
        text: "Test text",
        ..Event::default()
    };

    let debug_str = format!("{event:?}");
    assert!(debug_str.contains("Event"));
    assert!(debug_str.contains("Dialogue"));
    assert!(debug_str.contains("Test text"));
}

#[test]
fn event_equality() {
    let event1 = Event {
        event_type: EventType::Dialogue,
        text: "Same text",
        ..Event::default()
    };

    let event2 = Event {
        event_type: EventType::Dialogue,
        text: "Same text",
        ..Event::default()
    };

    assert_eq!(event1, event2);

    let event3 = Event {
        event_type: EventType::Comment,
        text: "Same text",
        ..Event::default()
    };

    assert_ne!(event1, event3);
}

#[test]
fn event_mixed_defaults() {
    let event = Event {
        event_type: EventType::Picture,
        start: "0:01:00.00",
        text: "Custom text",
        ..Event::default()
    };

    // Custom fields
    assert_eq!(event.event_type, EventType::Picture);
    assert_eq!(event.start, "0:01:00.00");
    assert_eq!(event.text, "Custom text");

    // Default fields
    assert_eq!(event.layer, "0");
    assert_eq!(event.end, "0:00:00.00");
    assert_eq!(event.style, "Default");
    assert_eq!(event.name, "");
    assert_eq!(event.effect, "");
}
