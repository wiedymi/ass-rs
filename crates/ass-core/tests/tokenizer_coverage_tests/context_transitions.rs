//! Tokenizer coverage tests for context transitions.
//!
//! Exercises section header, field value, style override, newline reset, and
//! nested context transitions within the tokenizer state machine.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[test]
fn test_section_header_transitions() {
    let input = "[Script Info]\nTitle: Test";
    let mut tokenizer = AssTokenizer::new(input);

    // Test section header - should trigger context change to SectionHeader
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::SectionHeader));

    // After section header, get section close
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::SectionClose));

    // Then get newline
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Newline));

    // Then get field content
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
}

#[test]
fn test_field_value_context_transitions() {
    let input = "Title: Test Value";
    let mut tokenizer = AssTokenizer::new(input);

    // Get the title text
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));

    // Test colon - should trigger context change to FieldValue
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Colon));

    // Skip whitespace
    let token = tokenizer.next_token().unwrap().unwrap();
    if matches!(token.token_type, TokenType::Whitespace) {
        // Get the actual value
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
    } else {
        assert!(matches!(token.token_type, TokenType::Text));
    }
}

#[test]
fn test_style_override_context_transitions() {
    let input = "{\\b1}text{\\b0}";
    let mut tokenizer = AssTokenizer::new(input);

    // Test first style override block
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::OverrideBlock));

    // Test first override close
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::OverrideClose));

    // Test text after override
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));

    // Test second override block
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::OverrideBlock));

    // Test second override close
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::OverrideClose));
}

#[test]
fn test_newline_context_reset() {
    let input = "Title: Test\nNext Line";
    let mut tokenizer = AssTokenizer::new(input);

    // Skip to colon to enter FieldValue context
    while let Some(token) = tokenizer.next_token().unwrap() {
        if matches!(token.token_type, TokenType::Colon) {
            break;
        }
    }

    // Skip to newline
    while let Some(token) = tokenizer.next_token().unwrap() {
        if matches!(token.token_type, TokenType::Newline) {
            break;
        }
    }

    // After newline, context should reset to Document
    let token = tokenizer.next_token().unwrap().unwrap();
    assert!(matches!(token.token_type, TokenType::Text));
}

#[test]
fn test_nested_context_transitions() {
    let input = "[V4+ Styles]: {\\b1}test{\\b0}, value";
    let mut tokenizer = AssTokenizer::new(input);

    let mut contexts_seen = 0;
    while let Some(_token) = tokenizer.next_token().unwrap() {
        contexts_seen += 1;
    }

    assert!(contexts_seen > 8); // Should see multiple context transitions
}
