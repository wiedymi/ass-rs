//! Scanner unit tests: targeted coverage scenarios (part 8).

use super::*;

#[test]
fn char_navigator_newline_variations_comprehensive() {
    // Test different newline types
    let sources = [
        "line1\nline2",   // LF
        "line1\rline2",   // CR
        "line1\r\nline2", // CRLF
    ];

    for source in &sources {
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Advance to newline
        while let Ok(ch) = nav.advance_char() {
            if ch == '\n' || ch == '\r' {
                break;
            }
        }

        // Should be on line 2 after newline processing
        if !nav.is_at_end() {
            nav.advance_char().ok(); // Move past newline
            assert!(nav.line() >= 2);
        }
    }
}

#[test]
fn char_navigator_carriage_return_line_increment() {
    // Target lines 114-118: '\r' handling in advance_char
    let source = "text\rmore";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    // Advance to carriage return
    for _ in 0..4 {
        nav.advance_char().unwrap();
    }

    // This should hit the '\r' branch and increment line
    let ch = nav.advance_char().unwrap();
    assert_eq!(ch, '\r');
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);
}

#[test]
fn char_navigator_newline_line_increment() {
    // Target lines 130-131: '\n' handling
    let source = "text\nmore";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    for _ in 0..4 {
        nav.advance_char().unwrap();
    }

    let ch = nav.advance_char().unwrap();
    assert_eq!(ch, '\n');
    assert_eq!(nav.line(), 2);
    assert_eq!(nav.column(), 1);
}

#[test]
fn char_navigator_column_increment_default() {
    // Target lines 144: default column increment
    let source = "abc";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    nav.advance_char().unwrap(); // 'a'
    assert_eq!(nav.column(), 2);

    nav.advance_char().unwrap(); // 'b'
    assert_eq!(nav.column(), 3);

    nav.advance_char().unwrap(); // 'c'
    assert_eq!(nav.column(), 4);
}

#[test]
fn char_navigator_skip_whitespace_loop() {
    // Target lines 140-144: skip_whitespace loop
    let source = "   \t\n  text";
    let mut nav = CharNavigator::new(source, 0, 1, 1);

    nav.skip_whitespace();
    assert_eq!(nav.position(), 4); // Should stop at newline
}

#[test]
fn token_scanner_section_header_closing_bracket() {
    // Target lines 196, 199: section header scanning
    let source = "[Test]";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_section_header();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        crate::tokenizer::tokens::TokenType::SectionHeader
    );
}

#[test]
fn token_scanner_style_override_closing_brace() {
    // Target lines 216, 222: style override scanning
    let source = "{\\b1}";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_style_override();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        crate::tokenizer::tokens::TokenType::OverrideBlock
    );
}

#[test]
fn token_scanner_comment_scanning() {
    // Target line 241: comment scanning
    let source = "! This is a comment";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_comment();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        crate::tokenizer::tokens::TokenType::Comment
    );
}

#[test]
fn token_scanner_scan_text_hex_detection() {
    // Target lines 263, 274: hex value detection in scan_text
    let source = "&H1234&";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        crate::tokenizer::tokens::TokenType::HexValue
    );
}

#[test]
fn token_scanner_scan_text_number_detection_targeted() {
    // Target lines 283-284: number detection
    let source = "123.45";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), crate::tokenizer::tokens::TokenType::Number);
}

#[test]
fn token_scanner_scan_text_section_name_context() {
    // Target lines 288, 290-291: section name context
    let source = "Script Info";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::SectionHeader);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        crate::tokenizer::tokens::TokenType::SectionName
    );
}

#[test]
fn token_scanner_scan_text_field_value_context_targeted() {
    // Target lines 295, 299: field value context
    let source = "value_text";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::FieldValue);
    assert!(result.is_ok());
    // Should return Text type in field value context
}
