//! Tests for `TokenScanner::scan_text` content scanning and delimiters.

use crate::tokenizer::scanner::TokenScanner;
use crate::tokenizer::state::TokenContext;
use crate::tokenizer::tokens::TokenType;

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
