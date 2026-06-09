//! Tokenizer coverage tests for delimiters and content handling.
//!
//! Exercises comma delimiters, carriage returns, comment detection,
//! whitespace, special characters, Unicode, and mixed delimiter content.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[test]
fn test_comma_delimiter_handling() {
    let input = "Layer,Start,End,Style";
    let mut tokenizer = AssTokenizer::new(input);

    // Get first field
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));

    // Test comma delimiter
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Comma));

    // Get next field
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
}

#[test]
fn test_carriage_return_handling() {
    let input = "Line1\r\nLine2";
    let mut tokenizer = AssTokenizer::new(input);

    // Get first line text
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));

    // Test carriage return handling
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Newline));

    // Get second line text
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
}

#[test]
fn test_comment_detection_semicolon() {
    let input = "; This is a comment";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Comment));
}

#[test]
fn test_comment_detection_exclamation() {
    let input = "!: This is also a comment";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Comment));
}

#[test]
fn test_whitespace_handling_different_contexts() {
    let input = " \t spaces and tabs \t ";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    // Whitespace is typically tokenized as text in document context
    assert!(matches!(token.token_type, TokenType::Text));
}

#[test]
fn test_mixed_delimiters_and_content() {
    let input = "[Section]:value,field{tag}";
    let mut tokenizer = AssTokenizer::new(input);

    let mut token_types = Vec::new();
    while let Some(token) = tokenizer.next_token().unwrap() {
        token_types.push(token.token_type);
    }

    // Should have mixed token types
    assert!(token_types.len() > 3);
    assert!(token_types
        .iter()
        .any(|t| matches!(t, TokenType::SectionHeader)));
    assert!(token_types.iter().any(|t| matches!(t, TokenType::Colon)));
    assert!(token_types.iter().any(|t| matches!(t, TokenType::Comma)));
    assert!(token_types
        .iter()
        .any(|t| matches!(t, TokenType::OverrideBlock)));
}

#[test]
fn test_unicode_content_handling() {
    let input = "测试 Unicode テスト";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
    // The token span should contain the full text until a delimiter
    assert!(token.span.contains("测试"));
}

#[test]
fn test_special_characters_in_text() {
    let input = "text with # @ $ % special chars";
    let mut tokenizer = AssTokenizer::new(input);

    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
    assert!(token.span.contains("text"));
}
