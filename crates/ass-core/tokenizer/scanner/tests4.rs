//! Scanner unit tests: field value and override comprehensives (part 4).

use super::*;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};

#[test]
fn token_scanner_scan_field_value_comprehensive() {
    // Test field value with colon (time format)
    let source1 = "0:01:30.50,next";
    let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
    let token_type1 = scanner1.scan_field_value().unwrap();
    assert_eq!(token_type1, TokenType::Number);
    assert_eq!(scanner1.navigator().position(), 10); // Stopped at comma

    // Test regular text field value
    let source2 = "Some text,next";
    let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
    let token_type2 = scanner2.scan_field_value().unwrap();
    assert_eq!(token_type2, TokenType::Text);
    assert_eq!(scanner2.navigator().position(), 9); // Stopped at comma

    // Test field value stopping at various delimiters
    let source3 = "text\nmore";
    let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
    let token_type3 = scanner3.scan_field_value().unwrap();
    assert_eq!(token_type3, TokenType::Text);
    assert_eq!(scanner3.navigator().position(), 4); // Stopped at newline

    let source4 = "text{override}";
    let mut scanner4 = TokenScanner::new(source4, 0, 1, 1);
    let token_type4 = scanner4.scan_field_value().unwrap();
    assert_eq!(token_type4, TokenType::Text);
    assert_eq!(scanner4.navigator().position(), 4); // Stopped at brace

    let source5 = "text[section]";
    let mut scanner5 = TokenScanner::new(source5, 0, 1, 1);
    let token_type5 = scanner5.scan_field_value().unwrap();
    assert_eq!(token_type5, TokenType::Text);
    assert_eq!(scanner5.navigator().position(), 4); // Stopped at bracket
}

#[test]
fn token_scanner_section_header_variations() {
    // Test section header with spaces
    let source1 = "[ Script Info ]";
    let mut scanner1 = TokenScanner::new(source1, 1, 1, 2); // Start after [
    let token_type1 = scanner1.scan_section_header().unwrap();
    assert_eq!(token_type1, TokenType::SectionHeader);

    // Test section header with special characters
    let source2 = "[V4+ Styles]";
    let mut scanner2 = TokenScanner::new(source2, 1, 1, 2); // Start after [
    let token_type2 = scanner2.scan_section_header().unwrap();
    assert_eq!(token_type2, TokenType::SectionHeader);

    // Test malformed section header (missing closing bracket)
    let source3 = "[Script Info\nNext line";
    let mut scanner3 = TokenScanner::new(source3, 1, 1, 2); // Start after [
    let token_type3 = scanner3.scan_section_header().unwrap();
    assert_eq!(token_type3, TokenType::SectionHeader);
}

#[test]
fn token_scanner_style_override_complex() {
    // Test nested braces
    let source1 = "{\\b1{nested}\\i1}";
    let mut scanner1 = TokenScanner::new(source1, 1, 1, 2); // Start after {
    let token_type1 = scanner1.scan_style_override().unwrap();
    assert_eq!(token_type1, TokenType::OverrideBlock);

    // Test override with no tags
    let source2 = "{ }";
    let mut scanner2 = TokenScanner::new(source2, 1, 1, 2); // Start after {
    let token_type2 = scanner2.scan_style_override().unwrap();
    assert_eq!(token_type2, TokenType::OverrideBlock);

    // Test unclosed override at end of line
    let source3 = "{\\b1\\i1\n";
    let mut scanner3 = TokenScanner::new(source3, 1, 1, 2); // Start after {
    let token_type3 = scanner3.scan_style_override().unwrap();
    assert_eq!(token_type3, TokenType::OverrideBlock);
}

#[test]
fn token_scanner_comment_variations() {
    // Test comment with exclamation
    let source1 = "!: This is a comment";
    let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
    let token_type1 = scanner1.scan_comment().unwrap();
    assert_eq!(token_type1, TokenType::Comment);

    // Test comment at end of file without newline
    let source2 = "; Comment";
    let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
    let token_type2 = scanner2.scan_comment().unwrap();
    assert_eq!(token_type2, TokenType::Comment);
    assert_eq!(scanner2.navigator().position(), source2.len());

    // Test empty comment
    let source3 = ";\n";
    let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
    let token_type3 = scanner3.scan_comment().unwrap();
    assert_eq!(token_type3, TokenType::Comment);
}

#[test]
fn token_scanner_unicode_handling() {
    // Test Unicode text
    let source1 = "中文测试";
    let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
    let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type1, TokenType::Text);
    assert_eq!(scanner1.navigator().position(), source1.len());

    // Test Unicode in section header
    let source2 = "[スクリプト情報]";
    let mut scanner2 = TokenScanner::new(source2, 1, 1, 2); // Start after [
    let token_type2 = scanner2.scan_section_header().unwrap();
    assert_eq!(token_type2, TokenType::SectionHeader);

    // Test emoji
    let source3 = "🎭🎬🎪";
    let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
    let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type3, TokenType::Text);
    assert_eq!(scanner3.navigator().position(), source3.len());
}

#[test]
fn token_scanner_empty_content_handling() {
    // Test empty text scan
    let source1 = ",next";
    let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
    let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type1, TokenType::Text);
    assert_eq!(scanner1.navigator().position(), 0); // No advancement for empty content

    // Test scan at end of input
    let source2 = "text";
    let mut scanner2 = TokenScanner::new(source2, 4, 1, 5); // Start at end
    let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
    assert_eq!(token_type2, TokenType::Text);
}

#[test]
fn char_navigator_boundary_conditions() {
    // Test very long line
    let long_line = "a".repeat(10000);
    let mut nav = CharNavigator::new(&long_line, 0, 1, 1);
    for i in 1..=10000 {
        nav.advance_char().unwrap();
        assert_eq!(nav.column(), i + 1);
    }
    assert!(nav.is_at_end());

    // Test many lines
    let many_lines = "a\n".repeat(1000);
    let mut nav2 = CharNavigator::new(&many_lines, 0, 1, 1);
    for i in 1..=1000 {
        nav2.advance_char().unwrap(); // 'a'
        nav2.advance_char().unwrap(); // '\n'
        assert_eq!(nav2.line(), i + 1);
        assert_eq!(nav2.column(), 1);
    }
}

#[test]
fn token_scanner_simd_fallback_coverage() {
    // Test contexts that should trigger scalar scanning even with SIMD
    let source = "field:value,next";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);
    let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
    assert_eq!(token_type, TokenType::Text);
    // Should stop at comma, not colon in field value context
    assert_eq!(scanner.navigator().position(), 11);
}
