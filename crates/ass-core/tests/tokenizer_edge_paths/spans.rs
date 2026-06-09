//! Text fallback, span creation, and iteration tokenizer edge paths
//!
//! Covers the text fallback branch, position advancement validation, span
//! metadata, and the `tokenize_all` iteration boundary.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[test]
fn test_text_fallback_path() {
    // This should hit the text fallback path (line 137+ area)
    let input = "regular text without special chars";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
    assert_eq!(token.span, "regular text without special chars");
}

#[test]
fn test_position_advancement_check() {
    // This should test the position advancement validation (lines 166-172)
    let input = "a";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
    assert_eq!(token.span, "a");

    // Next call should return None (EOF)
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn test_token_span_creation() {
    // This should hit the span creation logic (lines 175-180)
    let input = "test";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.span, "test");
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
}

#[test]
fn test_tokenize_all_iteration_boundary() {
    // This should hit the tokenize_all method paths (lines 193-195)
    let input = "short";
    let mut tokenizer = AssTokenizer::new(input);

    let tokens = tokenizer.tokenize_all().unwrap();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].token_type, TokenType::Text));
}
