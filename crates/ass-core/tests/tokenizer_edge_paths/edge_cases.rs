//! Whitespace, empty-container, Unicode, and consecutive-delimiter edge paths
//!
//! Covers mixed whitespace, context-dependent character handling, empty
//! sections/overrides, Unicode boundaries, and consecutive delimiters.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[test]
fn test_mixed_whitespace_handling() {
    // Test various whitespace characters
    let input = " \t\n";
    let mut tokenizer = AssTokenizer::new(input);

    // Get first token (may be whitespace, text, or newline depending on tokenizer behavior)
    let token = tokenizer.next_token().unwrap().unwrap();
    // Accept any valid token type - the goal is to exercise tokenizer paths
    assert!(matches!(
        token.token_type,
        TokenType::Whitespace | TokenType::Text | TokenType::Newline
    ));

    // Get remaining tokens if any
    while let Some(_token) = tokenizer.next_token().unwrap() {
        // Just consume remaining tokens to exercise paths
    }
}

#[test]
fn test_context_dependent_character_handling() {
    // Test how same characters behave in different contexts
    let input = "text:value{override}";
    let mut tokenizer = AssTokenizer::new(input);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have multiple token types
    assert!(tokens.len() >= 4);
}

#[test]
fn test_empty_section_header() {
    // Test edge case of empty section
    let input = "[]";
    let mut tokenizer = AssTokenizer::new(input);

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token1.token_type, TokenType::SectionHeader));

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token2.token_type, TokenType::SectionClose));
}

#[test]
fn test_empty_style_override() {
    // Test edge case of empty override
    let input = "{}";
    let mut tokenizer = AssTokenizer::new(input);

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token1.token_type, TokenType::OverrideBlock));

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token2.token_type, TokenType::OverrideClose));
}

#[test]
fn test_unicode_boundary_handling() {
    // Test Unicode character boundaries
    let input = "café🎬";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
    assert!(token.span.contains("café"));
}

#[test]
fn test_consecutive_delimiters() {
    // Test multiple delimiters in sequence
    let input = "::,,";
    let mut tokenizer = AssTokenizer::new(input);

    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokens.len() >= 4); // Should have multiple delimiter tokens
}

#[test]
fn test_field_value_context_boundary() {
    // Test field value context transitions
    let input = "Key: Value\nNext: Line";
    let mut tokenizer = AssTokenizer::new(input);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have colon tokens that trigger context changes
    let has_colon = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Colon));
    let has_newline = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Newline));

    assert!(has_colon);
    assert!(has_newline);
}
