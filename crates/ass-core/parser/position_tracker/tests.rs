//! Unit tests for [`PositionTracker`] advancing and span construction.

use super::*;
#[test]
fn tracker_creation() {
    let source = "Hello\nWorld";
    let tracker = PositionTracker::new(source);
    assert_eq!(tracker.offset(), 0);
    assert_eq!(tracker.line(), 1);
    assert_eq!(tracker.column(), 1);
}

#[test]
fn tracker_advance_single_line() {
    let source = "Hello World";
    let mut tracker = PositionTracker::new(source);

    tracker.advance(5);
    assert_eq!(tracker.offset(), 5);
    assert_eq!(tracker.line(), 1);
    assert_eq!(tracker.column(), 6);

    tracker.advance(6);
    assert_eq!(tracker.offset(), 11);
    assert_eq!(tracker.line(), 1);
    assert_eq!(tracker.column(), 12);
}

#[test]
fn tracker_advance_multiline() {
    let source = "Hello\nWorld\nTest";
    let mut tracker = PositionTracker::new(source);

    tracker.advance(6); // "Hello\n"
    assert_eq!(tracker.offset(), 6);
    assert_eq!(tracker.line(), 2);
    assert_eq!(tracker.column(), 1);

    tracker.advance(6); // "World\n"
    assert_eq!(tracker.offset(), 12);
    assert_eq!(tracker.line(), 3);
    assert_eq!(tracker.column(), 1);
}

#[test]
fn tracker_skip_whitespace() {
    let source = "   Hello";
    let mut tracker = PositionTracker::new(source);

    tracker.skip_whitespace();
    assert_eq!(tracker.offset(), 3);
    assert_eq!(tracker.column(), 4);
}

#[test]
fn tracker_skip_line() {
    let source = "Hello World\nNext Line";
    let mut tracker = PositionTracker::new(source);

    tracker.skip_line();
    assert_eq!(tracker.offset(), 12);
    assert_eq!(tracker.line(), 2);
    assert_eq!(tracker.column(), 1);
}

#[test]
fn tracker_span_creation() {
    let source = "Hello\nWorld";
    let mut tracker = PositionTracker::new(source);

    let start = tracker.checkpoint();
    tracker.advance(5);

    let span = tracker.span_from(&start);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 5);
    assert_eq!(span.line, 1);
    assert_eq!(span.column, 1);
}

#[test]
fn tracker_remaining_text() {
    let source = "Hello World";
    let mut tracker = PositionTracker::new(source);

    tracker.advance(6);
    assert_eq!(tracker.remaining(), "World");
}

#[test]
fn tracker_advance_to() {
    let source = "Hello World Test";
    let mut tracker = PositionTracker::new(source);

    tracker.advance_to(11);
    assert_eq!(tracker.offset(), 11);
    assert_eq!(tracker.column(), 12);
}

#[test]
fn tracker_at_end() {
    let source = "Hi";
    let mut tracker = PositionTracker::new(source);

    assert!(!tracker.is_at_end());
    tracker.advance(2);
    assert!(tracker.is_at_end());
}

#[test]
fn tracker_new_at_position() {
    let source = "Hello\nWorld";
    let tracker = PositionTracker::new_at(source, 6, 2, 1);

    assert_eq!(tracker.offset(), 6);
    assert_eq!(tracker.line(), 2);
    assert_eq!(tracker.column(), 1);
}

#[test]
fn tracker_span_for() {
    let source = "Hello World";
    let tracker = PositionTracker::new(source);

    let span = tracker.span_for(5);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 5);
    assert_eq!(span.line, 1);
    assert_eq!(span.column, 1);
}

#[test]
fn tracker_windows_line_endings() {
    let source = "Hello\r\nWorld";
    let mut tracker = PositionTracker::new(source);

    tracker.advance(7); // "Hello\r\n"
    assert_eq!(tracker.offset(), 7);
    assert_eq!(tracker.line(), 2);
    assert_eq!(tracker.column(), 1);
}
