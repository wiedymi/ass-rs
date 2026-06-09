//! Edge case tests for the `Spans` helper in the utils module.
//!
//! Covers span validation, position lookups, substring extraction, and
//! handling of varied source text formats.

use ass_core::utils::Spans;

/// Test `Spans` struct with invalid span data
#[test]
fn test_spans_invalid_span_validation() {
    let source = "Hello, World! This is test data.";
    let spans = Spans::new(source);

    // Test validate_span with span from this source (should be valid)
    let valid_span = &source[0..5]; // "Hello"
    assert!(spans.validate_span(valid_span));

    // Test with span from different text (should be invalid)
    let other_text = "Different text entirely";
    let other_span = &other_text[0..5]; // From different source
    assert!(!spans.validate_span(other_span));

    // Test with empty span
    let empty_span = &source[0..0];
    assert!(spans.validate_span(empty_span));
}

/// Test `span_offset`, `span_line`, and `span_column` with invalid spans
#[test]
fn test_spans_position_methods_invalid_input() {
    let source = "Line 1\nLine 2\nLine 3";
    let spans = Spans::new(source);

    // Test with valid spans
    let line1_span = &source[0..6]; // "Line 1"
    assert_eq!(spans.span_offset(line1_span), Some(0));
    assert_eq!(spans.span_line(line1_span), Some(1));
    assert_eq!(spans.span_column(line1_span), Some(1));

    let line2_span = &source[7..13]; // "Line 2"
    assert_eq!(spans.span_offset(line2_span), Some(7));
    assert_eq!(spans.span_line(line2_span), Some(2));
    assert_eq!(spans.span_column(line2_span), Some(1));

    // Test with span from different source (should return None)
    let other_text = "Different text";
    let invalid_span = &other_text[0..5];
    assert!(spans.span_offset(invalid_span).is_none());
    assert!(spans.span_line(invalid_span).is_none());
    assert!(spans.span_column(invalid_span).is_none());
}

/// Test `substring` with out-of-bounds ranges
#[test]
fn test_spans_substring_out_of_bounds() {
    let source = "Hello, World!";
    let spans = Spans::new(source);

    // Test with valid range
    let result = spans.substring(0..5);
    assert_eq!(result, Some("Hello"));

    // Test with range beyond source length
    let result = spans.substring(0..source.len() + 10);
    assert!(result.is_none());

    // Test with start beyond source length
    let result = spans.substring(source.len() + 5..source.len() + 10);
    assert!(result.is_none());

    // Test with backwards range (intentionally empty)
    #[allow(clippy::reversed_empty_ranges)]
    let result = spans.substring(10..5);
    assert!(result.is_none());

    // Test with exact bounds
    let result = spans.substring(0..source.len());
    assert_eq!(result, Some(source));
}

/// Test `Spans` with various source text formats
#[test]
fn test_spans_with_different_text_formats() {
    // Test with empty source
    let empty_source = "";
    let empty_spans = Spans::new(empty_source);

    let empty_span = &empty_source[0..0];
    assert!(empty_spans.validate_span(empty_span));
    assert_eq!(empty_spans.substring(0..0), Some(""));

    // Test with Unicode text
    let unicode_source = "Hello 🌍 World! こんにちは";
    let unicode_spans = Spans::new(unicode_source);

    // Create span with valid byte boundaries
    let hello_span = &unicode_source[0..5]; // "Hello"
    assert!(unicode_spans.validate_span(hello_span));

    // Test with various line endings
    let mixed_endings = "Line1\nLine2\r\nLine3\rLine4";
    let mixed_spans = Spans::new(mixed_endings);

    let full_span = &mixed_endings[0..mixed_endings.len()];
    assert!(mixed_spans.validate_span(full_span));
    assert_eq!(
        mixed_spans.substring(0..mixed_endings.len()),
        Some(mixed_endings)
    );
}
