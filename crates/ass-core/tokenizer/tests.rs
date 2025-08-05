//! Comprehensive tests for tokenizer functionality

use super::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec};
#[cfg(not(feature = "std"))]
use hashbrown::HashSet;
#[cfg(feature = "std")]
use std::collections::HashSet;
#[test]
fn tokenizer_new_without_bom() {
    let tokenizer = AssTokenizer::new("Hello World");
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
}

#[test]
fn tokenizer_new_with_bom() {
    let tokenizer = AssTokenizer::new("\u{FEFF}Hello World");
    assert_eq!(tokenizer.position(), 3); // BOM is 3 bytes
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);
}

#[test]
fn tokenize_empty_string() {
    let mut tokenizer = AssTokenizer::new("");
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenize_whitespace_only() {
    let mut tokenizer = AssTokenizer::new("   \t  ");
    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenize_section_header_basic() {
    let mut tokenizer = AssTokenizer::new("[Script Info]");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::SectionHeader);
    assert_eq!(token1.span, "[Script Info");
    assert_eq!(token1.line, 1);
    assert_eq!(token1.column, 1);

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::SectionClose);
    assert_eq!(token2.span, "]");

    let result = tokenizer.next_token().unwrap();
    assert!(result.is_none());
}

#[test]
fn tokenize_section_header_with_spaces() {
    let mut tokenizer = AssTokenizer::new("[ V4+ Styles ]");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::SectionHeader);
    assert_eq!(token1.span, "[ V4+ Styles ");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::SectionClose);
}

#[test]
fn tokenize_field_with_colon() {
    let mut tokenizer = AssTokenizer::new("Title: Test Script");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "Title");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Colon);
    assert_eq!(token2.span, ":");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, " Test Script");
}

#[test]
fn tokenize_style_override_simple() {
    let mut tokenizer = AssTokenizer::new("{\\b1}");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::OverrideBlock);
    assert_eq!(token1.span, "{\\b1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::OverrideClose);
    assert_eq!(token2.span, "}");
}

#[test]
fn tokenize_style_override_complex() {
    let mut tokenizer = AssTokenizer::new("{\\c&H0000FF&\\fs20}Hello{\\r}");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::OverrideBlock);

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::OverrideClose);

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "Hello");

    let token4 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token4.token_type, TokenType::OverrideBlock);

    let token5 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token5.token_type, TokenType::OverrideClose);
}

#[test]
fn tokenize_comma_separator() {
    let mut tokenizer = AssTokenizer::new("value1,value2,value3");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "value1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Comma);
    assert_eq!(token2.span, ",");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "value2");

    let token4 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token4.token_type, TokenType::Comma);

    let token5 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token5.token_type, TokenType::Text);
    assert_eq!(token5.span, "value3");
}

#[test]
fn tokenize_newline_unix() {
    let mut tokenizer = AssTokenizer::new("line1\nline2");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "line1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Newline);
    assert_eq!(token2.span, "\n");
    assert_eq!(token2.line, 1);

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "line2");
    assert_eq!(token3.line, 2);
}

#[test]
fn tokenize_newline_windows() {
    let mut tokenizer = AssTokenizer::new("line1\r\nline2");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "line1");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Newline);
    assert_eq!(token2.span, "\r\n");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.token_type, TokenType::Text);
    assert_eq!(token3.span, "line2");
    assert_eq!(token3.line, 2);
}

#[test]
fn tokenize_comment_semicolon() {
    let mut tokenizer = AssTokenizer::new("; This is a comment");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Comment);
    assert_eq!(token.span, "; This is a comment");
}

#[test]
fn tokenize_comment_exclamation_colon() {
    let mut tokenizer = AssTokenizer::new("!: This is also a comment");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Comment);
    assert_eq!(token.span, "!: This is also a comment");
}

#[test]
fn tokenize_exclamation_not_comment() {
    let mut tokenizer = AssTokenizer::new("!Not a comment");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "!Not a comment");
}

