//! Tests for `TokenScanner::scan_field_value` content scanning.

use crate::tokenizer::scanner::TokenScanner;
use crate::tokenizer::tokens::TokenType;

#[test]
fn scan_field_value_basic() {
    let source = "Test Script Title";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Text);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_field_value_with_colons() {
    let source = "0:00:30.50";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Number); // Time format should be recognized as number

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_field_value_stops_at_comma() {
    let source = "Value1,Value2";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), ',');
}

#[test]
fn scan_field_value_stops_at_newline() {
    let source = "Value\nNext line";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '\n');
}

#[test]
fn scan_field_value_stops_at_brace() {
    let source = "Text{override}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '{');
}

#[test]
fn scan_field_value_stops_at_bracket() {
    let source = "Text[section]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Text);

    assert_eq!(scanner.navigator_mut().peek_char().unwrap(), '[');
}

#[test]
fn scan_field_value_numeric_content() {
    let source = "123.45";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Number);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_field_value_negative_number() {
    let source = "-123.45";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Number);

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_field_value_empty() {
    let source = "";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Text); // Empty string is now correctly classified as Text

    assert!(scanner.navigator().is_at_end());
}

#[test]
fn scan_field_value_whitespace_only() {
    let source = "   ";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value().unwrap();
    assert_eq!(result, TokenType::Text); // Whitespace is not numeric

    assert!(scanner.navigator().is_at_end());
}
