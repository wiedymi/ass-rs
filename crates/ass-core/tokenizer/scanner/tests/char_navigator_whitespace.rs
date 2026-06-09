//! Tests for `CharNavigator` whitespace skipping and newline tracking.

use crate::tokenizer::scanner::CharNavigator;

#[test]
fn char_navigator_newline_tracking() {
    let source = "line1\nline2\r\nline3\rline4";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    // Advance to first newline
    for _ in 0..5 {
        navigator.advance_char().unwrap();
    }
    assert_eq!(navigator.line(), 1);

    // Cross Unix newline
    navigator.advance_char().unwrap(); // \n
    assert_eq!(navigator.line(), 2);
    assert_eq!(navigator.column(), 1);

    // Advance to Windows newline
    for _ in 0..5 {
        navigator.advance_char().unwrap();
    }

    // Cross Windows newline
    navigator.advance_char().unwrap(); // \r
    assert_eq!(navigator.line(), 3);
    navigator.advance_char().unwrap(); // \n
    assert_eq!(navigator.line(), 3); // Should not increment again for \n after \r

    // Advance to Mac newline
    for _ in 0..5 {
        navigator.advance_char().unwrap();
    }

    // Cross Mac newline
    navigator.advance_char().unwrap(); // \r
    assert_eq!(navigator.line(), 4);
    assert_eq!(navigator.column(), 1);
}

#[test]
fn char_navigator_skip_whitespace() {
    let source = "   \t  abc";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    navigator.skip_whitespace();
    assert_eq!(navigator.peek_char().unwrap(), 'a');
    assert_eq!(navigator.column(), 7); // Should be at position after whitespace
}

#[test]
fn char_navigator_skip_whitespace_with_newlines() {
    let source = " \n \t\r\n  abc";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    navigator.skip_whitespace();
    assert_eq!(navigator.peek_char().unwrap(), '\n'); // Should stop at newline
    assert_eq!(navigator.line(), 1); // Should not have crossed newlines
}

#[test]
fn char_navigator_skip_whitespace_empty() {
    let source = "";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    navigator.skip_whitespace(); // Should not panic
    assert!(navigator.is_at_end());
}

#[test]
fn char_navigator_skip_whitespace_only() {
    let source = "   \t\n  ";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    navigator.skip_whitespace();
    assert_eq!(navigator.peek_char().unwrap(), '\n'); // Should stop at newline
}

#[test]
fn char_navigator_column_reset_on_newline() {
    let source = "abc\ndef";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    // Advance to position 3 (column 4)
    for _ in 0..3 {
        navigator.advance_char().unwrap();
    }
    assert_eq!(navigator.column(), 4);

    // Cross newline - should reset column
    navigator.advance_char().unwrap(); // \n
    assert_eq!(navigator.line(), 2);
    assert_eq!(navigator.column(), 1);
}

#[test]
fn mixed_line_endings_handling() {
    let source = "line1\r\nline2\nline3\rline4";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    // Advance past line1
    for _ in 0..5 {
        navigator.advance_char().unwrap();
    }

    // Cross CRLF
    navigator.advance_char().unwrap(); // \r
    assert_eq!(navigator.line(), 2);
    navigator.advance_char().unwrap(); // \n (should not increment line again)
    assert_eq!(navigator.line(), 2);

    // Advance past line2
    for _ in 0..5 {
        navigator.advance_char().unwrap();
    }

    // Cross LF
    navigator.advance_char().unwrap(); // \n
    assert_eq!(navigator.line(), 3);

    // Advance past line3
    for _ in 0..5 {
        navigator.advance_char().unwrap();
    }

    // Cross CR
    navigator.advance_char().unwrap(); // \r
    assert_eq!(navigator.line(), 4);
}
