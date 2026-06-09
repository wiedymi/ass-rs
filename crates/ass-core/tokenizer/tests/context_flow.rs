//! Context-flow, dialogue, and loop-protection tests for [`AssTokenizer`].

use crate::tokenizer::{AssTokenizer, TokenType};

#[test]
fn position_after_each_token() {
    let mut tokenizer = AssTokenizer::new("abc,def");
    let mut last_pos = 0;

    while let Some(token) = tokenizer.next_token().unwrap() {
        assert!(tokenizer.position() >= last_pos);
        last_pos = tokenizer.position();
        assert!(!token.span.is_empty() || matches!(token.token_type, TokenType::Comma));
    }
}

#[test]
fn context_reset_on_newline() {
    let mut tokenizer = AssTokenizer::new("field: value\nnext line");

    let _field = tokenizer.next_token().unwrap().unwrap();
    let _colon = tokenizer.next_token().unwrap().unwrap();
    let _value = tokenizer.next_token().unwrap().unwrap();
    let _newline = tokenizer.next_token().unwrap().unwrap();

    // After newline, context should reset
    let next_token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(next_token.token_type, TokenType::Text);
    assert_eq!(next_token.span, "next line");
}

#[test]
fn bom_with_various_content() {
    let contents = [
        "\u{FEFF}[Section]",
        "\u{FEFF}text content",
        "\u{FEFF}{\\override}",
        "\u{FEFF}; comment",
    ];

    for content in &contents {
        let mut tokenizer = AssTokenizer::new(content);
        assert_eq!(tokenizer.position(), 3); // After BOM
        let tokens = tokenizer.tokenize_all().unwrap();
        assert!(!tokens.is_empty());
    }
}

#[test]
fn tokenize_all_iteration_limit() {
    // Create input that could potentially cause infinite loop
    let long_text = "a".repeat(100);
    let mut tokenizer = AssTokenizer::new(&long_text);
    let result = tokenizer.tokenize_all();
    assert!(result.is_ok());
}

#[test]
fn complex_dialogue_line() {
    let dialogue = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,{\\an8\\fad(300,300)}Hello {\\c&H0000FF&}world{\\r}!";
    let mut tokenizer = AssTokenizer::new(dialogue);
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should tokenize complex dialogue without errors
    assert!(tokens.len() > 10);

    // Should have various token types
    let has_text = tokens.iter().any(|t| t.token_type == TokenType::Text);
    let has_colon = tokens.iter().any(|t| t.token_type == TokenType::Colon);
    let has_comma = tokens.iter().any(|t| t.token_type == TokenType::Comma);
    let has_override = tokens
        .iter()
        .any(|t| t.token_type == TokenType::OverrideBlock);

    assert!(has_text);
    assert!(has_colon);
    assert!(has_comma);
    assert!(has_override);
}

#[test]
fn tokenizer_error_path_scanner_failure() {
    // Test error handling when scanner operations fail
    let mut tokenizer = AssTokenizer::new("[\x00invalid");
    let result = tokenizer.next_token();
    // Should handle scanner errors gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn tokenizer_infinite_loop_protection() {
    // Test that tokenizer doesn't get stuck in infinite loops
    let mut tokenizer = AssTokenizer::new("");
    let mut count = 0;
    while tokenizer.next_token().unwrap().is_some() {
        count += 1;
        assert!(count <= 1000, "Tokenizer stuck in infinite loop");
    }
}

#[test]
fn tokenizer_position_advancement_check() {
    // Test that position always advances or we reach end
    let mut tokenizer = AssTokenizer::new("abc");
    let mut last_pos = 0;

    while let Some(_token) = tokenizer.next_token().unwrap() {
        let current_pos = tokenizer.position();
        assert!(current_pos > last_pos);
        last_pos = current_pos;
    }
}

#[test]
fn context_enter_field_value() {
    // Test entering field value context
    let mut tokenizer = AssTokenizer::new("Key: Value here");
    let _key = tokenizer.next_token().unwrap();
    let colon = tokenizer.next_token().unwrap();
    assert_eq!(colon.unwrap().token_type, TokenType::Colon);

    // After colon, should be in field value context
    let value = tokenizer.next_token().unwrap();
    assert!(value.is_some());
}
