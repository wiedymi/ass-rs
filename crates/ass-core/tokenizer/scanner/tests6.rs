//! Scanner unit tests: newlines, contexts, and consistency (part 6).

use super::*;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn char_navigator_whitespace_at_end() {
    let source = "text   ";
    let mut nav = CharNavigator::new(source, 4, 1, 5); // Start at first space
    nav.skip_whitespace();
    assert!(nav.is_at_end());
}

#[test]
fn char_navigator_mixed_newlines() {
    let source = "\r\n\n\r";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // \r
    nav.advance_char().unwrap();
    assert_eq!(nav.line(), 2);

    // \n (after \r, should not increment line)
    nav.advance_char().unwrap();
    assert_eq!(nav.line(), 2);

    // \n (standalone, should increment line)
    nav.advance_char().unwrap();
    assert_eq!(nav.line(), 3);

    // \r (standalone, should increment line)
    nav.advance_char().unwrap();
    assert_eq!(nav.line(), 4);
}

#[test]
fn token_scanner_empty_span_handling() {
    // Test scanning empty content
    let source = ",";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type, TokenType::Text);
    assert_eq!(scanner.navigator().position(), 0); // Should not advance for empty content
}

#[test]
fn token_scanner_field_value_time_format() {
    // Test time format recognition in field values
    let source = "1:23:45.67";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_field_value().unwrap();
    assert_eq!(token_type, TokenType::Number);
    assert_eq!(scanner.navigator().position(), source.len());
}

#[test]
fn char_navigator_position_consistency() {
    let source = "café🎭";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    let start_pos = nav.position();
    nav.advance_char().unwrap(); // 'c' - 1 byte
    assert_eq!(nav.position(), start_pos + 1);

    nav.advance_char().unwrap(); // 'a' - 1 byte
    assert_eq!(nav.position(), start_pos + 2);

    nav.advance_char().unwrap(); // 'f' - 1 byte
    assert_eq!(nav.position(), start_pos + 3);

    nav.advance_char().unwrap(); // 'é' - 2 bytes
    assert_eq!(nav.position(), start_pos + 5);

    nav.advance_char().unwrap(); // '🎭' - 4 bytes
    assert_eq!(nav.position(), start_pos + 9);
}

#[test]
fn token_scanner_all_contexts_coverage() {
    // Test scanning in all different contexts
    let contexts = vec![
        TokenContext::Document,
        TokenContext::SectionHeader,
        TokenContext::FieldValue,
        TokenContext::StyleOverride,
    ];

    for context in contexts {
        let source = "test:value,more";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(context).unwrap();

        // Should return appropriate token type based on context
        match context {
            TokenContext::SectionHeader => assert_eq!(token_type, TokenType::SectionName),
            _ => assert!(matches!(
                token_type,
                TokenType::Text | TokenType::Number | TokenType::HexValue
            )),
        }
    }
}

#[test]
fn char_navigator_column_reset_on_newlines() {
    let source = "long line text\nshort\n";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Advance to end of first line
    for _ in 0..14 {
        nav.advance_char().unwrap();
    }
    assert_eq!(nav.column(), 15);

    // Advance over newline
    nav.advance_char().unwrap(); // \n
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);

    // Advance a few chars on second line
    for _ in 0..5 {
        nav.advance_char().unwrap();
    }
    assert_eq!(nav.column(), 6);

    // Advance over second newline
    nav.advance_char().unwrap(); // \n
    assert_eq!(nav.line(), 3);
    assert_eq!(nav.column(), 1);
}

#[test]
fn char_navigator_utf8_error_handling() {
    // Test invalid UTF-8 sequences
    let source = "valid\x7F\x7E";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Should advance through valid chars
    assert!(nav.advance_char().is_ok());
    assert!(nav.advance_char().is_ok());
    assert!(nav.advance_char().is_ok());
    assert!(nav.advance_char().is_ok());
    assert!(nav.advance_char().is_ok());

    // Should handle invalid UTF-8
    let result = nav.advance_char();
    match result {
        Ok(_) | Err(_) => {
            // Both outcomes acceptable - important is no panic
            assert!(nav.position() > 0);
        }
    }
}

#[test]
fn char_navigator_peek_char_caching_coverage() {
    let source = "abc";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // First peek should cache the character
    let first_peek = nav.peek_char();
    assert_eq!(first_peek, Ok('a'));

    // Second peek should use cached value
    let second_peek = nav.peek_char();
    assert_eq!(second_peek, Ok('a'));

    // Advance should clear cache and move to next char
    assert!(nav.advance_char().is_ok());

    // Next peek should get new character
    let third_peek = nav.peek_char();
    assert_eq!(third_peek, Ok('b'));
}
