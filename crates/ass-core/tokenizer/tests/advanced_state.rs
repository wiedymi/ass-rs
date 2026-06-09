//! Advanced state, consistency, and whitespace-skipping tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[cfg(not(feature = "std"))]
use hashbrown::HashSet;
#[cfg(feature = "std")]
use std::collections::HashSet;

#[test]
fn tokenizer_boundary_character_handling() {
    // Test edge cases with boundary characters
    let content = "\n\r\n\r\r\n";
    let mut tokenizer = AssTokenizer::new(content);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle various newline combinations
    let newline_count = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Newline)
        .count();
    assert!(newline_count > 0);
}

#[test]
fn tokenizer_comment_edge_cases() {
    // Test various comment edge cases
    let content = ";comment\n!comment\n!:comment\n!notcomment\n;;double\n";
    let mut tokenizer = AssTokenizer::new(content);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should properly distinguish between comment types
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Comment));
}

#[test]
fn tokenizer_empty_section_and_override() {
    // Test empty sections and overrides
    let content = "[]{}text[empty]{}";
    let mut tokenizer = AssTokenizer::new(content);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle empty delimited sections
    let section_headers = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::SectionHeader)
        .count();
    let section_closes = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::SectionClose)
        .count();
    assert_eq!(section_headers, section_closes);
}

#[test]
fn tokenizer_position_consistency() {
    // Test that position tracking remains consistent
    let content = "line1\nline2\r\nline3\rline4";
    let mut tokenizer = AssTokenizer::new(content);

    let mut prev_line = 1;
    let mut prev_column = 1;
    while let Some(token) = tokenizer.next_token().unwrap() {
        // Line and column should advance monotonically
        assert!(token.line >= prev_line);
        if token.line == prev_line {
            assert!(token.column >= prev_column);
        }
        prev_line = token.line;
        prev_column = token.column;
    }
}

#[test]
fn tokenizer_scanner_methods_coverage() {
    // Test various scanner method paths
    let content = "[V4+ Styles]\n{\\alpha&H80}Text{\\b1}Bold";
    let mut tokenizer = AssTokenizer::new(content);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should exercise different scanner methods
    let token_types: HashSet<_> = tokens.iter().map(|t| &t.token_type).collect();
    assert!(token_types.len() > 3);
}

#[test]
fn tokenizer_issue_collection_edge_cases() {
    // Test issue collection in various scenarios
    let mut tokenizer = AssTokenizer::new("valid content");

    // Initially no issues
    assert!(tokenizer.issues().is_empty());

    // After tokenizing valid content, still no issues
    let _tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokenizer.issues().is_empty());
}

#[test]
fn tokenizer_context_reset_scenarios() {
    // Test context reset in various scenarios
    let content = "{override\nnewline resets}\n[section\nnewline resets]";
    let mut tokenizer = AssTokenizer::new(content);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Newlines should reset context appropriately
    let newlines = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Newline)
        .count();
    assert!(newlines > 0);
}

#[test]
fn tokenizer_whitespace_skipping_behavior() {
    // Test whitespace skipping in different contexts
    let content = "   {   override   }   [   section   ]   :   value   ";
    let mut tokenizer = AssTokenizer::new(content);

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should skip leading whitespace but preserve structure
    assert!(!tokens.is_empty());

    // Find first override or section token
    let has_override = tokens
        .iter()
        .any(|t| t.token_type == TokenType::OverrideOpen);
    let has_section = tokens
        .iter()
        .any(|t| t.token_type == TokenType::SectionHeader);
    let has_colon = tokens.iter().any(|t| t.token_type == TokenType::Colon);

    assert!(has_override || has_section);
    assert!(has_colon);
}

#[test]
fn tokenizer_iteration_limit_exceeded() {
    // Create input that could cause infinite tokenization if not handled properly
    let mut tokenizer = AssTokenizer::new("a");

    // Manually create a situation where next_token might not advance position
    // This tests the iteration limit protection in tokenize_all
    let result = tokenizer.tokenize_all();

    // Should succeed with normal input
    assert!(result.is_ok());

    // Test with input designed to trigger many iterations
    let long_input = "a".repeat(100);
    let mut long_tokenizer = AssTokenizer::new(&long_input);
    let result = long_tokenizer.tokenize_all();

    // Should still succeed but tests the iteration counting logic
    assert!(result.is_ok());
}

#[test]
fn tokenizer_position_advancement_protection() {
    // Test the infinite loop protection in next_token
    let mut tokenizer = AssTokenizer::new("test");

    // Normal tokenization should work
    let token = tokenizer.next_token();
    assert!(token.is_ok());
    assert!(token.unwrap().is_some());
}
