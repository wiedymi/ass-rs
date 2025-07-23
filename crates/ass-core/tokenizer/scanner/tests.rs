//! Comprehensive tests for tokenizer scanner functionality

use super::*;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};

#[test]
fn char_navigator_creation() {
    let source = "Hello World";
    let navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.position(), 0);
    assert_eq!(navigator.line(), 1);
    assert_eq!(navigator.column(), 1);
    assert!(!navigator.is_at_end());
}

#[test]
fn char_navigator_advance() {
    let source = "abc";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_char().unwrap(), 'a');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 1);
    assert_eq!(navigator.column(), 2);

    assert_eq!(navigator.peek_char().unwrap(), 'b');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 2);
    assert_eq!(navigator.column(), 3);

    assert_eq!(navigator.peek_char().unwrap(), 'c');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 3);
    assert_eq!(navigator.column(), 4);

    assert!(navigator.is_at_end());
}

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
fn char_navigator_peek_next() {
    let source = "abc";
    let navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_next().unwrap(), 'b');
    assert_eq!(navigator.position(), 0); // Position should not change
}

#[test]
fn char_navigator_peek_at_end() {
    let source = "";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert!(navigator.is_at_end());
    let result = navigator.peek_char();
    assert!(result.is_err());
}

#[test]
fn char_navigator_advance_at_end() {
    let source = "a";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    navigator.advance_char().unwrap(); // Consume 'a'
    assert!(navigator.is_at_end());

    let result = navigator.advance_char();
    assert!(result.is_err());
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
fn char_navigator_unicode_characters() {
    let source = "こんにちは世界";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_char().unwrap(), 'こ');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.peek_char().unwrap(), 'ん');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.peek_char().unwrap(), 'に');

    // Position should advance by correct byte count for UTF-8
    assert!(navigator.position() > 2); // Each Japanese char is 3 bytes in UTF-8
}

#[test]
fn char_navigator_emoji() {
    let source = "🎬📽️🎭";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_char().unwrap(), '🎬');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.column(), 2);

    // Should handle emoji correctly
    assert_eq!(navigator.peek_char().unwrap(), '📽');
}

#[test]
fn token_scanner_creation() {
    let source = "test content";
    let scanner = TokenScanner::new(source, 0, 1, 1);

    assert_eq!(scanner.navigator().position(), 0);
    assert_eq!(scanner.navigator().line(), 1);
    assert_eq!(scanner.navigator().column(), 1);
}

#[test]
fn token_scanner_section_header() {
    let source = "[Script Info]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_section_header().unwrap();
    assert_eq!(result, TokenType::SectionHeader);

    // Should have consumed everything except the closing bracket
    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ']');
}

#[test]
fn token_scanner_section_header_with_spaces() {
    let source = "[ V4+ Styles ]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_section_header().unwrap();
    assert_eq!(result, TokenType::SectionHeader);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ']');
}

#[test]
fn token_scanner_section_header_empty() {
    let source = "[]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_section_header().unwrap();
    assert_eq!(result, TokenType::SectionHeader);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ']');
}

#[test]
fn token_scanner_section_header_no_close() {
    let source = "[Script Info";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_section_header().unwrap();
    assert_eq!(result, TokenType::SectionHeader);

    // Should have consumed all content when no closing bracket
    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_style_override_simple() {
    let source = "{\\b1}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_style_override().unwrap();
    assert_eq!(result, TokenType::OverrideBlock);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '}');
}

#[test]
fn token_scanner_style_override_complex() {
    let source = "{\\c&H0000FF&\\fs20}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_style_override().unwrap();
    assert_eq!(result, TokenType::OverrideBlock);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '}');
}

#[test]
fn token_scanner_style_override_empty() {
    let source = "{}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_style_override().unwrap();
    assert_eq!(result, TokenType::OverrideBlock);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '}');
}

#[test]
fn token_scanner_style_override_no_close() {
    let source = "{\\b1";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_style_override().unwrap();
    assert_eq!(result, TokenType::OverrideBlock);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_comment_semicolon() {
    let source = "; This is a comment";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_comment().unwrap();
    assert_eq!(result, TokenType::Comment);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_comment_exclamation() {
    let source = "!: This is also a comment";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_comment().unwrap();
    assert_eq!(result, TokenType::Comment);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_comment_with_newline() {
    let source = "; Comment\nNext line";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_comment().unwrap();
    assert_eq!(result, TokenType::Comment);

    // Should stop at newline
    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '\n');
}

#[test]
fn token_scanner_text_simple() {
    let source = "Hello World";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_text_with_delimiters() {
    let source = "Hello:World";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    // Should stop at colon
    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ':');
}

#[test]
fn token_scanner_text_field_value() {
    let source = "Test Script Title";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_text_empty() {
    let source = "";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_text_whitespace_only() {
    let source = "   \t  ";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_text_unicode() {
    let source = "こんにちは世界";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_text_with_special_chars() {
    let source = "Test with émojis 🎬 and symbols ©®™";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn token_scanner_text_stops_at_brace() {
    let source = "Text{override}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '{');
}

#[test]
fn token_scanner_text_stops_at_comma() {
    let source = "Value1,Value2";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ',');
}

#[test]
fn token_scanner_text_stops_at_newline() {
    let source = "Line1\nLine2";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '\n');
}

#[test]
fn token_scanner_navigator_access() {
    let source = "test";
    let scanner = TokenScanner::new(source, 0, 1, 1);

    // Test immutable access
    let navigator = scanner.navigator();
    assert_eq!(navigator.position(), 0);
}

#[test]
fn token_scanner_navigator_mut_access() {
    let source = "test";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Test mutable access
    let navigator = scanner.navigator_mut();
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 1);
}

#[test]
fn token_scanner_multiple_scans() {
    let source = "[Section]\nField: Value";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Scan section header
    let result1 = scanner.scan_section_header().unwrap();
    assert_eq!(result1, TokenType::SectionHeader);

    // Skip closing bracket manually
    scanner.navigator_mut().advance_char().unwrap();

    // Skip newline manually
    scanner.navigator_mut().advance_char().unwrap();

    // Scan field name
    let result2 = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result2, TokenType::Text);

    // Should be at colon now
    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ':');
}

#[test]
fn char_navigator_peek_with_lookahead() {
    let source = "abc";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    // First peek should set up lookahead
    assert_eq!(navigator.peek_char().unwrap(), 'a');

    // Second peek should return same character
    assert_eq!(navigator.peek_char().unwrap(), 'a');

    // Advance should use the lookahead
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 1);

    // Next peek should be next character
    assert_eq!(navigator.peek_char().unwrap(), 'b');
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
fn token_scanner_error_handling() {
    let source = "";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Trying to scan on empty source should handle gracefully
    let result = scanner.scan_text(TokenContext::Document);
    assert!(result.is_ok());
}

#[test]
fn char_navigator_bounds_checking() {
    let source = "a";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert!(!navigator.is_at_end());
    navigator.advance_char().unwrap();
    assert!(navigator.is_at_end());

    // Further attempts should fail gracefully
    let result = navigator.peek_char();
    assert!(result.is_err());

    let result = navigator.advance_char();
    assert!(result.is_err());
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
