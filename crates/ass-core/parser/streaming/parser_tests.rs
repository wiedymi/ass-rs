//! Construction, buffering, and reset tests for [`StreamingParser`].

use super::*;

#[test]
fn streaming_parser_creation() {
    let parser = StreamingParser::new();
    assert_eq!(parser.sections.len(), 0);
}

#[test]
fn partial_line_handling() {
    let mut parser = StreamingParser::new();

    // Feed partial line
    let chunk1 = b"[Script ";
    parser.feed_chunk(chunk1).unwrap();
    assert_eq!(parser.buffer, "[Script ");

    // Complete the line
    let chunk2 = b"Info]\n";
    parser.feed_chunk(chunk2).unwrap();
    assert!(parser.buffer.is_empty());
}

#[test]
fn streaming_parser_with_capacity() {
    let parser = StreamingParser::with_capacity(100);
    assert_eq!(parser.sections.len(), 0);
    assert!(parser.sections.capacity() >= 100);
}

#[test]
fn streaming_parser_default() {
    let parser = StreamingParser::default();
    assert_eq!(parser.sections.len(), 0);
}

#[test]
fn reset_functionality() {
    let mut parser = StreamingParser::new();

    // Add some content
    parser.feed_chunk(b"[Script Info]\nTitle: ").unwrap();
    assert!(!parser.buffer.is_empty());

    // Reset should clear everything
    parser.reset();
    assert!(parser.buffer.is_empty());
    assert_eq!(parser.sections.len(), 0);
}

#[test]
fn feed_chunk_edge_cases() {
    let mut parser = StreamingParser::new();

    // Single character
    parser.feed_chunk(b"a").unwrap();
    assert_eq!(parser.buffer, "a");

    // Just newline
    parser.feed_chunk(b"\n").unwrap();
    assert!(parser.buffer.is_empty());

    // Empty line
    parser.reset();
    parser.feed_chunk(b"\n").unwrap();
    assert!(parser.buffer.is_empty());
}

#[cfg(feature = "benches")]
#[test]
fn memory_tracking() {
    let mut parser = StreamingParser::new();
    let initial_memory = parser.peak_memory();

    // Feed some content to increase memory usage
    parser.feed_chunk(b"[Script Info]\nTitle: Test\n").unwrap();

    // Memory should be tracked
    assert!(parser.peak_memory() >= initial_memory);
}
