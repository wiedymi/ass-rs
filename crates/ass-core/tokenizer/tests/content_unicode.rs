//! Content, unicode, debug, and clone tokenization tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn complex_script_structure() {
    let script =
        "[Script Info]\nTitle: Test\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";

    let mut tokenizer = AssTokenizer::new(script);
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have multiple tokens without errors
    assert!(tokens.len() > 5);

    // Verify basic token types are present
    let has_section = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::SectionHeader));
    let has_colon = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Colon));
    let has_comma = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Comma));
    let has_text = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Text));

    assert!(has_section);
    assert!(has_colon);
    assert!(has_comma);
    assert!(has_text);
}

#[test]
fn unicode_content() {
    let mut tokenizer = AssTokenizer::new("こんにちは世界");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "こんにちは世界");
}

#[test]
fn special_characters() {
    let mut tokenizer = AssTokenizer::new("Test with émojis 🎬 and symbols ©®™");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "Test with émojis 🎬 and symbols ©®™");
}

#[test]
fn carriage_return_only() {
    let mut tokenizer = AssTokenizer::new("line1\rline2");

    let _token1 = tokenizer.next_token().unwrap().unwrap();
    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Newline);
    assert_eq!(token2.span, "\r");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.span, "line2");
    assert_eq!(token3.line, 2);
}

#[test]
fn multiple_consecutive_newlines() {
    let mut tokenizer = AssTokenizer::new("line1\n\n\nline2");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have: text, newline, newline, newline, text
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].token_type, TokenType::Text);
    assert_eq!(tokens[1].token_type, TokenType::Newline);
    assert_eq!(tokens[2].token_type, TokenType::Newline);
    assert_eq!(tokens[3].token_type, TokenType::Newline);
    assert_eq!(tokens[4].token_type, TokenType::Text);

    assert_eq!(tokens[4].line, 4); // Should be on line 4
}

#[test]
fn nested_braces_in_text() {
    let mut tokenizer = AssTokenizer::new("Text with {nested {braces} inside}");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "Text with ");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::OverrideBlock);

    // Continue tokenizing to ensure no panic
    let _remaining_tokens = tokenizer.tokenize_all().unwrap();
}

#[test]
fn tokenizer_debug() {
    let tokenizer = AssTokenizer::new("test");
    let debug_str = format!("{tokenizer:?}");
    assert!(debug_str.contains("AssTokenizer"));
}

#[test]
fn tokenizer_clone() {
    let tokenizer = AssTokenizer::new("test content");
    let cloned = tokenizer.clone();
    assert_eq!(tokenizer.position(), cloned.position());
    assert_eq!(tokenizer.line(), cloned.line());
    assert_eq!(tokenizer.column(), cloned.column());
}
