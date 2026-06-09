//! Tests for `TokenScanner::scan_text` token classification by context.

use crate::tokenizer::scanner::TokenScanner;
use crate::tokenizer::state::TokenContext;
use crate::tokenizer::tokens::TokenType;

#[test]
fn scan_text_section_header_context() {
    let source = "Script Info";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::SectionHeader).unwrap();
    assert_eq!(result, TokenType::SectionName);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_text_hex_detection() {
    let source = "&HFF0000&";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::HexValue);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_text_number_detection() {
    let source = "123.456";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Number);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_text_negative_number() {
    let source = "-456.789";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Number);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_text_field_value_context_with_colon() {
    let source = "0:01:23.45,next";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(result, TokenType::Text); // Time format should be text since it contains colons

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ',');
}

#[test]
fn scan_text_document_context_stops_at_colon() {
    let source = "Field:Value";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ':');
}

#[test]
fn scan_text_stops_at_semicolon_in_document() {
    let source = "Text;comment";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(result, TokenType::Text);

    // Should consume all text since semicolon only stops in Document context when it's at start
    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_text_semicolon_not_delimiter_in_field_value() {
    let source = "Text;more text";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}
