//! Targeted coverage tests for `Spans` helpers in `utils/mod.rs`.

use ass_core::utils::Spans;

#[test]
fn test_spans_column_with_invalid_span() {
    // This should hit line 97: span_column when span_offset returns None
    let source = "Hello\nWorld\nTest";
    let spans = Spans::new(source);

    // Test with span not in source
    let invalid_span = "NotInSource";
    let result = spans.span_column(invalid_span);
    assert!(result.is_none());

    // Test with empty span
    let empty_span = "";
    let result = spans.span_column(empty_span);
    // Empty span might return None depending on implementation
    let _ = result;
}

#[test]
fn test_spans_substring_out_of_bounds() {
    // Test substring with out of bounds range
    let source = "Hello World";
    let spans = Spans::new(source);

    // Test range beyond source length
    let result = spans.substring(20..25);
    assert!(result.is_none());

    // Test range starting beyond source length
    let result = spans.substring(15..20);
    assert!(result.is_none());

    // Test empty range at end of string
    let result = spans.substring(11..11);
    assert_eq!(result, Some(""));

    // Test range at exact boundary
    let result = spans.substring(10..12);
    assert!(result.is_none());
}

#[test]
fn test_spans_edge_cases_with_unicode() {
    // Test spans with Unicode content
    let unicode_source = "Hello 世界\n测试 🎬\nEnd";
    let spans = Spans::new(unicode_source);

    // Test span_line with Unicode - get actual substring from source
    let unicode_start = unicode_source.find("世界").unwrap();
    let unicode_span = &unicode_source[unicode_start..unicode_start + "世界".len()];
    let line = spans.span_line(unicode_span);
    assert!(line.is_some());

    // Test span_column with Unicode
    let column = spans.span_column(unicode_span);
    assert!(column.is_some());

    // Test with emoji - get actual substring from source
    let emoji_start = unicode_source.find("🎬").unwrap();
    let emoji_span = &unicode_source[emoji_start..emoji_start + "🎬".len()];
    let emoji_line = spans.span_line(emoji_span);
    let emoji_column = spans.span_column(emoji_span);
    assert!(emoji_line.is_some());
    assert!(emoji_column.is_some());
}

#[test]
fn test_spans_with_special_characters() {
    // Test spans with special characters and edge cases
    let special_source = "Line1\r\nLine2\n\nLine4\r";
    let spans = Spans::new(special_source);

    // Test with carriage return + newline
    let crlf_span = "\r\n";
    let _ = spans.span_line(crlf_span);
    let _ = spans.span_column(crlf_span);

    // Test with empty lines
    let empty_line_span = "\n\n";
    let _ = spans.span_line(empty_line_span);

    // Test with carriage return only
    let cr_span = "\r";
    let _ = spans.span_line(cr_span);
}

#[test]
fn test_spans_offset_calculation_edge_cases() {
    // Test span offset calculation with edge cases
    let source = "Multi\nLine\nContent\nWith\nMany\nLines";
    let spans = Spans::new(source);

    // Test with spans at line boundaries - get actual substring from source
    let newline_pos = source.find('\n').unwrap();
    let newline_span = &source[newline_pos..=newline_pos];
    let newline_offset = spans.span_offset(newline_span);
    assert!(newline_offset.is_some());

    // Test with partial matches - get actual substring from source
    let line_pos = source.find("Line").unwrap();
    let partial_span = &source[line_pos..line_pos + "Line".len()];
    let offset = spans.span_offset(partial_span);
    assert!(offset.is_some());

    // Test with first and last characters
    let first_char = &source[0..1];
    let first_offset = spans.span_offset(first_char);
    assert!(first_offset.is_some());

    let last_char = &source[source.len() - 1..];
    let last_offset = spans.span_offset(last_char);
    assert!(last_offset.is_some());
}
