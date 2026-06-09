//! Edge-case and malformed-input tokenization tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use hashbrown::HashSet;

#[test]
fn field_value_context() {
    let mut tokenizer = AssTokenizer::new("Key: Value with spaces and, commas");

    let _key = tokenizer.next_token().unwrap().unwrap();
    let _colon = tokenizer.next_token().unwrap().unwrap();

    // After colon, should be in field value context
    // Field values stop at commas (CSV-style delimiter)
    let value = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(value.token_type, TokenType::Text);
    assert_eq!(value.span, " Value with spaces and");

    // Next token should be the comma delimiter
    let comma = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(comma.token_type, TokenType::Comma);
}

#[test]
fn malformed_section_header() {
    let mut tokenizer = AssTokenizer::new("[Unclosed section header");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    // Should still tokenize something rather than panic
}

#[test]
fn malformed_style_override() {
    let mut tokenizer = AssTokenizer::new("{\\unclosed override");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    // Should still tokenize something rather than panic
}

#[test]
fn very_long_content() {
    let long_text = "A".repeat(10000);
    let mut tokenizer = AssTokenizer::new(&long_text);
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span.len(), 10000);
}

#[test]
fn only_delimiters() {
    let mut tokenizer = AssTokenizer::new("{}[]:,\n");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have tokens for each delimiter
    assert!(tokens.len() >= 6);
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::OverrideBlock)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::OverrideClose)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::SectionHeader)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::SectionClose)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Colon)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Comma)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Newline)));
}

#[test]
fn alternating_contexts() {
    let mut tokenizer = AssTokenizer::new("[Section]field:value{\\override}text[Another]");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle rapid context changes without errors
    assert!(tokens.len() >= 8);

    // Verify we have different token types
    let token_types: HashSet<_> = tokens.iter().map(|t| &t.token_type).collect();
    assert!(token_types.len() >= 4); // At least 4 different token types
}

#[test]
fn whitespace_variations() {
    let inputs = [
        " \t spaces and tabs",
        "\u{00A0}non-breaking space",
        "\u{2000}en quad",
        "\u{3000}ideographic space",
    ];

    for input in &inputs {
        let mut tokenizer = AssTokenizer::new(input);
        let tokens = tokenizer.tokenize_all().unwrap();
        assert!(!tokens.is_empty());
    }
}

#[test]
fn control_characters() {
    let mut tokenizer = AssTokenizer::new("text\x00\x01\x02control");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    // Should handle control characters without panic
}

#[test]
fn single_character_tokens() {
    let inputs = ["{", "}", "[", "]", ":", ",", ";", "!"];

    for &input in &inputs {
        let mut tokenizer = AssTokenizer::new(input);
        let result = tokenizer.next_token().unwrap();
        assert!(result.is_some());
    }
}

#[test]
fn mixed_line_endings() {
    let mut tokenizer = AssTokenizer::new("line1\nline2\r\nline3\rline4");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have multiple newline tokens with different styles
    let newlines: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Newline)
        .collect();
    assert_eq!(newlines.len(), 3);
    assert_eq!(newlines[0].span, "\n");
    assert_eq!(newlines[1].span, "\r\n");
    assert_eq!(newlines[2].span, "\r");
}

#[test]
fn empty_tokens_edge_case() {
    let mut tokenizer = AssTokenizer::new("::,,{}{}\n\n");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle consecutive delimiters
    assert!(tokens.len() >= 6);
}
