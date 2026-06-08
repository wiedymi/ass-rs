//! Scanner unit tests: edge cases and number detection (part 3).

use super::*;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};

#[test]
fn token_scanner_empty_section_header() {
    let source = "[]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_section_header().unwrap();
    assert_eq!(token_type, TokenType::SectionHeader);
}

#[test]
fn token_scanner_unclosed_section_header() {
    let source = "[Script Info";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_section_header().unwrap();
    assert_eq!(token_type, TokenType::SectionHeader);
}

#[test]
fn token_scanner_empty_style_override() {
    let source = "{}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_style_override().unwrap();
    assert_eq!(token_type, TokenType::OverrideBlock);
}

#[test]
fn token_scanner_unclosed_style_override() {
    let source = "{\\b1\\i1";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_style_override().unwrap();
    assert_eq!(token_type, TokenType::OverrideBlock);
}

#[test]
fn token_scanner_comment_at_end() {
    let source = "; Comment at end";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_comment().unwrap();
    assert_eq!(token_type, TokenType::Comment);
}

#[test]
fn char_navigator_advance_char_error_handling() {
    let mut nav = CharNavigator::new("", 0, 1, 1);
    assert!(nav.advance_char().is_err());
    assert!(nav.peek_char().is_err());
    assert!(nav.peek_next().is_err());
}

#[test]
fn char_navigator_peek_operations_edge_cases() {
    let source = "a";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Test peek_char caching
    assert_eq!(nav.peek_char().unwrap(), 'a');
    assert_eq!(nav.peek_char().unwrap(), 'a'); // Should return cached value

    // Test peek_next at boundaries
    assert!(nav.peek_next().is_err()); // Only one character

    nav.advance_char().unwrap();
    assert!(nav.peek_char().is_err()); // At end
    assert!(nav.peek_next().is_err()); // At end
}

#[test]
fn char_navigator_line_column_tracking_complex() {
    let source = "line1\r\nline2\rline3\nline4";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Test \r\n handling
    for _ in "line1".chars() {
        nav.advance_char().unwrap();
    }
    assert_eq!(nav.line(), 1);
    assert_eq!(nav.column(), 6);

    nav.advance_char().unwrap(); // \r
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);

    nav.advance_char().unwrap(); // \n (shouldn't increment line again)
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);

    // Test standalone \r
    for _ in "line2".chars() {
        nav.advance_char().unwrap();
    }
    nav.advance_char().unwrap(); // \r
    assert_eq!(nav.line(), 3);
    assert_eq!(nav.column(), 1);

    // Test standalone \n
    for _ in "line3".chars() {
        nav.advance_char().unwrap();
    }
    nav.advance_char().unwrap(); // \n
    assert_eq!(nav.line(), 4);
    assert_eq!(nav.column(), 1);
}

#[test]
fn char_navigator_skip_whitespace_variations() {
    let source = " \t\u{00A0}\u{2000} text"; // Various whitespace types
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    nav.skip_whitespace();
    assert_eq!(nav.peek_char().unwrap(), 't');

    // Test with newlines (should not skip)
    let source2 = "  \n  text";
    let mut nav2 = CharNavigator::new(source2, 0, 1, 1);
    nav2.skip_whitespace();
    assert_eq!(nav2.peek_char().unwrap(), '\n');
}

#[test]
fn token_scanner_scan_text_field_value_context_edge_cases() {
    // Test colon in field values (should not be delimiter)
    let source = "0:00:30.50";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type, TokenType::Text);
    assert_eq!(scanner.navigator().position(), source.len());

    // Test various delimiters in field value context
    let source2 = "text,next";
    let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
    let token_type2 = scanner2.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type2, TokenType::Text);
    assert_eq!(scanner2.navigator().position(), 4); // Stopped at comma
}

#[test]
fn token_scanner_scan_text_number_detection() {
    // Test positive number
    let source1 = "123.45";
    let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
    let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type1, TokenType::Number);

    // Test negative number
    let source2 = "-123.45";
    let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
    let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type2, TokenType::Number);

    // Test integer
    let source3 = "123";
    let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
    let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type3, TokenType::Number);

    // Test text (contains letters)
    let source4 = "123abc";
    let mut scanner4 = TokenScanner::new(source4, 0, 1, 1);
    let token_type4 = scanner4.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type4, TokenType::Text);
}
