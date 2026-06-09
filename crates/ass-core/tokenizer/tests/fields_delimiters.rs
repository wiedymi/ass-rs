//! Field, delimiter, newline, and comment tokenization tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[test]
fn tokenize_comma_separator() {
    let mut tokenizer = AssTokenizer::new("value1,value2,value3");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "value1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Comma);
    assert_eq!(token2.span, ",");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "value2");

    let token4 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token4.token_type, TokenType::Comma);

    let token5 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token5.token_type, TokenType::Text);
    assert_eq!(token5.span, "value3");
}

#[test]
fn tokenize_newline_unix() {
    let mut tokenizer = AssTokenizer::new("line1\nline2");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "line1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Newline);
    assert_eq!(token2.span, "\n");
    assert_eq!(token2.line, 1);

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "line2");
    assert_eq!(token3.line, 2);
}

#[test]
fn tokenize_newline_windows() {
    let mut tokenizer = AssTokenizer::new("line1\r\nline2");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "line1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Newline);
    assert_eq!(token2.span, "\r\n");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "line2");
    assert_eq!(token3.line, 2);
}

#[test]
fn tokenize_comment_semicolon() {
    let mut tokenizer = AssTokenizer::new("; This is a comment");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Comment);
    assert_eq!(token.span, "; This is a comment");
}

#[test]
fn tokenize_comment_exclamation_colon() {
    let mut tokenizer = AssTokenizer::new("!: This is also a comment");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Comment);
    assert_eq!(token.span, "!: This is also a comment");
}

#[test]
fn tokenize_exclamation_not_comment() {
    let mut tokenizer = AssTokenizer::new("!Not a comment");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "!Not a comment");
}

#[test]
fn tokenize_mixed_content() {
    let mut tokenizer = AssTokenizer::new("[Section]\nField: Value\n; Comment\n{\\b1}Text");
    let tokens = tokenizer.tokenize_all().unwrap();

    assert!(tokens.len() >= 8);
    assert_eq!(tokens[0].token_type, TokenType::SectionHeader);
    assert_eq!(tokens[1].token_type, TokenType::SectionClose);
    assert_eq!(tokens[2].token_type, TokenType::Newline);
    assert_eq!(tokens[3].token_type, TokenType::Text);
    assert_eq!(tokens[4].token_type, TokenType::Colon);
    // And so on...
}
