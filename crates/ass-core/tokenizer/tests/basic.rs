//! Basic construction and core token-type tokenization tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[test]
fn tokenizer_new_without_bom() {
    let tokenizer = AssTokenizer::new("Hello World");
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
}

#[test]
fn tokenizer_new_with_bom() {
    let tokenizer = AssTokenizer::new("\u{FEFF}Hello World");
    assert_eq!(tokenizer.position(), 3); // BOM is 3 bytes
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
}

#[test]
fn tokenize_empty_string() {
    let mut tokenizer = AssTokenizer::new("");
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenize_whitespace_only() {
    let mut tokenizer = AssTokenizer::new("   \t  ");
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenize_section_header_basic() {
    let mut tokenizer = AssTokenizer::new("[Script Info]");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::SectionHeader);
    assert_eq!(token1.span, "[Script Info");
    assert_eq!(token1.line, 1);
    assert_eq!(token1.column, 1);

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::SectionClose);
    assert_eq!(token2.span, "]");

    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenize_section_header_with_spaces() {
    let mut tokenizer = AssTokenizer::new("[ V4+ Styles ]");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::SectionHeader);
    assert_eq!(token1.span, "[ V4+ Styles ");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::SectionClose);
}

#[test]
fn tokenize_field_with_colon() {
    let mut tokenizer = AssTokenizer::new("Title: Test Script");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "Title");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Colon);
    assert_eq!(token2.span, ":");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, " Test Script");
}

#[test]
fn tokenize_style_override_simple() {
    let mut tokenizer = AssTokenizer::new("{\\b1}");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::OverrideBlock);
    assert_eq!(token1.span, "{\\b1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::OverrideClose);
    assert_eq!(token2.span, "}");
}

#[test]
fn tokenize_style_override_complex() {
    let mut tokenizer = AssTokenizer::new("{\\c&H0000FF&\\fs20}Hello{\\r}");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::OverrideBlock);

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::OverrideClose);

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "Hello");

    let token4 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token4.token_type, TokenType::OverrideBlock);

    let token5 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token5.token_type, TokenType::OverrideClose);
}