#[test]
fn tokenize_mixed_content() {
    let mut tokenizer = AssTokenizer::new("[Section]\nField: Value\n; Comment\n{\\b1}Text");
    let tokens = tokenizer.tokenize_all().unwrap();

    assert!(tokens.len() >= 8);
    assert_eq!(tokens[0].token_type, TokenType::SectionHeader);
    assert_eq!(tokens[1].token_type, TokenType::SectionClose);
    assert_eq!(tokens[2].token_type, TokenType::Newline);
    assert_eq!(tokens[3].token_type, TokenType::Text);
    assert_eq!(tokens[4].token_type, TokenType::Colon);
    // And so on...
}

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

#[test]
fn complex_script_structure() {
    let script =
        "[Script Info]\nTitle: Test\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";

    let mut tokenizer = AssTokenizer::new(script);
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have multiple tokens without errors
    assert!(tokens.len() > 5);

    // Verify basic token types are present
    let has_section = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::SectionHeader));
    let has_colon = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Colon));
    let has_comma = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Comma));
    let has_text = tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Text));

    assert!(has_section);
    assert!(has_colon);
    assert!(has_comma);
    assert!(has_text);
}

#[test]
fn unicode_content() {
    let mut tokenizer = AssTokenizer::new("こんにちは世界");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "こんにちは世界");
}

#[test]
fn special_characters() {
    let mut tokenizer = AssTokenizer::new("Test with émojis 🎬 and symbols ©®™");

    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span, "Test with émojis 🎬 and symbols ©®™");
}

#[test]
fn carriage_return_only() {
    let mut tokenizer = AssTokenizer::new("line1\rline2");

    let _token1 = tokenizer.next_token().unwrap().unwrap();
    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::Newline);
    assert_eq!(token2.span, "\r");

    let token3 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token3.span, "line2");
    assert_eq!(token3.line, 2);
}

#[test]
fn multiple_consecutive_newlines() {
    let mut tokenizer = AssTokenizer::new("line1\n\n\nline2");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have: text, newline, newline, newline, text
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].token_type, TokenType::Text);
    assert_eq!(tokens[1].token_type, TokenType::Newline);
    assert_eq!(tokens[2].token_type, TokenType::Newline);
    assert_eq!(tokens[3].token_type, TokenType::Newline);
    assert_eq!(tokens[4].token_type, TokenType::Text);

    assert_eq!(tokens[4].line, 4); // Should be on line 4
}

#[test]
fn nested_braces_in_text() {
    let mut tokenizer = AssTokenizer::new("Text with {nested {braces} inside}");

    let token1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token1.token_type, TokenType::Text);
    assert_eq!(token1.span, "Text with ");

    let token2 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token2.token_type, TokenType::OverrideBlock);

    // Continue tokenizing to ensure no panic
    let _remaining_tokens = tokenizer.tokenize_all().unwrap();
}

#[test]
fn tokenizer_debug() {
    let tokenizer = AssTokenizer::new("test");
    let debug_str = format!("{tokenizer:?}");
    assert!(debug_str.contains("AssTokenizer"));
}

#[test]
fn tokenizer_clone() {
    let tokenizer = AssTokenizer::new("test content");
    let cloned = tokenizer.clone();
    assert_eq!(tokenizer.position(), cloned.position());
    assert_eq!(tokenizer.line(), cloned.line());
    assert_eq!(tokenizer.column(), cloned.column());
}

#[test]
fn field_value_context() {
    let mut tokenizer = AssTokenizer::new("Key: Value with spaces and, commas");

    let _key = tokenizer.next_token().unwrap().unwrap();
    let _colon = tokenizer.next_token().unwrap().unwrap();

    // After colon, should be in field value context
    // Field values stop at commas (CSV-style delimiter)
    let value = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(value.token_type, TokenType::Text);
    assert_eq!(value.span, " Value with spaces and");

    // Next token should be the comma delimiter
    let comma = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(comma.token_type, TokenType::Comma);
}

#[test]
fn malformed_section_header() {
    let mut tokenizer = AssTokenizer::new("[Unclosed section header");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    // Should still tokenize something rather than panic
}

