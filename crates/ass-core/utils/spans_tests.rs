//! Tests for the zero-copy [`Spans`] utilities.

use super::*;

#[test]
fn spans_validation() {
    let source = "Hello, World!";
    let spans = Spans::new(source);

    let valid_span = &source[0..5]; // "Hello"
    assert!(spans.validate_span(valid_span));
    assert_eq!(spans.span_offset(valid_span), Some(0));
    assert_eq!(spans.span_line(valid_span), Some(1));
    assert_eq!(spans.span_column(valid_span), Some(1));

    let another_span = &source[7..12]; // "World"
    assert!(spans.validate_span(another_span));
    assert_eq!(spans.span_offset(another_span), Some(7));
}

#[test]
fn spans_multiline() {
    let source = "Line 1\nLine 2\nLine 3";
    let spans = Spans::new(source);

    let line2_span = &source[7..13]; // "Line 2"
    assert_eq!(spans.span_line(line2_span), Some(2));
    assert_eq!(spans.span_column(line2_span), Some(1));
}

#[test]
fn spans_edge_cases() {
    let source = "line1\nline2\nline3";
    let spans = Spans::new(source);

    // Test span validation with actual substrings
    let line1 = &source[0..5]; // "line1"
    let line2 = &source[6..11]; // "line2"
    let line3 = &source[12..17]; // "line3"

    assert!(spans.validate_span(line1));
    assert!(spans.validate_span(line2));
    assert!(spans.validate_span(line3));
    assert!(spans.validate_span(source)); // Entire source

    // Test span offset calculations
    assert_eq!(spans.span_offset(line1), Some(0));
    assert_eq!(spans.span_offset(line2), Some(6));
    assert_eq!(spans.span_offset(line3), Some(12));

    // Test line calculations
    assert_eq!(spans.span_line(line1), Some(1));
    assert_eq!(spans.span_line(line2), Some(2));
    assert_eq!(spans.span_line(line3), Some(3));

    // Test column calculations
    assert_eq!(spans.span_column(line1), Some(1));
    assert_eq!(spans.span_column(line2), Some(1));
    assert_eq!(spans.span_column(line3), Some(1));

    // Test substring extraction
    assert_eq!(spans.substring(0..5), Some("line1"));
    assert_eq!(spans.substring(6..11), Some("line2"));
    assert_eq!(spans.substring(12..17), Some("line3"));
    assert_eq!(spans.substring(0..source.len()), Some(source));

    // Test invalid range
    assert_eq!(spans.substring(0..100), None);
}
