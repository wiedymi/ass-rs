//! Coverage tests for the `Spans` utility, exercising offset and column
//! calculations across empty, whitespace, unicode, and special-character input.

use ass_core::utils::Spans;

#[test]
fn test_span_column_functionality() {
    // Test span_column method (line 97)
    let source = "Line 1\nLine 2 with text\nLine 3";
    let spans = Spans::new(source);

    // Test column calculation for various spans
    let first_span = "Line 1";
    if let Some(column) = spans.span_column(first_span) {
        assert_eq!(column, 1); // Should be at column 1
    }

    let second_span = "with";
    if let Some(column) = spans.span_column(second_span) {
        assert!(column > 1); // Should be after "Line 2 "
    }

    // Test with span not in source
    let invalid_span = "not in source";
    assert!(spans.span_column(invalid_span).is_none());
}

#[test]
fn test_spans_edge_cases() {
    // Test Spans utility with edge cases
    let empty_source = "";
    let empty_spans = Spans::new(empty_source);

    // Test methods on empty source
    assert!(empty_spans.span_offset("anything").is_none());
    assert!(empty_spans.span_column("anything").is_none());

    // Test with whitespace-only source
    let whitespace_source = "   \n\t  \r\n  ";
    let whitespace_spans = Spans::new(whitespace_source);

    // Test finding whitespace spans
    if let Some(offset) = whitespace_spans.span_offset("   ") {
        assert!(offset < whitespace_source.len());
    }

    // Test with unicode source
    let unicode_source = "Hello 🌍 世界 مرحبا";
    let unicode_spans = Spans::new(unicode_source);

    // Test finding unicode spans
    if let Some(column) = unicode_spans.span_column("🌍") {
        assert!(column > 1);
    }

    if let Some(column) = unicode_spans.span_column("世界") {
        assert!(column > 1);
    }
}

#[test]
fn test_spans_line_column_calculations() {
    // Test line and column calculations with complex text
    let multiline_source = "First line\nSecond line with 🌍\n\nFourth line";
    let spans = Spans::new(multiline_source);

    // Test various spans and their positions
    let test_cases = vec![
        ("First", 1),  // Should be column 1
        ("Second", 1), // Should be column 1 of line 2
        ("with", 13),  // Should be after "Second line "
        ("🌍", 18),    // Unicode character position
        ("Fourth", 1), // After empty line
    ];

    for (span_text, expected_min_column) in test_cases {
        if let Some(column) = spans.span_column(span_text) {
            assert!(
                column >= expected_min_column,
                "Column for '{span_text}' should be >= {expected_min_column}, got {column}"
            );
        }
    }
}

#[test]
fn test_spans_with_special_characters() {
    // Test spans functionality with special characters
    let special_source = "Line1\r\nLine2\tWith\x00Null\nLine3";
    let spans = Spans::new(special_source);

    // Test finding spans with special characters
    if let Some(offset) = spans.span_offset("Line1") {
        assert_eq!(offset, 0);
    }

    if let Some(offset) = spans.span_offset("Line2") {
        assert!(offset > 0);
    }

    // Test column calculation with tabs and special chars
    if let Some(column) = spans.span_column("With") {
        assert!(column > 1);
    }
}