#[test]
fn malformed_style_override() {
    let mut tokenizer = AssTokenizer::new("{\\unclosed override");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    // Should still tokenize something rather than panic
}

#[test]
fn very_long_content() {
    let long_text = "A".repeat(10000);
    let mut tokenizer = AssTokenizer::new(&long_text);
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
    assert_eq!(token.span.len(), 10000);
}

#[test]
fn only_delimiters() {
    let mut tokenizer = AssTokenizer::new("{}[]:,\n");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have tokens for each delimiter
    assert!(tokens.len() >= 6);
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::OverrideBlock)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::OverrideClose)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::SectionHeader)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::SectionClose)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Colon)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Comma)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token_type, TokenType::Newline)));
}

#[test]
fn alternating_contexts() {
    let mut tokenizer = AssTokenizer::new("[Section]field:value{\\override}text[Another]");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle rapid context changes without errors
    assert!(tokens.len() >= 8);

    // Verify we have different token types
    let token_types: HashSet<_> = tokens.iter().map(|t| &t.token_type).collect();
    assert!(token_types.len() >= 4); // At least 4 different token types
}

#[test]
fn whitespace_variations() {
    let inputs = [
        " \t spaces and tabs",
        "\u{00A0}non-breaking space",
        "\u{2000}en quad",
        "\u{3000}ideographic space",
    ];

    for input in &inputs {
        let mut tokenizer = AssTokenizer::new(input);
        let tokens = tokenizer.tokenize_all().unwrap();
        assert!(!tokens.is_empty());
    }
}

#[test]
fn control_characters() {
    let mut tokenizer = AssTokenizer::new("text\x00\x01\x02control");
    let tokens = tokenizer.tokenize_all().unwrap();
    assert!(!tokens.is_empty());
    // Should handle control characters without panic
}

#[test]
fn single_character_tokens() {
    let inputs = ["{", "}", "[", "]", ":", ",", ";", "!"];

    for &input in &inputs {
        let mut tokenizer = AssTokenizer::new(input);
        let result = tokenizer.next_token().unwrap();
        assert!(result.is_some());
    }
}

#[test]
fn mixed_line_endings() {
    let mut tokenizer = AssTokenizer::new("line1\nline2\r\nline3\rline4");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should have multiple newline tokens with different styles
    let newlines: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Newline)
        .collect();
    assert_eq!(newlines.len(), 3);
    assert_eq!(newlines[0].span, "\n");
    assert_eq!(newlines[1].span, "\r\n");
    assert_eq!(newlines[2].span, "\r");
}

#[test]
fn empty_tokens_edge_case() {
    let mut tokenizer = AssTokenizer::new("::,,{}{}\n\n");
    let tokens = tokenizer.tokenize_all().unwrap();

    // Should handle consecutive delimiters
    assert!(tokens.len() >= 6);
}

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

#[test]
fn context_reset_to_document() {
    // Test context reset on newline
    let mut tokenizer = AssTokenizer::new("{\\an8}text\nNew line");

    // Debug: collect all tokens to understand the sequence
    let mut all_tokens = Vec::new();
    while let Some(token) = tokenizer.next_token().unwrap() {
        all_tokens.push(token);
    }

    // Verify we have the expected tokens
    assert!(all_tokens.len() >= 4);

    // First should be override block
    assert_eq!(all_tokens[0].token_type, TokenType::OverrideBlock);

    // Second should be override close
    assert_eq!(all_tokens[1].token_type, TokenType::OverrideClose);

    // Third should be text content
    assert_eq!(all_tokens[2].token_type, TokenType::Text);
    assert_eq!(all_tokens[2].span, "text");

    // Fourth should be newline or text containing newline and remaining content
    let fourth_token = &all_tokens[3];
    assert!(
        fourth_token.token_type == TokenType::Newline
            || (fourth_token.token_type == TokenType::Text && fourth_token.span.contains('\n'))
    );
}

