//! Delimiter coverage and navigation-access tests for [`AssTokenizer`].

use super::*;

#[cfg(not(feature = "std"))]
use hashbrown::HashSet;
#[cfg(feature = "std")]
use std::collections::HashSet;

#[test]
fn tokenizer_position_line_column_tracking() {
    let mut tokenizer = AssTokenizer::new("first\nsecond\nthird");

    // Initial position
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);

    // After first token
    let _token1 = tokenizer.next_token().unwrap().unwrap();
    let pos1 = tokenizer.position();
    let line1 = tokenizer.line();
    let _col1 = tokenizer.column();

    // After newline
    let _token2 = tokenizer.next_token().unwrap().unwrap(); // newline
    assert!(tokenizer.line() > line1); // Line should increment

    // Position should always advance unless at end
    let _token3 = tokenizer.next_token().unwrap().unwrap();
    assert!(tokenizer.position() > pos1);
}

#[test]
fn tokenizer_all_delimiter_types() {
    let mut tokenizer = AssTokenizer::new("[section]:value,field{override}text\n");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have various token types
    let types: HashSet<_> = tokens.iter().map(|t| &t.token_type).collect();

    // Should include multiple different types
    assert!(types.len() > 1);
    assert!(types.contains(&TokenType::SectionHeader) || types.contains(&TokenType::SectionOpen));
    assert!(types.contains(&TokenType::Colon));
    assert!(types.contains(&TokenType::Comma));
}

#[test]
fn tokenizer_empty_reset_state() {
    let mut tokenizer = AssTokenizer::new("");

    // Should handle empty input
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());

    // Reset should work on empty input
    tokenizer.reset();
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
}

#[test]
fn tokenizer_whitespace_handling_contexts() {
    // Test whitespace skipping in different contexts
    let mut tokenizer = AssTokenizer::new("   [   section   ]   ");

    // Document context should skip whitespace
    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(
        token1.token_type,
        TokenType::SectionHeader | TokenType::SectionOpen
    ));

    // Continue parsing
    let _remaining = tokenizer.tokenize_all().unwrap();
}

#[test]
fn tokenizer_issue_collection_access() {
    let mut tokenizer = AssTokenizer::new("valid content");

    // Initially no issues
    assert!(tokenizer.issues().is_empty());

    // After tokenizing
    let _tokens = tokenizer.tokenize_all().unwrap();
    let _issues = tokenizer.issues(); // Should be accessible

    // Reset should clear issues
    tokenizer.reset();
    assert!(tokenizer.issues().is_empty());
}

#[test]
fn tokenizer_scanner_navigation_access() {
    let mut tokenizer = AssTokenizer::new("test content");

    // Test that we can access navigator properties through public methods
    let initial_pos = tokenizer.position();
    let initial_line = tokenizer.line();
    let initial_col = tokenizer.column();

    assert_eq!(initial_pos, 0);
    assert_eq!(initial_line, 1);
    assert_eq!(initial_col, 1);

    // After tokenizing, positions should be accessible
    let _token = tokenizer.next_token().unwrap();
    let _new_pos = tokenizer.position();
    let _new_line = tokenizer.line();
    let _new_col = tokenizer.column();
}

#[test]
fn tokenizer_mixed_context_characters() {
    // Test various combinations that might cause context confusion
    let mut tokenizer = AssTokenizer::new("text{override[section]:value}more");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle mixed contexts without errors
    assert!(!tokens.is_empty());

    // Should have text tokens at minimum
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Text));
}

#[test]
fn tokenizer_semicolon_comment_in_document_context() {
    let mut tokenizer = AssTokenizer::new("; comment in document context");

    // Should recognize semicolon comment in Document context
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Comment);
}

#[test]
fn tokenizer_no_bom_content() {
    let mut tokenizer = AssTokenizer::new("content without BOM");
    assert_eq!(tokenizer.position(), 0); // Should start at 0 without BOM

    let _token = tokenizer.next_token().unwrap();
    assert!(tokenizer.position() > 0);
}

#[test]
fn tokenizer_infinite_loop_protection_error() {
    // Create a tokenizer that could potentially get stuck
    let source = "invalid_char\x00";
    let mut tokenizer = AssTokenizer::new(source);

    // Try to get next token - should handle the error gracefully
    match tokenizer.next_token() {
        Ok(_) | Err(_) => {
            // Both outcomes are acceptable as long as we don't infinite loop
            assert!(tokenizer.position() < source.len() || tokenizer.position() == source.len());
        }
    }
}
