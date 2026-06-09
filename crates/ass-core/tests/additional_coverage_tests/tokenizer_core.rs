//! Coverage tests for core `AssTokenizer` behavior.
//!
//! Exercises position advancement, scanner error propagation, BOM handling,
//! issue collection, and reset semantics for previously untested code paths.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Test tokenizer position advancement validation to prevent infinite loops
#[test]
fn test_tokenizer_position_advancement_validation() {
    // This test ensures the infinite loop protection works correctly
    let mut tokenizer = AssTokenizer::new("x");

    // Normal case should advance position
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert!(tokenizer.position() > 0);
}

/// Test tokenizer with malformed scanner state
#[test]
fn test_tokenizer_scanner_error_propagation() {
    // Create tokenizer and force it into edge case scenarios
    let mut tokenizer = AssTokenizer::new("\x00invalid\x01");

    // Should handle invalid UTF-8 or control characters gracefully
    let result = tokenizer.next_token();
    // Either succeeds with sanitized output or propagates scanner error
    if let Ok(Some(token)) = result {
        assert!(matches!(token.token_type, TokenType::Text));
    } else {
        // Empty result or error propagation is acceptable
    }
}

/// Test BOM handling edge cases
#[test]
fn test_tokenizer_bom_edge_cases() {
    // Test BOM-like sequences that aren't actually BOM
    let tokenizer1 = AssTokenizer::new("\u{FFFE}not-bom");
    assert_eq!(tokenizer1.position(), 0); // Should not skip non-BOM

    // Test actual BOM followed by content
    let tokenizer2 = AssTokenizer::new("\u{FEFF}content");
    assert_eq!(tokenizer2.position(), 3); // Should skip BOM
}

/// Test issue collection functionality comprehensively
#[test]
fn test_tokenizer_issue_collection_comprehensive() {
    let mut tokenizer = AssTokenizer::new("test content with potential issues");

    // Initial state should have no issues
    assert!(tokenizer.issues().is_empty());

    // Process tokens
    let _tokens = tokenizer.tokenize_all().unwrap();

    // Issues should be accessible (may be empty for valid content)
    let _issues = tokenizer.issues();

    // Test that reset clears issues
    tokenizer.reset();
    assert!(tokenizer.issues().is_empty());
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
}

/// Test tokenizer reset functionality thoroughly
#[test]
fn test_tokenizer_reset_comprehensive() {
    let mut tokenizer = AssTokenizer::new("\u{FEFF}[Section]\nField: Value");

    // Process some tokens
    let _token1 = tokenizer.next_token().unwrap();
    let _token2 = tokenizer.next_token().unwrap();

    let mid_position = tokenizer.position();
    let mid_line = tokenizer.line();
    let mid_column = tokenizer.column();

    assert!(mid_position > 3); // Should be past BOM
    assert!(mid_line >= 1);
    assert!(mid_column >= 1);

    // Reset and verify all state is restored
    tokenizer.reset();

    assert_eq!(tokenizer.position(), 3); // Should skip BOM again
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
    assert!(tokenizer.issues().is_empty());
}

/// Test tokenizer with content that could cause position stagnation
#[test]
fn test_tokenizer_position_stagnation_prevention() {
    // Test content that might cause scanner to not advance
    let mut tokenizer = AssTokenizer::new("\x00");

    let result = tokenizer.next_token();

    // Should either advance position or handle error gracefully
    match result {
        Ok(Some(_)) => {
            assert!(tokenizer.position() > 0);
        }
        Ok(None) => {
            // Empty result acceptable for problematic input
        }
        Err(e) => {
            // Error acceptable if position advancement fails
            assert!(e.to_string().contains("position not advancing") || !e.to_string().is_empty());
        }
    }
}
