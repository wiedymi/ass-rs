//! Scanner unit tests: targeted coverage scenarios (part 9).

use super::*;

#[test]
fn token_scanner_scan_text_default_case() {
    // Target line 305: default text case
    let source = "regular_text";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), crate::tokenizer::tokens::TokenType::Text);
}

#[test]
fn token_scanner_is_hex_value_ampersand_suffix() {
    // Target lines 324, 326, 328: hex value validation
    assert!(TokenScanner::is_hex_value("&H1234&"));
    assert!(TokenScanner::is_hex_value("&HABCD&"));
    assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
}

#[test]
fn token_scanner_is_hex_value_no_ampersand() {
    // Target line 339: hex without trailing ampersand
    assert!(TokenScanner::is_hex_value("&H1234"));
    assert!(TokenScanner::is_hex_value("&HABCD"));
}

#[test]
fn token_scanner_scan_field_value_basic() {
    // Target lines 381, 386: scan_field_value
    let source = "field_value,next";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_field_value();
    assert!(result.is_ok());
}

#[test]
fn char_navigator_peek_char_error_path() {
    // Target lines 76, 79, 81-82: peek_char error handling
    let source = "a";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Advance past end
    nav.advance_char().unwrap();
    assert!(nav.is_at_end());

    // peek_char should return error at end
    let result = nav.peek_char();
    assert!(result.is_err());
}

#[test]
fn char_navigator_peek_next_error_path() {
    // Target lines 108, 110-111, 113: peek_next error handling
    let source = "a";
    let nav = CharNavigator::new(source, 0, 1, 1);

    // peek_next from last character should error
    let result = nav.peek_next();
    assert!(result.is_err());
}