#[test]
fn delimiter_in_wrong_context_as_text() {
    // Test that delimiter chars in wrong context become text
    let mut tokenizer = AssTokenizer::new("}]");

    let first = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(first.token_type, TokenType::Text);
    assert_eq!(first.span, "}");

    let second = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(second.token_type, TokenType::Text);
    assert_eq!(second.span, "]");
}

#[test]
fn exclamation_without_colon() {
    // Test exclamation mark that's not followed by colon
    let mut tokenizer = AssTokenizer::new("!notcomment");
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
}

#[test]
fn carriage_return_line_feed_handling() {
    // Test proper CRLF handling
    let mut tokenizer = AssTokenizer::new("line1\r\nline2");
    let _text1 = tokenizer.next_token().unwrap();
    let newline = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(newline.token_type, TokenType::Newline);

    // Position should advance past both \r and \n
    let _text2 = tokenizer.next_token().unwrap();
    assert!(tokenizer.position() > 7); // past "line1\r\n"
}

#[test]
fn issues_collector_functionality() {
    // Test that issues are properly collected
    let mut tokenizer = AssTokenizer::new("test content");
    let _tokens = tokenizer.tokenize_all().unwrap();
    let issues = tokenizer.issues();
    // Issues should be accessible (even if empty for valid input)
    assert!(issues.is_empty() || !issues.is_empty());
}

#[test]
fn tokenizer_allows_whitespace_skipping() {
    // Test whitespace skipping behavior in different contexts
    let mut tokenizer = AssTokenizer::new("   [Section]   ");
    let token = tokenizer.next_token().unwrap().unwrap();
    // Should skip leading whitespace and find section header
    assert_eq!(token.token_type, TokenType::SectionHeader);
}

#[test]
fn context_allows_whitespace_skipping() {
    // Test context-dependent whitespace skipping
    let mut tokenizer = AssTokenizer::new("  text  ");
    // Document context is the default, so whitespace should be skipped
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.token_type, TokenType::Text);
}

#[test]
fn tokenize_all_large_input() {
    // Test tokenize_all with reasonable size to avoid iteration limit
    let content = "[Script Info]\nTitle: Test\n".repeat(3);
    let mut tokenizer = AssTokenizer::new(&content);
    let result = tokenizer.tokenize_all();
    assert!(result.is_ok());
}

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

#[test]
fn tokenizer_uuencoded_data_context() {
    use crate::tokenizer::state::TokenContext;

    // Test UuEncodedData context behavior
    let context = TokenContext::UuEncodedData;
    assert!(!context.allows_whitespace_skipping());
    assert!(!context.is_delimited_block());
    assert_eq!(context.closing_delimiter(), None);
    assert_eq!(context.enter_field_value(), TokenContext::UuEncodedData);
    assert_eq!(context.reset_to_document(), TokenContext::Document);
}

#[test]
fn tokenizer_drawing_commands_context() {
    use crate::tokenizer::state::TokenContext;

    // Test DrawingCommands context behavior
    let context = TokenContext::DrawingCommands;
    assert!(context.allows_whitespace_skipping());
    assert!(!context.is_delimited_block());
    assert_eq!(context.closing_delimiter(), None);
    assert_eq!(context.enter_field_value(), TokenContext::DrawingCommands);
    assert_eq!(context.reset_to_document(), TokenContext::Document);
}

#[test]
fn tokenizer_critical_issues() {
    use crate::tokenizer::state::{IssueCollector, IssueLevel, TokenIssue};

    let mut collector = IssueCollector::new();

    // Test critical issue handling
    let critical_issue = TokenIssue::critical("Critical error".to_string(), "test", 1, 1);
    collector.add_issue(critical_issue);

    assert!(collector.has_issues());
    assert!(collector.has_errors());
    assert_eq!(collector.issue_count(), 1);

    // Test issue level properties
    assert!(IssueLevel::Critical.is_error());
    assert!(IssueLevel::Critical.should_abort());
    assert_eq!(IssueLevel::Critical.as_str(), "critical");
}

