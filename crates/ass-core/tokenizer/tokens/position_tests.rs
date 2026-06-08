//! Unit tests for the [`TokenPosition`] cursor.

use super::*;

#[test]
fn token_position_advance() {
    let mut pos = TokenPosition::start();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);

    pos = pos.advance('a');
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 2);
    assert_eq!(pos.offset, 1);

    pos = pos.advance('\n');
    assert_eq!(pos.line, 2);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 2);
}

#[test]
fn token_position_advance_string() {
    let pos = TokenPosition::start();
    let new_pos = pos.advance_by_str("hello\nworld");

    assert_eq!(new_pos.line, 2);
    assert_eq!(new_pos.column, 6); // "world" = 5 chars + 1
    assert_eq!(new_pos.offset, 11); // "hello\nworld".len()
}

#[test]
fn token_position_edge_cases() {
    let pos = TokenPosition::new(100, 50, 25);
    assert_eq!(pos.offset, 100);
    assert_eq!(pos.line, 50);
    assert_eq!(pos.column, 25);

    // Test default implementation
    let default_pos = TokenPosition::default();
    assert_eq!(default_pos.offset, 0);
    assert_eq!(default_pos.line, 1);
    assert_eq!(default_pos.column, 1);

    // Test start method
    let start_pos = TokenPosition::start();
    assert_eq!(start_pos.offset, 0);
    assert_eq!(start_pos.line, 1);
    assert_eq!(start_pos.column, 1);
}

#[test]
fn token_position_unicode_advance() {
    let mut pos = TokenPosition::start();

    // Test multibyte UTF-8 character
    pos = pos.advance('🎵'); // 4-byte UTF-8 character
    assert_eq!(pos.offset, 4);
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 2);

    // Test combination of ASCII and Unicode
    pos = pos.advance('a');
    assert_eq!(pos.offset, 5);
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 3);

    // Test newline after Unicode
    pos = pos.advance('\n');
    assert_eq!(pos.offset, 6);
    assert_eq!(pos.line, 2);
    assert_eq!(pos.column, 1);
}

#[test]
fn token_position_advance_by_str_edge_cases() {
    let pos = TokenPosition::start();

    // Empty string
    let pos = pos.advance_by_str("");
    assert_eq!(pos.offset, 0);
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);

    // String with only newlines
    let pos = pos.advance_by_str("\n\n\n");
    assert_eq!(pos.offset, 3);
    assert_eq!(pos.line, 4);
    assert_eq!(pos.column, 1);

    // Mixed content with Unicode
    let pos = pos.advance_by_str("hello🎵world\ntest");
    assert_eq!(pos.offset, 3 + 19); // previous 3 + "hello🎵world\ntest".len()
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 5); // "test".len() + 1
}
