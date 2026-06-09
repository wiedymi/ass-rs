//! Tokenizer coverage tests for tokenizer state and API surface.
//!
//! Exercises EOF handling, position advancement, span boundaries, reset,
//! issue collection, position tracking, bulk tokenization, malformed input,
//! and multiline line/column tracking.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[test]
fn test_empty_input_eof() {
    let input = "";
    let mut tokenizer = AssTokenizer::new(input);

    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn test_position_advancement_validation() {
    // This should test the infinite loop protection
    let input = "normal text";
    let mut tokenizer = AssTokenizer::new(input);

    let mut token_count = 0;
    while let Some(_token) = tokenizer.next_token().unwrap() {
        token_count += 1;
        assert!(
            token_count <= 100,
            "Too many tokens, possible infinite loop"
        );
    }

    assert!(token_count > 0);
}

#[test]
fn test_token_span_boundaries() {
    let input = "test";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.span, "test");
    assert!(token.line > 0);
    assert!(token.column > 0);
}

#[test]
fn test_tokenizer_reset_functionality() {
    let input = "test input";
    let mut tokenizer = AssTokenizer::new(input);

    // Get first token
    let first_token = tokenizer.next_token().unwrap().unwrap();

    // Reset tokenizer
    tokenizer.reset();

    // Should get same first token again
    let reset_token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(first_token.span, reset_token.span);
}

#[test]
fn test_tokenizer_issues_collection() {
    let input = "test input";
    let tokenizer = AssTokenizer::new(input);

    // Should start with no issues
    assert!(tokenizer.issues().is_empty());
}

#[test]
fn test_tokenizer_position_tracking() {
    let input = "test";
    let tokenizer = AssTokenizer::new(input);

    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
}

#[test]
fn test_tokenize_all_method() {
    let input = "test input";
    let mut tokenizer = AssTokenizer::new(input);

    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());

    // All tokens should have valid spans
    for token in &tokens {
        assert!(!token.span.is_empty());
    }
}

#[test]
fn test_malformed_section_handling() {
    let input = "[Unclosed Section\nNext line";
    let mut tokenizer = AssTokenizer::new(input);

    // Should handle unclosed section gracefully
    let mut token_count = 0;
    while let Some(_token) = tokenizer.next_token().unwrap() {
        token_count += 1;
        if token_count > 10 {
            break; // Prevent infinite loop in malformed input
        }
    }

    assert!(token_count > 0);
}

#[test]
fn test_line_column_tracking_multiline() {
    let input = "line1\nline2\nline3";
    let mut tokenizer = AssTokenizer::new(input);

    // First token should be at line 1
    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.line, 1);

    // Skip to newline
    while let Some(token) = tokenizer.next_token().unwrap() {
        if matches!(token.token_type, TokenType::Newline) {
            break;
        }
    }

    // Next text token should be at line 2
    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.line, 2);
}
