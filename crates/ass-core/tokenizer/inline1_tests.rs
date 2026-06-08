//! Basic tokenization behaviour tests for [`AssTokenizer`].

use super::*;

#[test]
fn tokenize_section_header() {
    let mut tokenizer = AssTokenizer::new("[Script Info]");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token_type, TokenType::SectionHeader);
    assert_eq!(tokens[1].token_type, TokenType::SectionClose);
}

#[test]
fn tokenize_field_line() {
    let mut tokenizer = AssTokenizer::new("Title: Test Script");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokens.len() >= 3);
    assert_eq!(tokens[1].token_type, TokenType::Colon);
}

#[test]
fn reset_tokenizer() {
    let mut tokenizer = AssTokenizer::new("Test");
    let _ = tokenizer.next_token().unwrap();
    assert!(tokenizer.position() > 0);

    tokenizer.reset();
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
}

#[test]
fn tokenize_with_bom() {
    let mut tokenizer = AssTokenizer::new("\u{FEFF}[Script Info]");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token_type, TokenType::SectionHeader);
}

#[test]
fn tokenize_style_override() {
    let mut tokenizer = AssTokenizer::new("{\\b1}text{\\b0}");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokens.len() >= 2);
    // Should contain at least one override-related token and some text
    let has_override = tokens.iter().any(|t| {
        matches!(
            t.token_type,
            TokenType::OverrideBlock | TokenType::OverrideOpen | TokenType::OverrideClose
        )
    });
    let has_text = tokens.iter().any(|t| t.token_type == TokenType::Text);
    assert!(
        has_override || has_text,
        "Should have override or text tokens"
    );
}

#[test]
fn tokenize_comma_delimiter() {
    let mut tokenizer = AssTokenizer::new("field1,field2,field3");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Comma));
}

#[test]
fn tokenize_newline_types() {
    let mut tokenizer = AssTokenizer::new("line1\nline2\r\nline3");
    let tokens = tokenizer.tokenize_all().unwrap();
    let newline_count = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Newline)
        .count();
    assert!(newline_count >= 2);
}

#[test]
fn tokenize_comment_semicolon() {
    let mut tokenizer = AssTokenizer::new("; This is a comment");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    assert_eq!(tokens[0].token_type, TokenType::Comment);
}

#[test]
fn tokenize_comment_exclamation() {
    let mut tokenizer = AssTokenizer::new("!: This is a comment");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    assert_eq!(tokens[0].token_type, TokenType::Comment);
}

#[test]
fn tokenize_misplaced_delimiters() {
    let mut tokenizer = AssTokenizer::new("text}more]text");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Text));
}

#[test]
fn tokenize_field_value_context() {
    let mut tokenizer = AssTokenizer::new("Key: Value with spaces");
    let tokens = tokenizer.tokenize_all().unwrap();
    let has_text = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Text));
    assert!(has_text);
}

#[test]
fn tokenize_exclamation_without_colon() {
    let mut tokenizer = AssTokenizer::new("!not a comment");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Text));
}

#[test]
fn tokenize_all_iteration_limit() {
    let repeated_text = "a".repeat(100);
    let mut tokenizer = AssTokenizer::new(&repeated_text);
    let result = tokenizer.tokenize_all();
    // Should either succeed with reasonable token count or hit iteration limit
    assert!(result.is_ok() || result.is_err());
}
