//! Unit tests for the [`Token`] type.

use super::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn token_creation() {
    let span = "test";
    let token = Token::new(TokenType::Text, span, 1, 5);

    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "test");
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 5);
    assert_eq!(token.len(), 4);
    assert_eq!(token.end_column(), 9);
}

#[test]
fn token_empty_check() {
    let empty_token = Token::new(TokenType::Text, "", 1, 1);
    assert!(empty_token.is_empty());

    let normal_token = Token::new(TokenType::Text, "text", 1, 1);
    assert!(!normal_token.is_empty());
}

#[test]
fn token_classification() {
    let text_token = Token::new(TokenType::Text, "hello", 1, 1);
    assert!(text_token.is_content());
    assert!(!text_token.is_delimiter());
    assert!(!text_token.is_whitespace());

    let comma_token = Token::new(TokenType::Comma, ",", 1, 6);
    assert!(comma_token.is_delimiter());
    assert!(!comma_token.is_content());

    let ws_token = Token::new(TokenType::Whitespace, " ", 1, 7);
    assert!(ws_token.is_whitespace());
    assert!(!ws_token.is_content());
}

#[test]
fn token_display() {
    let token = Token::new(TokenType::Text, "hello", 2, 5);
    let display = format!("{token}");
    assert!(display.contains("Text"));
    assert!(display.contains("2:5"));
    assert!(display.contains("hello"));
}

#[test]
fn token_utf8_validation() {
    let token = Token::new(TokenType::Text, "valid utf8", 1, 1);
    assert!(token.validate_utf8());

    let unicode_token = Token::new(TokenType::Text, "🎵", 1, 1);
    assert!(unicode_token.validate_utf8());
}

#[test]
fn token_unicode_length() {
    // Test token with Unicode characters
    let unicode_token = Token::new(TokenType::Text, "🎵🎶🎤", 1, 1);
    assert_eq!(unicode_token.len(), 3); // 3 Unicode characters
    assert_eq!(unicode_token.end_column(), 4); // column 1 + 3 chars

    // Test token with mixed ASCII and Unicode
    let mixed_token = Token::new(TokenType::Text, "hello🎵world", 1, 1);
    assert_eq!(mixed_token.len(), 11); // 5 + 1 + 5 characters
    assert_eq!(mixed_token.end_column(), 12);

    // Test empty span
    let empty_token = Token::new(TokenType::Text, "", 1, 1);
    assert_eq!(empty_token.len(), 0);
    assert_eq!(empty_token.end_column(), 1);
}

#[test]
fn token_comprehensive_classification() {
    // Test instance methods match TokenType methods
    let text_token = Token::new(TokenType::Text, "text", 1, 1);
    assert_eq!(text_token.is_content(), TokenType::Text.is_content());
    assert_eq!(text_token.is_delimiter(), TokenType::Text.is_delimiter());

    let comma_token = Token::new(TokenType::Comma, ",", 1, 1);
    assert_eq!(comma_token.is_content(), TokenType::Comma.is_content());
    assert_eq!(comma_token.is_delimiter(), TokenType::Comma.is_delimiter());

    let whitespace_token = Token::new(TokenType::Whitespace, " ", 1, 1);
    assert_eq!(
        whitespace_token.is_whitespace(),
        matches!(TokenType::Whitespace, TokenType::Whitespace)
    );
}

#[test]
fn token_equality_and_cloning() {
    let token1 = Token::new(TokenType::Text, "test", 1, 5);
    let token2 = Token::new(TokenType::Text, "test", 1, 5);
    let token3 = Token::new(TokenType::Number, "test", 1, 5);
    let token4 = Token::new(TokenType::Text, "different", 1, 5);

    assert_eq!(token1, token2);
    assert_ne!(token1, token3);
    assert_ne!(token1, token4);

    let cloned = token1.clone();
    assert_eq!(token1, cloned);
}

#[test]
fn token_debug_formatting() {
    let token = Token::new(TokenType::SectionHeader, "[Script Info]", 2, 1);
    let debug_output = format!("{token:?}");
    assert!(debug_output.contains("SectionHeader"));
    assert!(debug_output.contains("[Script Info]"));
    assert!(debug_output.contains("line: 2"));
    assert!(debug_output.contains("column: 1"));
}
