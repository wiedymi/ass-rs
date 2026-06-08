//! Chunk-feeding behaviour tests for [`StreamingParser`].

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::String;

#[test]
fn empty_chunk_processing() {
    let mut parser = StreamingParser::new();
    let result = parser.feed_chunk(b"");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn feed_chunk_invalid_utf8() {
    let mut parser = StreamingParser::new();
    let invalid_utf8 = b"\xff\xfe";
    let result = parser.feed_chunk(invalid_utf8);
    assert!(result.is_err());
}

#[test]
fn feed_chunk_complete_lines() {
    let mut parser = StreamingParser::new();
    let chunk = b"[Script Info]\nTitle: Test\n";
    let result = parser.feed_chunk(chunk);
    assert!(result.is_ok());
    assert!(parser.buffer.is_empty());
}

#[test]
fn feed_chunk_partial_lines() {
    let mut parser = StreamingParser::new();

    // Feed partial line without newline
    let chunk1 = b"[Script Info]\nTitle: ";
    parser.feed_chunk(chunk1).unwrap();
    assert_eq!(parser.buffer, "Title: ");

    // Complete the partial line
    let chunk2 = b"Test\n";
    parser.feed_chunk(chunk2).unwrap();
    assert!(parser.buffer.is_empty());
}

#[test]
fn feed_chunk_multiple_calls() {
    let mut parser = StreamingParser::new();

    let chunk1 = b"[Script Info]\n";
    let chunk2 = b"Title: Test\n";
    let chunk3 = b"Author: Someone\n";

    parser.feed_chunk(chunk1).unwrap();
    parser.feed_chunk(chunk2).unwrap();
    parser.feed_chunk(chunk3).unwrap();

    assert!(parser.buffer.is_empty());
}

#[test]
fn feed_chunk_different_line_endings() {
    let mut parser = StreamingParser::new();

    // Unix line endings
    parser.feed_chunk(b"Line1\nLine2\n").unwrap();
    assert!(parser.buffer.is_empty());

    // Windows line endings
    parser.feed_chunk(b"Line3\r\nLine4\r\n").unwrap();
    assert!(parser.buffer.is_empty());

    // Mac line endings
    parser.feed_chunk(b"Line5\rLine6\r").unwrap();
    assert!(parser.buffer.is_empty());
}

#[test]
fn feed_chunk_whitespace_only() {
    let mut parser = StreamingParser::new();
    let result = parser.feed_chunk(b"   \n\t\n  \n");
    assert!(result.is_ok());
    assert!(parser.buffer.is_empty());
}

#[test]
fn feed_chunk_unicode_content() {
    let mut parser = StreamingParser::new();
    let unicode_content = "[Script Info]\nTitle: Unicode Test 测试 🎬\n";
    let result = parser.feed_chunk(unicode_content.as_bytes());
    assert!(result.is_ok());
    assert!(parser.buffer.is_empty());
}

#[test]
fn streaming_large_chunk_comprehensive() {
    #[cfg(not(feature = "std"))]
    use alloc::fmt::Write;
    #[cfg(feature = "std")]
    use std::fmt::Write;

    let mut parser = StreamingParser::new();
    // Create a large chunk
    let mut large_content = String::from("[Script Info]\n");
    for i in 0..1000 {
        writeln!(large_content, "Field{i}: Value{i}").unwrap();
    }

    let result = parser.feed_chunk(large_content.as_bytes());
    assert!(result.is_ok());
    assert!(parser.buffer.is_empty());
}
