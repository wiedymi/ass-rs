//! Unicode, long-content, and nested-context tokenization tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

#[test]
fn tokenizer_bom_variants() {
    // Test different BOM scenarios
    let bom_content = "\u{FEFF}[Script Info]\nTitle: Test";
    let mut tokenizer = AssTokenizer::new(bom_content);

    // Should skip BOM and parse normally
    let first_token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(first_token.token_type, TokenType::SectionHeader);
}

#[test]
fn tokenizer_malformed_unicode() {
    // Test handling of complex Unicode scenarios
    let unicode_content = "emoji: 🎭🎬 text: こんにちは bidirectional: עברית";
    let mut tokenizer = AssTokenizer::new(unicode_content);

    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
}

#[test]
fn tokenizer_context_state_edge_cases() {
    // Test context state transitions in edge cases
    let mut tokenizer = AssTokenizer::new("[Section]\n:value\n}outside");

    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle context transitions properly
    let types: Vec<_> = tokens.iter().map(|t| &t.token_type).collect();
    assert!(types.contains(&&TokenType::SectionHeader));
    assert!(types.contains(&&TokenType::Colon));
}

#[test]
fn tokenizer_very_long_tokens() {
    // Test tokenizing very long content
    let long_text = "a".repeat(10000);
    let content = format!("[Section]\nTitle: {long_text}");
    let mut tokenizer = AssTokenizer::new(&content);

    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());

    // Should handle long content without issues
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Text));
}

#[test]
fn tokenizer_nested_context_handling() {
    // Test nested context scenarios that might cause issues
    let content = "{override{nested}text}[section{mixed}]:value{another}";
    let mut tokenizer = AssTokenizer::new(content);

    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());

    // Should handle nested contexts without infinite loops
    assert!(tokens.iter().any(|t| {
        matches!(
            t.token_type,
            TokenType::OverrideOpen | TokenType::OverrideClose
        )
    }));
}
