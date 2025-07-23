//! Comprehensive tests for tokenizer functionality

use super::*;

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
