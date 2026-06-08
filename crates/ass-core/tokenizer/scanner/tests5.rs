//! Scanner unit tests: caching, classification, and detection (part 5).

use super::*;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};
#[cfg(not(feature = "std"))]
use alloc::{format, vec};

#[test]
fn char_navigator_error_recovery() {
    // Test error handling when advancing past end
    let source = "a";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Advance to end
    nav.advance_char().unwrap();
    assert!(nav.is_at_end());

    // Further advances should fail
    assert!(nav.advance_char().is_err());
    assert!(nav.peek_char().is_err());
}

#[test]
fn char_navigator_peek_char_caching() {
    let source = "hello";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // First peek should cache the character
    assert_eq!(nav.peek_char().unwrap(), 'h');

    // Subsequent peeks should return cached value
    assert_eq!(nav.peek_char().unwrap(), 'h');
    assert_eq!(nav.peek_char().unwrap(), 'h');

    // After advancing, peek should work on next character
    nav.advance_char().unwrap();
    assert_eq!(nav.peek_char().unwrap(), 'e');
}

#[test]
fn char_navigator_last_char_tracking() {
    let source = "a\r\nb";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    nav.advance_char().unwrap(); // 'a'
    nav.advance_char().unwrap(); // '\r' - should increment line
    assert_eq!(nav.line(), 2);

    nav.advance_char().unwrap(); // '\n' - should NOT increment line again
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);
}

#[test]
fn token_scanner_hex_value_edge_cases() {
    // Raw hex without &H prefix is not detected to avoid conflicts
    assert!(!TokenScanner::is_hex_value("FF"));
    assert!(!TokenScanner::is_hex_value("00"));
    assert!(!TokenScanner::is_hex_value("ABCDEF"));
    assert!(!TokenScanner::is_hex_value("123456"));

    // Test &H prefix variations
    assert!(TokenScanner::is_hex_value("&HFF&"));
    assert!(TokenScanner::is_hex_value("&HFF"));
    assert!(TokenScanner::is_hex_value("&H00FF00FF&"));
    assert!(TokenScanner::is_hex_value("&H00FF00FF"));

    // Test invalid cases
    assert!(!TokenScanner::is_hex_value("F")); // Raw hex not detected
    assert!(!TokenScanner::is_hex_value("GG")); // Raw hex not detected
    assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
    assert!(!TokenScanner::is_hex_value("&HG&")); // Invalid hex with prefix
    assert!(!TokenScanner::is_hex_value("")); // Empty string
}

#[test]
fn scan_text_classification_verification() {
    // Verify that previously problematic cases now work correctly

    // Test case 1: "0:00:30.50" should be Text (timestamps are structured data, not numbers)
    let source1 = "0:00:30.50";
    let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
    let token_type1 = scanner1.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type1, TokenType::Text);

    // Test case 2: "123abc" should be Text (not hex without &H prefix)
    let source2 = "123abc";
    let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
    let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type2, TokenType::Text);

    // Test case 3: Proper hex with &H prefix should be HexValue
    let source3 = "&H00FF00&";
    let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
    let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type3, TokenType::HexValue);
}

#[test]
fn token_scanner_delimiter_combinations() {
    // Test complex delimiter combinations
    let source = "text:{}[],more";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type, TokenType::Text);
    assert_eq!(scanner.navigator().position(), 4); // Should stop at first delimiter ':'
}

#[test]
fn token_scanner_field_value_delimiter_handling() {
    // Test that colons are not delimiters in field value context
    let source = "0:00:30.50";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type, TokenType::Text); // Contains colons, so treated as text in scan_text
    assert_eq!(scanner.navigator().position(), source.len()); // Should consume entire string
}

#[test]
fn token_scanner_semicolon_context_sensitivity() {
    // Test semicolon handling in different contexts
    let source = "text;comment";

    // In Document context, semicolon behavior depends on SIMD vs scalar implementation
    let mut scanner1 = TokenScanner::new(source, 0, 1, 1);
    let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type1, TokenType::Text);
    // Position may vary based on SIMD feature and implementation details
    assert!(scanner1.navigator().position() >= 4);

    // In FieldValue context, should not stop at semicolon
    let mut scanner2 = TokenScanner::new(source, 0, 1, 1);
    let token_type2 = scanner2.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type2, TokenType::Text);
    assert_eq!(scanner2.navigator().position(), source.len());
}

#[test]
fn token_scanner_number_detection_edge_cases() {
    // Test various number formats
    let test_cases = vec![
        ("123", true),
        ("123.45", true),
        ("-123", true),
        ("-123.45", true),
        ("123.", true),
        (".45", true),
        ("-.45", true),
        ("123abc", false),  // Contains letters
        ("", false),        // Empty
        (".", true),        // Just decimal point
        ("-", true),        // Just minus
        ("--123", true),    // Still only contains valid number chars
        ("12.34.56", true), // Multiple decimals but still only valid chars
    ];

    for (input, expected_is_number) in test_cases {
        let source = format!("{input},");
        let mut scanner = TokenScanner::new(&source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::Document).unwrap();

        if expected_is_number && !input.is_empty() {
            assert_eq!(token_type, TokenType::Number, "Failed for input: {input}");
        } else {
            assert_ne!(token_type, TokenType::Number, "Failed for input: {input}");
        }
    }
}

#[test]
fn token_scanner_style_override_brace_depth() {
    // Test proper brace depth tracking
    let source = "{{{{}}}}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_style_override().unwrap();
    assert_eq!(token_type, TokenType::OverrideBlock);
    assert_eq!(scanner.navigator().position(), 7); // Should stop before final closing brace
}

#[test]
fn token_scanner_style_override_unbalanced() {
    // Test unbalanced braces
    let source = "{{{}}"; // Missing one closing brace
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_style_override().unwrap();
    assert_eq!(token_type, TokenType::OverrideBlock);
    // Should consume until end even if unbalanced
    assert_eq!(scanner.navigator().position(), source.len());
}
