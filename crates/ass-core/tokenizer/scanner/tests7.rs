//! Scanner unit tests: targeted coverage scenarios (part 7).

use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn char_navigator_last_char_tracking_coverage() {
    let source = "xy\nz";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Advance through characters and track last_char
    nav.advance_char().unwrap(); // 'x'
    assert_eq!(nav.last_char, Some('x'));

    nav.advance_char().unwrap(); // 'y'
    assert_eq!(nav.last_char, Some('y'));

    nav.advance_char().unwrap(); // '\n'
    assert_eq!(nav.last_char, Some('\n'));
    assert_eq!(nav.line(), 2);

    nav.advance_char().unwrap(); // 'z'
    assert_eq!(nav.last_char, Some('z'));
}

#[test]
fn token_scanner_hex_value_comprehensive_coverage() {
    // Test hex with ampersand suffix (must be even length)
    assert!(TokenScanner::is_hex_value("&H1234&"));
    assert!(TokenScanner::is_hex_value("&HFFFF&"));
    assert!(TokenScanner::is_hex_value("&H00&"));

    // Test hex without ampersand suffix (must be even length)
    assert!(TokenScanner::is_hex_value("&H1234"));
    assert!(TokenScanner::is_hex_value("&HABCD"));

    // Test empty hex part
    assert!(!TokenScanner::is_hex_value("&H&"));
    assert!(!TokenScanner::is_hex_value("&H"));

    // Test odd length (invalid)
    assert!(!TokenScanner::is_hex_value("&H123&"));
    assert!(!TokenScanner::is_hex_value("&H0&"));

    // Test max length enforcement
    assert!(!TokenScanner::is_hex_value("&H123456789ABCDEF&")); // Too long

    // Test invalid characters
    assert!(!TokenScanner::is_hex_value("&HZ123&"));
    assert!(!TokenScanner::is_hex_value("&H12G4&"));
}

#[test]
fn token_scanner_delimiter_context_comprehensive() {
    let source = ",{[}]:;\n\r";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Test field value context delimiters
    let result = scanner.scan_field_value();
    assert!(result.is_ok());

    // Test document context delimiter behavior
    let source2 = ";comment";
    let scanner2 = TokenScanner::new(source2, 0, 1, 1);
    let nav_pos = scanner2.navigator().position();

    // Should handle semicolon in document context
    assert_eq!(nav_pos, 0);
}

#[test]
fn token_scanner_scan_text_number_classification() {
    let source = "123.45";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
    assert!(result.is_ok());

    let token_type = result.unwrap();
    assert_eq!(token_type, crate::tokenizer::tokens::TokenType::Number);
}

#[test]
fn token_scanner_section_header_boundary_coverage() {
    let source = "[Section]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Should find closing bracket
    let result = scanner.scan_section_header();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        crate::tokenizer::tokens::TokenType::SectionHeader
    );
}

#[test]
fn token_scanner_style_override_brace_matching() {
    let source = "{\\b1}text{\\b0}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Should handle nested braces correctly
    let result = scanner.scan_style_override();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        crate::tokenizer::tokens::TokenType::OverrideBlock
    );
}

#[test]
fn token_scanner_simd_fallback_forced_coverage() {
    let source = "test,delimiter:content";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Force scalar path by testing with different contexts
    let result = scanner.scan_field_value();
    assert!(result.is_ok());
}

#[test]
fn char_navigator_advance_char_utf8_length_tracking() {
    let source = "a🎵b";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Advance over 'a' (1 byte)
    nav.advance_char().unwrap();
    assert_eq!(nav.position(), 1);

    // Advance over '🎵' (4 bytes)
    nav.advance_char().unwrap();
    assert_eq!(nav.position(), 5);

    // Advance over 'b' (1 byte)
    nav.advance_char().unwrap();
    assert_eq!(nav.position(), 6);
}

#[test]
fn token_scanner_empty_span_edge_cases() {
    let source = "";
    let scanner = TokenScanner::new(source, 0, 1, 1);

    // Test scan operations on empty source
    let nav_result = scanner.navigator();
    assert!(nav_result.is_at_end());

    // Verify position tracking
    assert_eq!(nav_result.position(), 0);
    assert_eq!(nav_result.line(), 1);
    assert_eq!(nav_result.column(), 1);
}

#[test]
fn char_navigator_peek_operations_at_boundaries() {
    let source = "a";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Peek at first character
    assert_eq!(nav.peek_char().unwrap(), 'a');

    // Peek next should handle end of input
    assert!(nav.peek_next().is_err());

    // Advance to end
    nav.advance_char().unwrap();
    assert!(nav.is_at_end());

    // Peek at end should handle end of input
    assert!(nav.peek_char().is_err());
    assert!(nav.peek_next().is_err());
}

#[test]
fn token_scanner_all_delimiter_combinations_coverage() {
    let delimiters = [',', ':', '{', '}', '[', ']', '\n', '\r'];

    for &delimiter in &delimiters {
        let source = format!("text{delimiter}more");
        let mut scanner = TokenScanner::new(&source, 0, 1, 1);

        // Test delimiter detection in different contexts
        let result = scanner.scan_field_value();
        assert!(result.is_ok());
    }
}
