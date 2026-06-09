//! Delimiter and context-bracket tokenizer edge paths
//!
//! Covers section brackets, colons, override braces, and commas across the
//! tokenizer's context-dependent character handling paths.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[test]
fn test_closing_section_bracket_in_section_context() {
    // This should hit line 104-108: (']', TokenContext::SectionHeader)
    let input = "[Script Info]";
    let mut tokenizer = AssTokenizer::new(input);

    // Get section header - this enters SectionHeader context
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::SectionHeader));

    // Get closing bracket - this should hit the SectionClose path
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::SectionClose));
}

#[test]
fn test_colon_in_document_context() {
    // This should hit line 109-113: (':', TokenContext::Document)
    let input = "Title: Value";
    let mut tokenizer = AssTokenizer::new(input);

    // Skip to colon in Document context
    let _text_token = tokenizer.next_token().unwrap().unwrap();

    // This colon should trigger the Document context path
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Colon));
}

#[test]
fn test_opening_style_override_brace() {
    // This should hit line 115-118: ('{', _)
    let input = "{\\b1}";
    let mut tokenizer = AssTokenizer::new(input);

    // This should trigger the style override opening path
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::OverrideBlock));
}

#[test]
fn test_closing_style_override_brace_in_override_context() {
    // This should hit line 119-123: ('}', TokenContext::StyleOverride)
    let input = "{\\b1}";
    let mut tokenizer = AssTokenizer::new(input);

    // Get the override block - this enters StyleOverride context
    let _override_token = tokenizer.next_token().unwrap().unwrap();

    // Get the closing brace - this should hit the OverrideClose path
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::OverrideClose));
}

#[test]
fn test_comma_delimiter_handling() {
    // This should hit line 124-127: (',', _)
    let input = "field1,field2";
    let mut tokenizer = AssTokenizer::new(input);

    // Skip first field
    let _field1 = tokenizer.next_token().unwrap().unwrap();

    // This comma should trigger the comma handling path
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Comma));
}
