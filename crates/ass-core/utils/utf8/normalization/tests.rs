//! Unit tests for the text normalization utilities.

use super::*;

#[test]
fn normalize_line_endings_windows() {
    let input = "Line 1\r\nLine 2\r\nLine 3";
    let normalized = normalize_line_endings(input);
    assert_eq!(normalized, "Line 1\nLine 2\nLine 3");
}

#[test]
fn normalize_line_endings_mac() {
    let input = "Line 1\rLine 2\rLine 3";
    let normalized = normalize_line_endings(input);
    assert_eq!(normalized, "Line 1\nLine 2\nLine 3");
}

#[test]
fn normalize_line_endings_mixed() {
    let input = "Line 1\r\nLine 2\rLine 3\n";
    let normalized = normalize_line_endings(input);
    assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
}

#[test]
fn normalize_line_endings_unix() {
    let input = "Line 1\nLine 2\nLine 3\n";
    let normalized = normalize_line_endings(input);
    assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
}

#[test]
fn normalize_whitespace_basic() {
    let input = "Hello\u{00A0}World\u{2000}Test"; // Non-breaking space and en quad
    let normalized = normalize_whitespace(input, false);
    assert_eq!(normalized, "Hello World Test");
}

#[test]
fn normalize_whitespace_preserve_structure() {
    let input = "Hello\tWorld\nNext Line";
    let normalized = normalize_whitespace(input, false);
    assert_eq!(normalized, "Hello\tWorld\nNext Line");
}

#[test]
fn normalize_whitespace_collapse() {
    let input = "Hello    World   Test";
    let normalized = normalize_whitespace(input, true);
    assert_eq!(normalized, "Hello World Test");
}

#[test]
fn normalize_whitespace_no_collapse() {
    let input = "Hello    World   Test";
    let normalized = normalize_whitespace(input, false);
    assert_eq!(normalized, "Hello    World   Test");
}

#[test]
fn remove_control_chars_basic() {
    let input = "Hello\x00World\x1FTest";
    let cleaned = remove_control_chars(input);
    assert_eq!(cleaned, "HelloWorldTest");
}

#[test]
fn remove_control_chars_preserve_essential() {
    let input = "Hello\tWorld\nNext\rLine";
    let cleaned = remove_control_chars(input);
    assert_eq!(cleaned, "Hello\tWorld\nNext\rLine");
}

#[test]
fn trim_lines_basic() {
    let input = "  Line 1  \n\t Line 2 \t\n   Line 3   ";
    let trimmed = trim_lines(input);
    assert_eq!(trimmed, "Line 1\nLine 2\nLine 3");
}

#[test]
fn trim_lines_empty_lines() {
    let input = "Line 1\n   \nLine 3";
    let trimmed = trim_lines(input);
    assert_eq!(trimmed, "Line 1\n\nLine 3");
}

#[test]
fn collapse_consecutive_spaces_basic() {
    let input = "Hello    World   Test";
    let collapsed = collapse_consecutive_spaces(input);
    assert_eq!(collapsed, "Hello World Test");
}

#[test]
fn collapse_consecutive_spaces_preserve_other() {
    let input = "Hello\t\tWorld\n\nTest";
    let collapsed = collapse_consecutive_spaces(input);
    assert_eq!(collapsed, "Hello\t\tWorld\n\nTest");
}

#[test]
fn normalization_chain() {
    let input = "  Line 1  \r\n\t Line 2 \t\r   Line 3   ";
    let normalized = normalize_line_endings(input);
    let trimmed = trim_lines(&normalized);
    let final_result = normalize_whitespace(&trimmed, true);
    assert_eq!(final_result, "Line 1\nLine 2\nLine 3");
}
