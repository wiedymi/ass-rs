//! Scanner unit tests: field values, hex detection, and delimiters (part 2).

use super::*;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};
#[cfg(not(feature = "std"))]
use alloc::{format, vec};

#[test]
fn token_scanner_scan_text_section_name() {
    let source = "Script Info]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::SectionHeader).unwrap();
    assert_eq!(token_type, TokenType::SectionName);
}

#[test]
fn token_scanner_scan_text_field_value_context() {
    let source = "0:01:23.45,";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type, TokenType::Text);
}

#[test]
fn token_scanner_scan_field_value() {
    let source = "Some field value,";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_field_value().unwrap();
    assert_eq!(token_type, TokenType::Text);
}

#[test]
fn token_scanner_scan_field_value_number() {
    let source = "0:01:23.45,";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_field_value().unwrap();
    assert_eq!(token_type, TokenType::Number);
}

#[test]
fn token_scanner_is_hex_value_simple() {
    // Raw hex without &H prefix is not detected to avoid conflicts
    assert!(!TokenScanner::is_hex_value("ABCD"));
    assert!(!TokenScanner::is_hex_value("1234"));

    // With &H prefix, hex values are detected
    assert!(TokenScanner::is_hex_value("&HABCD&"));
    assert!(TokenScanner::is_hex_value("&H1234&"));
    assert!(!TokenScanner::is_hex_value("&HABCDE&")); // Odd length
    assert!(!TokenScanner::is_hex_value("&HGHIJ&")); // Invalid hex
    assert!(!TokenScanner::is_hex_value("")); // Empty
}

#[test]
fn token_scanner_is_hex_value_with_prefix() {
    assert!(TokenScanner::is_hex_value("&HFF00FF&"));
    assert!(TokenScanner::is_hex_value("&HFF00FF"));
    assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
    assert!(!TokenScanner::is_hex_value("&HGHIJ&")); // Invalid hex
}

#[test]
fn token_scanner_is_hex_value_max_length() {
    // Test that hex values have a reasonable maximum length limit
    // Raw hex without &H prefix is not detected anymore
    assert!(!TokenScanner::is_hex_value("ABCDEF")); // 6 chars - no prefix
    assert!(!TokenScanner::is_hex_value("00FF00FF")); // 8 chars - no prefix
    assert!(!TokenScanner::is_hex_value("1234567890")); // 10 chars - too long
    assert!(!TokenScanner::is_hex_value(&"A".repeat(100))); // Very long - not hex value

    // Test with &H prefix format (more permissive)
    assert!(TokenScanner::is_hex_value("&H00FF00FF&")); // 8 hex chars - valid
    assert!(TokenScanner::is_hex_value("&HABCD&")); // 4 hex chars - valid with prefix
    assert!(!TokenScanner::is_hex_value("&H1234567890&")); // 10 hex chars - too long

    // Test that numbers/text are not detected as hex
    assert!(!TokenScanner::is_hex_value("00")); // No prefix
    assert!(!TokenScanner::is_hex_value("123abc")); // No prefix
}

#[test]
fn token_scanner_hex_value_trailing_ampersand_variants() {
    // Test hex values with trailing ampersand
    assert!(TokenScanner::is_hex_value("&H00FFFFFF&"));
    assert!(TokenScanner::is_hex_value("&HFF0000&"));
    assert!(TokenScanner::is_hex_value("&H80FF00FF&"));

    // Test hex values without trailing ampersand (common in ASS files)
    assert!(TokenScanner::is_hex_value("&H00FFFFFF"));
    assert!(TokenScanner::is_hex_value("&HFF0000"));
    assert!(TokenScanner::is_hex_value("&H80FF00FF"));

    // Test edge cases
    assert!(TokenScanner::is_hex_value("&H00&"));
    assert!(TokenScanner::is_hex_value("&H00"));
    assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
    assert!(!TokenScanner::is_hex_value("&H")); // No hex part
}

#[test]
fn token_scanner_scan_text_hex_value_ampersand_variants() {
    // Test hex value with trailing ampersand
    let source1 = "&H00FFFFFF&";
    let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
    let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type1, TokenType::HexValue);
    assert_eq!(scanner1.navigator().position(), source1.len());

    // Test hex value without trailing ampersand (common in ASS files)
    let source2 = "&H00FFFFFF";
    let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
    let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type2, TokenType::HexValue);
    assert_eq!(scanner2.navigator().position(), source2.len());

    // Test short hex value variants
    let source3 = "&HFF00&";
    let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
    let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type3, TokenType::HexValue);

    let source4 = "&HFF00";
    let mut scanner4 = TokenScanner::new(source4, 0, 1, 1);
    let token_type4 = scanner4.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type4, TokenType::HexValue);
}

#[test]
fn token_scanner_delimiter_context_field_value() {
    let source = "Title: My Script";
    let mut scanner = TokenScanner::new(source, 7, 1, 8); // Start after "Title: "
    let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type, TokenType::Text);
    // Should have consumed "My Script" without stopping at colon
}

#[test]
fn token_scanner_delimiter_context_document() {
    let source = "Field:Value";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type, TokenType::Text);
    // Should stop at the colon, so only "Field" is consumed
    assert_eq!(scanner.navigator().position(), 5);
}

#[test]
fn token_scanner_various_delimiters() {
    let test_cases = vec![
        (",", TokenContext::Document),
        ("{", TokenContext::Document),
        ("}", TokenContext::Document),
        ("[", TokenContext::Document),
        ("]", TokenContext::Document),
        ("\n", TokenContext::Document),
        ("\r", TokenContext::Document),
    ];

    for (delimiter, context) in test_cases {
        let source = format!("text{delimiter}more");
        let mut scanner = TokenScanner::new(&source, 0, 1, 1);
        let _token_type = scanner.scan_text(context).unwrap();
        assert_eq!(scanner.navigator().position(), 4); // Should stop at delimiter
    }
}

#[test]
fn token_scanner_navigator_mut() {
    let source = "test";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    {
        let nav_mut = scanner.navigator_mut();
        nav_mut.advance_char().unwrap();
    }
    assert_eq!(scanner.navigator().position(), 1);
}

#[test]
fn char_navigator_utf8_handling() {
    let source = "café";
    let mut nav = CharNavigator::new(source, 0, 1, 1);
    assert_eq!(nav.advance_char().unwrap(), 'c');
    assert_eq!(nav.advance_char().unwrap(), 'a');
    assert_eq!(nav.advance_char().unwrap(), 'f');
    assert_eq!(nav.advance_char().unwrap(), 'é');
    assert_eq!(nav.position(), 5); // 'é' is 2 bytes in UTF-8
}
