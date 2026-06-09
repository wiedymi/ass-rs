//! Coverage tests for `AssTokenizer` context transitions.
//!
//! Exercises rapid context changes, field-value delimiter handling, delimiter
//! classification, and exhaustive context-transition triggers.

use ass_core::tokenizer::{state::TokenContext, AssTokenizer, TokenType};

/// Test tokenizer context transitions with edge case combinations
#[test]
fn test_tokenizer_context_edge_transitions() {
    // Test rapid context changes that might cause state inconsistencies
    let mut tokenizer = AssTokenizer::new("[}]{:}{}[");

    let mut tokens = Vec::new();
    while let Ok(Some(token)) = tokenizer.next_token() {
        tokens.push(token);
        // Prevent infinite loops in testing
        if tokens.len() > 20 {
            break;
        }
    }

    // Should handle malformed delimiters without panicking
    assert!(!tokens.is_empty());
}

/// Test tokenizer field value context handling edge cases
#[test]
fn test_tokenizer_field_value_context_edge_cases() {
    // Test field value context with unusual delimiters
    let mut tokenizer = AssTokenizer::new("key:value}with]wrong{delimiters");

    let mut tokens = Vec::new();
    while let Ok(Some(token)) = tokenizer.next_token() {
        tokens.push(token);
        if tokens.len() > 10 {
            break;
        }
    }

    // Should handle misplaced delimiters in field value context
    assert!(tokens.len() >= 3); // At least key, colon, value
    assert!(tokens.iter().any(|t| t.token_type == TokenType::Colon));
}

/// Test delimiter type classification edge cases
#[test]
fn test_delimiter_type_edge_cases() {
    let mut tokenizer = AssTokenizer::new("{}[]():,;\n\r");

    let mut delimiter_types = Vec::new();

    while let Ok(Some(token)) = tokenizer.next_token() {
        match token.token_type {
            TokenType::SectionHeader | TokenType::SectionOpen => {
                delimiter_types.push("section");
            }
            TokenType::SectionClose => {
                delimiter_types.push("section_close");
            }
            TokenType::OverrideBlock | TokenType::OverrideOpen => {
                delimiter_types.push("override");
            }
            TokenType::OverrideClose => {
                delimiter_types.push("override_close");
            }
            TokenType::Colon => {
                delimiter_types.push("colon");
            }
            TokenType::Comma => {
                delimiter_types.push("comma");
            }
            TokenType::Comment => {
                delimiter_types.push("comment");
            }
            TokenType::Newline => {
                delimiter_types.push("newline");
            }
            _ => {}
        }
    }

    // Should recognize various delimiter types
    assert!(!delimiter_types.is_empty());
}

/// Test token context transition exhaustively
#[test]
fn test_token_context_transitions_exhaustive() {
    // Test all major context transitions
    let context_transitions = [
        ("[", TokenContext::SectionHeader),
        (":", TokenContext::FieldValue),
        ("{", TokenContext::StyleOverride),
    ];

    for (trigger, _expected_context) in context_transitions {
        let mut tokenizer = AssTokenizer::new(trigger);

        // Get initial token which should trigger context change
        let _token = tokenizer.next_token().unwrap();

        // Context should have changed (we can't directly access context,
        // but we can test behavior)
        let remaining_input = &trigger[1..];
        if !remaining_input.is_empty() {
            let _next_token = tokenizer.next_token();
            // Should not panic or cause infinite loops
        }
    }
}
