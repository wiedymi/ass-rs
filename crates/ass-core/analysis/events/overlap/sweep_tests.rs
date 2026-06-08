//! Ordering and equality tests for the sweep-line event primitives.

use super::sweep::{SweepEvent, SweepEventType};
use core::cmp::Ordering;

#[test]
fn test_sweep_event_ordering() {
    let start_event = SweepEvent {
        time: 1000,
        event_type: SweepEventType::Start,
        event_index: 0,
    };
    let end_event = SweepEvent {
        time: 1000,
        event_type: SweepEventType::End,
        event_index: 1,
    };

    // End events should come before start events at the same time
    assert!(end_event < start_event);
    assert_eq!(start_event.partial_cmp(&end_event), Some(Ordering::Greater));
}

#[test]
fn test_sweep_event_equality() {
    let event1 = SweepEvent {
        time: 1000,
        event_type: SweepEventType::Start,
        event_index: 0,
    };
    let event2 = SweepEvent {
        time: 1000,
        event_type: SweepEventType::Start,
        event_index: 1,
    };

    assert_eq!(event1, event2);
}

#[test]
fn test_sweep_event_ordering_different_times() {
    let early_event = SweepEvent {
        time: 500,
        event_type: SweepEventType::Start,
        event_index: 0,
    };
    let late_event = SweepEvent {
        time: 1000,
        event_type: SweepEventType::End,
        event_index: 1,
    };

    assert!(early_event < late_event);
}
