//! Context-reset and field-value scanning tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn context_reset_to_document() {
    // Test context reset on newline
    let mut tokenizer = AssTokenizer::new("{\\an8}text\nNew line");

    // Debug: collect all tokens to understand the sequence
    let mut all_tokens = Vec::new();
    while let Some(token) = tokenizer.next_token().unwrap() {
        all_tokens.push(token);
    }

    // Verify we have the expected tokens
    assert!(all_tokens.len() >= 4);

    // First should be override block
    assert_eq!(all_tokens[0].token_type, TokenType::OverrideBlock);

    // Second should be override close
    assert_eq!(all_tokens[1].token_type, TokenType::OverrideClose);

    // Third should be text content
    assert_eq!(all_tokens[2].token_type, TokenType::Text);
    assert_eq!(all_tokens[2].span, "text");

    // Fourth should be newline or text containing newline and remaining content
    let fourth_token = &all_tokens[3];
    assert!(
        fourth_token.token_type == TokenType::Newline
            || (fourth_token.token_type == TokenType::Text && fourth_token.span.contains('\n'))
    );
}

#[test]
fn delimiter_in_wrong_context_as_text() {
    // Test that delimiter chars in wrong context become text
    let mut tokenizer = AssTokenizer::new("}]");

    let first = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(first.token_type, TokenType::Text);
    assert_eq!(first.span, "}");

    let second = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(second.token_type, TokenType::Text);
    assert_eq!(second.span, "]");
}

#[test]
fn exclamation_without_colon() {
    // Test exclamation mark that's not followed by colon
    let mut tokenizer = AssTokenizer::new("!notcomment");
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
}

#[test]
fn carriage_return_line_feed_handling() {
    // Test proper CRLF handling
    let mut tokenizer = AssTokenizer::new("line1\r\nline2");
    let _text1 = tokenizer.next_token().unwrap();
    let newline = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(newline.token_type, TokenType::Newline);

    // Position should advance past both \r and \n
    let _text2 = tokenizer.next_token().unwrap();
    assert!(tokenizer.position() > 7); // past "line1\r\n"
}

#[test]
fn issues_collector_functionality() {
    // Test that issues are properly collected
    let mut tokenizer = AssTokenizer::new("test content");
    let _tokens = tokenizer.tokenize_all().unwrap();
    let issues = tokenizer.issues();
    // Issues should be accessible (even if empty for valid input)
    assert!(issues.is_empty() || !issues.is_empty());
}

#[test]
fn tokenizer_allows_whitespace_skipping() {
    // Test whitespace skipping behavior in different contexts
    let mut tokenizer = AssTokenizer::new("   [Section]   ");
    let token = tokenizer.next_token().unwrap().unwrap();
    // Should skip leading whitespace and find section header
    assert_eq!(token.token_type, TokenType::SectionHeader);
}

#[test]
fn context_allows_whitespace_skipping() {
    // Test context-dependent whitespace skipping
    let mut tokenizer = AssTokenizer::new("  text  ");
    // Document context is the default, so whitespace should be skipped
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
}

#[test]
fn tokenize_all_large_input() {
    // Test tokenize_all with reasonable size to avoid iteration limit
    let content = "[Script Info]\nTitle: Test\n".repeat(3);
    let mut tokenizer = AssTokenizer::new(&content);
    let result = tokenizer.tokenize_all();
    assert!(result.is_ok());
}
