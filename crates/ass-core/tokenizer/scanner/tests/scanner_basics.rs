//! Tests for `TokenScanner` creation, section headers, and navigator access.

use crate::tokenizer::scanner::TokenScanner;
use crate::tokenizer::state::TokenContext;
use crate::tokenizer::tokens::TokenType;

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
fn token_scanner_error_handling() {
    let source = "";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Trying to scan on empty source should handle gracefully
    let result = scanner.scan_text(TokenContext::Document);
    assert!(result.is_ok());
}
