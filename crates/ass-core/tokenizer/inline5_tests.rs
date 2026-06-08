//! Targeted token-kind coverage tests for [`AssTokenizer`].

use super::*;

#[test]
fn tokenizer_context_based_token_creation() {
    let source = "{\\b1}Bold text{\\b0}";
    let mut tokenizer = AssTokenizer::new(source);

    let mut token_count = 0;
    while let Ok(Some(token)) = tokenizer.next_token() {
        // Verify each token was created with proper context
        assert!(token.line >= 1);
        assert!(token.column >= 1);
        assert!(!token.span.is_empty());

        token_count += 1;
        if token_count > 15 {
            break;
        }
    }

    assert!(token_count > 0);
}

#[test]
fn tokenizer_section_header_start_tracking() {
    // Target lines 93-95: start position tracking
    let source = "[Script Info]";
    let mut tokenizer = AssTokenizer::new(source);

    // This should hit the start_pos, start_line, start_column tracking
    let token = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
}

#[test]
fn tokenizer_section_close_bracket() {
    // Target lines 104-106: ']' in SectionHeader context
    let source = "[Test]";
    let mut tokenizer = AssTokenizer::new(source);

    // Get section header
    let _header = tokenizer.next_token().unwrap().unwrap();
    // Get closing bracket - this should hit lines 104-106
    let close = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(
        close.token_type,
        crate::tokenizer::tokens::TokenType::SectionClose
    );
}

#[test]
fn tokenizer_colon_field_separator() {
    // Target lines 109: ':' in Document context
    let source = "Key:Value";
    let mut tokenizer = AssTokenizer::new(source);

    // Get key
    let _key = tokenizer.next_token().unwrap().unwrap();
    // Get colon - this should hit line 109
    let colon = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(colon.token_type, crate::tokenizer::tokens::TokenType::Colon);
}

#[test]
fn tokenizer_comma_separator() {
    // Target lines 115: ',' token
    let source = "val1,val2";
    let mut tokenizer = AssTokenizer::new(source);

    let _val1 = tokenizer.next_token().unwrap().unwrap();
    let comma = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(comma.token_type, crate::tokenizer::tokens::TokenType::Comma);
}

#[test]
fn tokenizer_newline_handling() {
    // Target lines 119, 121: newline tokens
    let source = "line1\nline2\r\nline3";
    let mut tokenizer = AssTokenizer::new(source);

    let _line1 = tokenizer.next_token().unwrap().unwrap();
    let newline1 = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(
        newline1.token_type,
        crate::tokenizer::tokens::TokenType::Newline
    );
}

#[test]
fn tokenizer_style_override_tokens() {
    // Target lines 124, 128: '{' and '}' tokens
    let source = "{\\b1}text{\\b0}";
    let mut tokenizer = AssTokenizer::new(source);

    let override_block = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(
        override_block.token_type,
        crate::tokenizer::tokens::TokenType::OverrideBlock
    );
}

#[test]
fn tokenizer_comment_exclamation() {
    // Target lines 137: '!' comment
    let source = "!: This is a comment";
    let mut tokenizer = AssTokenizer::new(source);

    let comment = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(
        comment.token_type,
        crate::tokenizer::tokens::TokenType::Comment
    );
}

#[test]
fn tokenizer_comment_semicolon() {
    // Target lines 146: ';' comment
    let source = "; This is a comment";
    let mut tokenizer = AssTokenizer::new(source);

    let comment = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(
        comment.token_type,
        crate::tokenizer::tokens::TokenType::Comment
    );
}

#[test]
fn tokenizer_whitespace_token() {
    // Target lines 151: whitespace handling
    // Use FieldValue context where whitespace isn't skipped
    let source = "Key:   \t  ";
    let mut tokenizer = AssTokenizer::new(source);

    // Get key token
    let _key = tokenizer.next_token().unwrap().unwrap();
    // Get colon token (switches to FieldValue context)
    let _colon = tokenizer.next_token().unwrap().unwrap();
    // Get whitespace token in FieldValue context
    let whitespace = tokenizer.next_token().unwrap().unwrap();
    assert_eq!(
        whitespace.token_type,
        crate::tokenizer::tokens::TokenType::Whitespace
    );
}
