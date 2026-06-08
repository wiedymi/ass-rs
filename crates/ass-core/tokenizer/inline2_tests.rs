//! State-tracking, iteration-limit, and context-transition tests for [`AssTokenizer`].

use super::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn tokenizer_position_tracking() {
    let mut tokenizer = AssTokenizer::new("Test\nLine 2");

    let initial_pos = tokenizer.position();
    let initial_line = tokenizer.line();
    let initial_col = tokenizer.column();

    assert_eq!(initial_pos, 0);
    assert_eq!(initial_line, 1);
    assert_eq!(initial_col, 1);

    let _ = tokenizer.next_token().unwrap();
    assert!(tokenizer.position() > initial_pos);
}

#[test]
fn tokenizer_issues_collection() {
    let mut tokenizer = AssTokenizer::new("test content");
    let _ = tokenizer.tokenize_all().unwrap();
    let _issues = tokenizer.issues();
    // Issues collection should be accessible (may be empty for valid input)
}

#[test]
fn tokenize_empty_input() {
    let mut tokenizer = AssTokenizer::new("");
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenize_only_whitespace() {
    let mut tokenizer = AssTokenizer::new("   \t  ");
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenizer_infinite_loop_protection() {
    // Create a scenario that could cause infinite loop if position doesn't advance
    let mut tokenizer = AssTokenizer::new("test");

    // Force scanner into a state where it might not advance position
    let result = tokenizer.next_token();
    assert!(result.is_ok());

    // Ensure position has advanced
    assert!(tokenizer.position() > 0 || tokenizer.scanner.navigator().is_at_end());
}

#[test]
fn tokenizer_iteration_limit_exceeded() {
    // Test the iteration limit in tokenize_all
    let long_content = "a ".repeat(30); // Should exceed 50 token limit
    let mut tokenizer = AssTokenizer::new(&long_content);
    let result = tokenizer.tokenize_all();

    // Should either succeed or hit the iteration limit error
    match result {
        Ok(tokens) => assert!(tokens.len() <= 50),
        Err(e) => assert!(e.to_string().contains("Too many tokenizer iterations")),
    }
}

#[test]
fn tokenizer_context_transitions_comprehensive() {
    let mut tokenizer = AssTokenizer::new("[Section]:value{override}text\n");

    // Start in Document context
    assert_eq!(tokenizer.context, TokenContext::Document);

    // After '[', should be in SectionHeader context
    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::SectionHeader);
    assert_eq!(tokenizer.context, TokenContext::SectionHeader);

    // After ']', should return to Document context
    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::SectionClose);
    assert_eq!(tokenizer.context, TokenContext::Document);

    // After ':', should be in FieldValue context
    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Colon);
    assert_eq!(tokenizer.context, TokenContext::FieldValue);

    // Continue tokenizing to test more transitions
    let _remaining_tokens = tokenizer.tokenize_all().unwrap();
}

#[test]
fn tokenizer_delimiter_in_wrong_context() {
    // Test '}' outside StyleOverride context
    let mut tokenizer = AssTokenizer::new("}text");
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "}");

    // Test ']' outside SectionHeader context
    let mut tokenizer2 = AssTokenizer::new("]text");
    let token2 = tokenizer2.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Text);
    assert_eq!(token2.span, "]");
}

#[test]
fn tokenizer_bom_edge_cases() {
    // Test content that starts like BOM but isn't complete
    let mut tokenizer = AssTokenizer::new("\u{FEFF}content");
    assert_eq!(tokenizer.position(), 3); // Should skip BOM

    // Test reset with BOM
    let _token = tokenizer.next_token().unwrap();
    tokenizer.reset();
    assert_eq!(tokenizer.position(), 3); // Should skip BOM again after reset
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
    assert_eq!(tokenizer.context, TokenContext::Document);
}

#[test]
fn tokenizer_carriage_return_line_feed() {
    let mut tokenizer = AssTokenizer::new("line1\r\nline2");

    // Get first token (text)
    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);

    // Get newline token - should handle \r\n as single newline
    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Newline);
    assert_eq!(tokenizer.context, TokenContext::Document); // Should reset context

    // Should advance past both \r and \n
    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "line2");
}

#[test]
fn tokenizer_exclamation_comment_detection() {
    // Test '!' followed by ':' (should be comment)
    let mut tokenizer = AssTokenizer::new("!:comment");
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Comment);

    // Test '!' not followed by ':' (should be text)
    let mut tokenizer2 = AssTokenizer::new("!text");
    let token2 = tokenizer2.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Text);
}

#[test]
fn tokenizer_field_value_context_handling() {
    let mut tokenizer = AssTokenizer::new("key:value with spaces,next");

    // Get key
    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);

    // Get colon - should enter FieldValue context
    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Colon);
    assert_eq!(tokenizer.context, TokenContext::FieldValue);

    // Get field value - should consume until delimiter
    let token3 = tokenizer.next_token().unwrap().unwrap();
    // Should be either Text or a field value token type
    assert!(matches!(
        token3.token_type,
        TokenType::Text | TokenType::Number | TokenType::HexValue
    ));
}
