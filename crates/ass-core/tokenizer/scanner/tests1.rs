//! Scanner unit tests: navigator basics and core scanning (part 1).

use super::*;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};

#[test]
fn char_navigator_new() {
    let source = "test content";
    let nav = CharNavigator::new(source, 5, 2, 3);
    assert_eq!(nav.position(), 5);
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 3);
}

#[test]
fn char_navigator_peek_char() {
    let source = "hello";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    assert_eq!(nav.peek_char().unwrap(), 'h');
    assert_eq!(nav.peek_char().unwrap(), 'h'); // Should not advance
    assert_eq!(nav.position(), 0);
}

#[test]
fn char_navigator_peek_next() {
    let source = "hello";
    let nav = CharNavigator::new(source, 0, 1, 1);
    assert_eq!(nav.peek_next().unwrap(), 'e');
}

#[test]
fn char_navigator_advance_char() {
    let source = "hello";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    assert_eq!(nav.advance_char().unwrap(), 'h');
    assert_eq!(nav.position(), 1);
    assert_eq!(nav.column(), 2);
}

#[test]
fn char_navigator_advance_newline() {
    let source = "line1\nline2";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    // Advance to newline
    for _ in 0..5 {
        nav.advance_char().unwrap();
    }
    assert_eq!(nav.advance_char().unwrap(), '\n');
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);
}

#[test]
fn char_navigator_advance_carriage_return() {
    let source = "line1\rline2";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    // Advance to carriage return
    for _ in 0..5 {
        nav.advance_char().unwrap();
    }
    assert_eq!(nav.advance_char().unwrap(), '\r');
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);
}

#[test]
fn char_navigator_advance_crlf() {
    let source = "line1\r\nline2";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    // Advance to \r
    for _ in 0..5 {
        nav.advance_char().unwrap();
    }
    assert_eq!(nav.advance_char().unwrap(), '\r');
    assert_eq!(nav.line(), 2);
    // Advance to \n
    assert_eq!(nav.advance_char().unwrap(), '\n');
    assert_eq!(nav.line(), 2); // Should not increment again
    assert_eq!(nav.column(), 1);
}

#[test]
fn char_navigator_skip_whitespace() {
    let source = "   \t  hello";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    nav.skip_whitespace();
    assert_eq!(nav.peek_char().unwrap(), 'h');
}

#[test]
fn char_navigator_skip_whitespace_preserves_newlines() {
    let source = "   \n  hello";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    nav.skip_whitespace();
    assert_eq!(nav.peek_char().unwrap(), '\n');
}

#[test]
fn char_navigator_is_at_end() {
    let source = "hi";
    let nav = CharNavigator::new(source, 2, 1, 1);
    assert!(nav.is_at_end());

    let nav2 = CharNavigator::new(source, 0, 1, 1);
    assert!(!nav2.is_at_end());
}

#[test]
fn char_navigator_peek_char_at_end() {
    let source = "hi";
    let mut nav = CharNavigator::new(source, 2, 1, 1);
    assert!(nav.peek_char().is_err());
}

#[test]
fn char_navigator_peek_next_at_end() {
    let source = "h";
    let nav = CharNavigator::new(source, 0, 1, 1);
    assert!(nav.peek_next().is_err());
}

#[test]
fn token_scanner_new() {
    let source = "test content";
    let scanner = TokenScanner::new(source, 5, 2, 3);
    assert_eq!(scanner.navigator().position(), 5);
    assert_eq!(scanner.navigator().line(), 2);
    assert_eq!(scanner.navigator().column(), 3);
}

#[test]
fn token_scanner_scan_section_header() {
    let source = "[Script Info]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_section_header().unwrap();
    assert_eq!(token_type, TokenType::SectionHeader);
}

#[test]
fn token_scanner_scan_style_override() {
    let source = "{\\b1\\i1}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_style_override().unwrap();
    assert_eq!(token_type, TokenType::OverrideBlock);
}

#[test]
fn token_scanner_scan_style_override_nested() {
    let source = "{\\b1{\\i1}\\b0}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_style_override().unwrap();
    assert_eq!(token_type, TokenType::OverrideBlock);
}

#[test]
fn token_scanner_scan_comment() {
    let source = "; This is a comment\nNext line";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_comment().unwrap();
    assert_eq!(token_type, TokenType::Comment);
}

#[test]
fn token_scanner_scan_text_basic() {
    let source = "Hello World,";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type, TokenType::Text);
}

#[test]
fn token_scanner_scan_text_number() {
    let source = "123.45,";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type, TokenType::Number);
}

#[test]
fn token_scanner_scan_text_hex_value() {
    let source = "&HABCDEF&,";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type, TokenType::HexValue);
}