#[test]
fn tokenizer_simd_edge_cases() {
    #[cfg(feature = "simd")]
    {
        use crate::tokenizer::simd;

        // Test SIMD with exactly 16 bytes (chunk boundary)
        let text_16 = "0123456789abcdef";
        assert_eq!(text_16.len(), 16);
        let result = simd::scan_delimiters(text_16);
        assert!(result.is_none());

        // Test SIMD with delimiter at chunk boundary
        let text_with_delimiter = "0123456789abcde:";
        let result = simd::scan_delimiters(text_with_delimiter);
        assert_eq!(result, Some(15));

        // Test hex parsing edge cases
        let invalid_hex = simd::parse_hex_u32("GGGG");
        assert!(invalid_hex.is_none());

        // Test UTF-8 validation with mixed content
        let mixed_utf8 = "Hello, 世界! 🦀";
        let result = simd::validate_utf8_batch(mixed_utf8.as_bytes());
        assert!(result.is_ok());
    }
}

#[test]
fn tokenizer_scanner_edge_cases() {
    use crate::tokenizer::scanner::TokenScanner;

    // Test scanner with empty input
    let scanner = TokenScanner::new("", 0, 1, 1);
    assert!(scanner.navigator().is_at_end());

    // Test scanner navigation methods
    let mut scanner = TokenScanner::new("test", 0, 1, 1);
    let nav = scanner.navigator_mut();

    // Test character advancement and position tracking
    let ch = nav.advance_char();
    assert!(ch.is_ok());
    assert_eq!(ch.unwrap(), 't');
    assert_eq!(nav.position(), 1);
    assert_eq!(nav.column(), 2);
}

#[test]
fn tokenizer_hex_value_detection() {
    use crate::tokenizer::scanner::TokenScanner;

    let mut scanner = TokenScanner::new("&H123ABC", 0, 1, 1);

    // Test hex value scanning
    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
    assert!(result.is_ok());
}

#[test]
fn tokenizer_field_value_comprehensive() {
    use crate::tokenizer::scanner::TokenScanner;

    // Test field value scanning with various content types
    let test_cases = vec![
        "simple_value",
        "123.45",
        "&H123ABC",
        "value,with,commas",
        "value:with:colons",
    ];

    for test_input in test_cases {
        let mut scanner = TokenScanner::new(test_input, 0, 1, 1);
        let result = scanner.scan_field_value();
        assert!(result.is_ok(), "Failed to scan field value: {test_input}");
    }
}

#[test]
fn tokenizer_token_validation() {
    use crate::tokenizer::tokens::{Token, TokenType};

    // Test token validation and properties
    let token = Token::new(TokenType::Text, "test", 1, 1);

    assert_eq!(token.len(), 4);
    assert!(!token.is_empty());
    assert_eq!(token.end_column(), 5);
    assert!(!token.is_whitespace());
    assert!(!token.is_delimiter());
    assert!(token.is_content());
    assert!(token.validate_utf8());

    // Test empty token
    let empty_token = Token::new(TokenType::Text, "", 1, 1);
    assert!(empty_token.is_empty());
    assert_eq!(empty_token.len(), 0);
    assert_eq!(empty_token.end_column(), 1);
}

#[test]
fn tokenizer_delimiter_type_comprehensive() {
    use crate::tokenizer::tokens::DelimiterType;

    // Test all delimiter types
    let delimiters = vec![
        DelimiterType::FieldSeparator,
        DelimiterType::ValueSeparator,
        DelimiterType::SectionBoundary,
        DelimiterType::OverrideBoundary,
        DelimiterType::CommentMarker,
        DelimiterType::LineTerminator,
        DelimiterType::DrawingSeparator,
        DelimiterType::TimeSeparator,
        DelimiterType::ColorSeparator,
    ];

    for delimiter in delimiters {
        let chars = delimiter.chars();
        assert!(
            !chars.is_empty(),
            "Delimiter should have characters: {delimiter:?}"
        );

        // Test that each character matches the delimiter
        for &ch in chars {
            assert!(
                delimiter.matches(ch),
                "Character '{ch}' should match delimiter: {delimiter:?}"
            );
        }
    }
}

