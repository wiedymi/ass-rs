//! Newline reset and comment-detection tokenizer edge paths
//!
//! Covers carriage-return/newline context resets and semicolon/exclamation
//! comment detection paths.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[test]
fn test_newline_context_reset() {
    // This should hit line 128-132: ('\n' | '\r', _)
    let input = "line1\nline2";
    let mut tokenizer = AssTokenizer::new(input);

    // Skip first line text
    let _line1 = tokenizer.next_token().unwrap().unwrap();

    // This newline should trigger the context reset path
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Newline));
}

#[test]
fn test_carriage_return_context_reset() {
    // This should hit the '\r' part of line 128: ('\n' | '\r', _)
    let input = "line1\rline2";
    let mut tokenizer = AssTokenizer::new(input);

    // Skip first line text
    let _line1 = tokenizer.next_token().unwrap().unwrap();

    // This carriage return should trigger the context reset path
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Newline));
}

#[test]
fn test_semicolon_comment_detection() {
    // This should hit comment detection paths
    let input = "; This is a comment";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Comment));
}

#[test]
fn test_exclamation_comment_detection() {
    // This should hit exclamation comment detection paths
    let input = "!: This is also a comment";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Comment));
}
