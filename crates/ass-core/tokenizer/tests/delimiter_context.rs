//! Delimiter-context and mixed-content tokenization tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use hashbrown::HashSet;

#[test]
fn bom_position_calculation() {
    // Test BOM handling in position calculation
    let content_with_bom = "\u{FEFF}[Script Info]";
    let mut tokenizer = AssTokenizer::new(content_with_bom);

    // Initial position should skip BOM
    assert_eq!(tokenizer.position(), 3);

    // Reset should also handle BOM correctly
    tokenizer.reset();
    assert_eq!(tokenizer.position(), 3);
}

#[test]
fn scanner_navigator_access() {
    // Test navigator access methods
    let tokenizer = AssTokenizer::new("test");
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
    assert_eq!(tokenizer.position(), 0);
}

#[test]
fn section_close_in_wrong_context() {
    // Test section close outside of section header context
    let mut tokenizer = AssTokenizer::new("]");
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
}

#[test]
fn override_close_in_wrong_context() {
    // Test override close outside of style override context
    let mut tokenizer = AssTokenizer::new("}");
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
}

#[test]
fn field_value_scanning() {
    // Test field value context scanning
    let mut tokenizer = AssTokenizer::new("Key: Value with spaces");

    let _key = tokenizer.next_token().unwrap();
    let _colon = tokenizer.next_token().unwrap();
    let value = tokenizer.next_token().unwrap().unwrap();

    // Should capture entire field value
    assert!(value.span.contains("Value"));
}

#[test]
fn consecutive_delimiters() {
    // Test handling of consecutive delimiter characters
    let mut tokenizer = AssTokenizer::new(",,::{}[]");
    let mut tokens = Vec::new();

    while let Some(token) = tokenizer.next_token().unwrap() {
        tokens.push(token);
    }

    // Should handle each delimiter appropriately
    assert!(!tokens.is_empty());
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Comma));
}

#[test]
fn mixed_delimiters_and_text() {
    // Test mixed content with various delimiters
    let mut tokenizer = AssTokenizer::new("text,more{override}[section]:value");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have variety of token types
    let types: HashSet<_> = tokens.iter().map(|t| &t.token_type).collect();
    assert!(types.len() > 1);
}

#[test]
fn tokenizer_error_handling_edge_cases() {
    // Test error handling for various edge cases
    let mut tokenizer = AssTokenizer::new("test\x00invalid");

    // Should handle null bytes gracefully
    let result = tokenizer.next_token();
    assert!(result.is_ok());
}