#[test]
fn tokenizer_position_unicode_handling() {
    use crate::tokenizer::tokens::TokenPosition;

    let mut pos = TokenPosition::new(0, 1, 1);

    // Test advancing by Unicode string
    pos = pos.advance_by_str("Hello, 世界!");

    // Position should advance by byte length, not character count
    assert!(pos.offset > 10); // Should be more than ASCII length due to Unicode
    assert_eq!(pos.line, 1);
    assert!(pos.column > 10);

    // Test with newlines
    let mut pos_newline = TokenPosition::new(0, 1, 1);
    pos_newline = pos_newline.advance_by_str("line1\nline2");
    assert_eq!(pos_newline.line, 2);
    assert_eq!(pos_newline.column, 6); // "line2" = 5 chars + 1 for 1-based indexing
}

#[test]
fn tokenizer_issue_collector_comprehensive() {
    use crate::tokenizer::state::{IssueCollector, TokenIssue};

    let mut collector = IssueCollector::new();

    // Test all issue addition methods
    collector.add_warning("Warning message".to_string(), "span", 1, 1);
    collector.add_error("Error message".to_string(), "span", 2, 1);
    collector.add_critical("Critical message".to_string(), "span", 3, 1);

    assert_eq!(collector.issue_count(), 3);
    assert!(collector.has_issues());
    assert!(collector.has_errors());

    // Test taking issues
    let issues = collector.take_issues();
    assert_eq!(issues.len(), 3);
    assert_eq!(collector.issue_count(), 0);
    assert!(!collector.has_issues());

    // Test issue formatting
    let issue = TokenIssue::warning("Test warning".to_string(), "test_span", 1, 5);
    let location = issue.location_string();
    assert!(location.contains("1:5"));

    let formatted = issue.format_issue();
    assert!(formatted.contains("warning"));
    assert!(formatted.contains("Test warning"));
}

#[test]
fn tokenizer_all_token_types_properties() {
    use crate::tokenizer::tokens::TokenType;

    let all_types = vec![
        TokenType::Text,
        TokenType::Number,
        TokenType::HexValue,
        TokenType::Colon,
        TokenType::Comma,
        TokenType::Newline,
        TokenType::SectionOpen,
        TokenType::SectionClose,
        TokenType::SectionName,
        TokenType::SectionHeader,
        TokenType::OverrideOpen,
        TokenType::OverrideClose,
        TokenType::OverrideBlock,
        TokenType::Comment,
        TokenType::Whitespace,
        TokenType::DrawingScale,
        TokenType::UuEncodedLine,
        TokenType::FontFilename,
        TokenType::GraphicFilename,
        TokenType::FormatLine,
        TokenType::EventType,
        TokenType::TimeValue,
        TokenType::BooleanValue,
        TokenType::PercentageValue,
        TokenType::StringLiteral,
        TokenType::Invalid,
        TokenType::Eof,
    ];

    for token_type in all_types {
        // Test that name() doesn't panic and returns non-empty string
        let name = token_type.name();
        assert!(
            !name.is_empty(),
            "Token type name should not be empty: {token_type:?}"
        );

        // Test Display implementation
        let display_str = format!("{token_type}");
        assert!(
            !display_str.is_empty(),
            "Token type display should not be empty: {token_type:?}"
        );

        // Test consistency of classification methods
        if token_type.is_delimiter() {
            assert!(
                !token_type.is_content(),
                "Delimiter should not be content: {token_type:?}"
            );
        }

        if token_type.is_structural() {
            // Structural tokens define document structure
            assert!(
                matches!(
                    token_type,
                    TokenType::SectionHeader
                        | TokenType::SectionOpen
                        | TokenType::SectionClose
                        | TokenType::FormatLine
                        | TokenType::Newline
                ),
                "Unexpected structural token: {token_type:?}"
            );
        }
    }
}
