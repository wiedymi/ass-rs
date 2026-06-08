//! Advancement, span-boundary, and error-recovery tests for [`AssTokenizer`].

use super::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn tokenizer_position_line_column_advancement() {
    let source = "[Section]\nKey=Value\n! Comment";
    let mut tokenizer = AssTokenizer::new(source);

    // Track position advancement through multiple tokens
    let mut last_pos = 0;
    let mut tokens = Vec::new();

    while let Ok(Some(token)) = tokenizer.next_token() {
        // Verify position always advances (except at end)
        let current_pos = tokenizer.position();
        if !tokenizer.scanner.navigator().is_at_end() {
            assert!(current_pos > last_pos, "Position must advance");
        }

        // Verify line/column tracking
        assert!(token.line >= 1);
        assert!(token.column >= 1);

        tokens.push(token);
        last_pos = current_pos;

        // Prevent infinite test loops
        if tokens.len() > 20 {
            break;
        }
    }

    assert!(!tokens.is_empty());
}

#[test]
fn tokenizer_span_creation_and_boundaries() {
    let source = "[Test]\nField=Value123";
    let mut tokenizer = AssTokenizer::new(source);

    while let Ok(Some(token)) = tokenizer.next_token() {
        // Verify span is valid and within source bounds
        assert!(
            !token.span.is_empty()
                || token.token_type == crate::tokenizer::tokens::TokenType::Comment
        );
        assert!(token.span.len() <= source.len());

        // Verify span content matches expected position
        let start_pos = token.span.as_ptr() as usize - source.as_ptr() as usize;
        assert!(start_pos < source.len());
    }
}

#[test]
fn tokenizer_iteration_limit_comprehensive() {
    // Create content that could cause many iterations
    let source = "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,1,2,3,4,5,6,7,8,9,0,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z";
    let mut tokenizer = AssTokenizer::new(source);

    // This should hit the iteration limit in tokenize_all
    let result = tokenizer.tokenize_all();

    // Should either succeed with limited tokens or fail gracefully
    if let Ok(tokens) = result {
        // Should have stopped due to iteration limit
        assert!(tokens.len() <= 50, "Should respect iteration limit");
    } else {
        // Error is acceptable for iteration limit exceeded
    }
}

#[test]
fn tokenizer_all_error_recovery() {
    let source = "Valid[Section]\n\x00InvalidChar\nKey=Value";
    let mut tokenizer = AssTokenizer::new(source);

    let result = tokenizer.tokenize_all();

    // Should handle errors gracefully
    match result {
        Ok(tokens) => {
            assert!(!tokens.is_empty());
            // Should have collected some valid tokens before error
        }
        Err(_) => {
            // Error handling is acceptable
            assert!(!tokenizer.issues().is_empty());
        }
    }
}

#[test]
fn tokenizer_empty_source_boundaries() {
    let source = "";
    let mut tokenizer = AssTokenizer::new(source);

    // Should handle empty source without panicking
    assert_eq!(tokenizer.position(), 0);
    assert_eq!(tokenizer.line(), 1);
    assert_eq!(tokenizer.column(), 1);

    let result = tokenizer.next_token();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn tokenizer_single_character_advancement() {
    let source = "a";
    let mut tokenizer = AssTokenizer::new(source);

    let start_pos = tokenizer.position();
    if let Ok(Some(token)) = tokenizer.next_token() {
        let end_pos = tokenizer.position();
        assert!(end_pos > start_pos);
        assert_eq!(token.span, "a");
    }
}

#[test]
fn tokenizer_multi_byte_character_advancement() {
    let source = "🎵音楽";
    let mut tokenizer = AssTokenizer::new(source);

    let mut positions = Vec::new();
    positions.push(tokenizer.position());

    while let Ok(Some(_)) = tokenizer.next_token() {
        positions.push(tokenizer.position());
        if positions.len() > 10 {
            break; // Prevent infinite loops
        }
    }

    // Positions should advance correctly for multi-byte chars
    for window in positions.windows(2) {
        if window[1] != window[0] {
            assert!(window[1] > window[0]);
        }
    }
}

#[test]
fn tokenizer_token_push_verification() {
    let source = "Key1=Value1\nKey2=Value2";
    let mut tokenizer = AssTokenizer::new(source);

    let tokens = tokenizer.tokenize_all().unwrap_or_default();

    // Verify tokens were actually pushed to the vector
    assert!(!tokens.is_empty());

    // Verify each token has valid content
    for token in &tokens {
        assert!(
            !token.span.is_empty()
                || token.token_type == crate::tokenizer::tokens::TokenType::Comment
        );
    }
}
