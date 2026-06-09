//! Tests for `TokenScanner` style override blocks and comment scanning.

use crate::tokenizer::scanner::TokenScanner;
use crate::tokenizer::tokens::TokenType;

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
fn token_scanner_style_override_nested_braces() {
    let source = "{\\t({\\an8})\\b1}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_style_override().unwrap();
    assert_eq!(result, TokenType::OverrideBlock);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '}');
}

#[test]
fn token_scanner_style_override_multiple_levels() {
    let source = "{{inner}outer}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_style_override().unwrap();
    assert_eq!(result, TokenType::OverrideBlock);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '}');
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
