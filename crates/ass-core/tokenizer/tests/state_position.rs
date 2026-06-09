//! Position tracking, reset, and context-transition tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[test]
fn position_tracking() {
    let mut tokenizer = AssTokenizer::new("Hello\nWorld");

    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);

    let _token1 = tokenizer.next_token().unwrap().unwrap();
    assert!(tokenizer.position() > 0);

    let _token2 = tokenizer.next_token().unwrap().unwrap(); // newline
    assert_eq!(tokenizer.line(), 2);

    let _token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(tokenizer.line(), 2);
}

#[test]
fn reset_functionality() {
    let mut tokenizer = AssTokenizer::new("Test content");

    // Consume some tokens
    let _token = tokenizer.next_token().unwrap().unwrap();
    assert!(tokenizer.position() > 0);

    // Reset and verify
    tokenizer.reset();
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);

    // Should be able to tokenize again
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.span, "Test content");
}

#[test]
fn reset_with_bom() {
    let mut tokenizer = AssTokenizer::new("\u{FEFF}Test");

    // Should start after BOM
    assert_eq!(tokenizer.position(), 3);

    let _token = tokenizer.next_token().unwrap().unwrap();

    // Reset should go back to after BOM
    tokenizer.reset();
    assert_eq!(tokenizer.position(), 3);
}

#[test]
fn issues_collection() {
    let mut tokenizer = AssTokenizer::new("Valid content");
    let _tokens = tokenizer.tokenize_all().unwrap();

    // Initially no issues
    assert!(tokenizer.issues().is_empty());
}

#[test]
fn context_transitions() {
    let mut tokenizer = AssTokenizer::new("[Section]\nField: Value\n{\\override}text");

    // Should handle all context transitions properly
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());

    // Verify we can tokenize various contexts without panicking
    for token in &tokens {
        assert!(
            !token.span.is_empty()
                || matches!(
                    token.token_type,
                    TokenType::Newline
                        | TokenType::Colon
                        | TokenType::Comma
                        | TokenType::SectionClose
                        | TokenType::OverrideClose
                )
        );
    }
}

#[test]
fn empty_section_header() {
    let mut tokenizer = AssTokenizer::new("[]");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::SectionHeader);
    assert_eq!(token1.span, "[");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::SectionClose);
}

#[test]
fn empty_style_override() {
    let mut tokenizer = AssTokenizer::new("{}");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::OverrideBlock);
    assert_eq!(token1.span, "{");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::OverrideClose);
}

#[test]
fn line_column_tracking() {
    let mut tokenizer = AssTokenizer::new("abc\ndef\nghi");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.line, 1);
    assert_eq!(token1.column, 1);

    let _newline1 = tokenizer.next_token().unwrap().unwrap();

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.line, 2);
    assert_eq!(token2.column, 1);
}

#[test]
fn tokenize_all_empty() {
    let mut tokenizer = AssTokenizer::new("");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn tokenize_all_whitespace() {
    let mut tokenizer = AssTokenizer::new("   \t\n  ");
    let tokens = tokenizer.tokenize_all().unwrap();
    // Should only have the newline token
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::Newline);
}
