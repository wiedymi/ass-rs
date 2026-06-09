//! Tests for `CharNavigator` creation, advancement, peeking, and bounds.

use crate::tokenizer::scanner::CharNavigator;

#[test]
fn char_navigator_creation() {
    let source = "Hello World";
    let navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.position(), 0);
    assert_eq!(navigator.line(), 1);
    assert_eq!(navigator.column(), 1);
    assert!(!navigator.is_at_end());
}

#[test]
fn char_navigator_advance() {
    let source = "abc";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_char().unwrap(), 'a');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 1);
    assert_eq!(navigator.column(), 2);

    assert_eq!(navigator.peek_char().unwrap(), 'b');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 2);
    assert_eq!(navigator.column(), 3);

    assert_eq!(navigator.peek_char().unwrap(), 'c');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 3);
    assert_eq!(navigator.column(), 4);

    assert!(navigator.is_at_end());
}

#[test]
fn char_navigator_peek_next() {
    let source = "abc";
    let navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_next().unwrap(), 'b');
    assert_eq!(navigator.position(), 0); // Position should not change
}

#[test]
fn char_navigator_peek_at_end() {
    let source = "";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert!(navigator.is_at_end());
    let result = navigator.peek_char();
    assert!(result.is_err());
}

#[test]
fn char_navigator_advance_at_end() {
    let source = "a";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    navigator.advance_char().unwrap(); // Consume 'a'
    assert!(navigator.is_at_end());

    let result = navigator.advance_char();
    assert!(result.is_err());
}

#[test]
fn char_navigator_unicode_characters() {
    let source = "こんにちは世界";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_char().unwrap(), 'こ');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.peek_char().unwrap(), 'ん');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.peek_char().unwrap(), 'に');

    // Position should advance by correct byte count for UTF-8
    assert!(navigator.position() > 2); // Each Japanese char is 3 bytes in UTF-8
}

#[test]
fn char_navigator_emoji() {
    let source = "🎬📽️🎭";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_char().unwrap(), '🎬');
    navigator.advance_char().unwrap();
    assert_eq!(navigator.column(), 2);

    // Should handle emoji correctly
    assert_eq!(navigator.peek_char().unwrap(), '📽');
}

#[test]
fn char_navigator_peek_with_lookahead() {
    let source = "abc";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    // First peek should set up lookahead
    assert_eq!(navigator.peek_char().unwrap(), 'a');

    // Second peek should return same character
    assert_eq!(navigator.peek_char().unwrap(), 'a');

    // Advance should use the lookahead
    navigator.advance_char().unwrap();
    assert_eq!(navigator.position(), 1);

    // Next peek should be next character
    assert_eq!(navigator.peek_char().unwrap(), 'b');
}

#[test]
fn char_navigator_bounds_checking() {
    let source = "a";
    let mut navigator = CharNavigator::new(source, 0, 1, 1);

    assert!(!navigator.is_at_end());
    navigator.advance_char().unwrap();
    assert!(navigator.is_at_end());

    // Further attempts should fail gracefully
    let result = navigator.peek_char();
    assert!(result.is_err());

    let result = navigator.advance_char();
    assert!(result.is_err());
}

#[test]
fn char_navigator_peek_next_at_end() {
    let source = "a";
    let navigator = CharNavigator::new(source, 0, 1, 1);

    let result = navigator.peek_next();
    assert!(result.is_err());
}

#[test]
fn char_navigator_peek_next_with_unicode() {
    let source = "こんにちは";
    let navigator = CharNavigator::new(source, 0, 1, 1);

    assert_eq!(navigator.peek_next().unwrap(), 'ん');
}
