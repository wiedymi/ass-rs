//! Finish, [`StreamingResult`], and source-rebuild tests.

use super::*;
use crate::ScriptVersion;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec, vec::Vec};

#[test]
fn finish_with_empty_buffer() {
    let parser = StreamingParser::new();
    let result = parser.finish();
    assert!(result.is_ok());

    let streaming_result = result.unwrap();
    assert_eq!(streaming_result.sections().len(), 0);
    assert_eq!(streaming_result.version(), ScriptVersion::AssV4);
    assert_eq!(streaming_result.issues().len(), 0);
}

#[test]
fn finish_with_buffered_content() {
    let mut parser = StreamingParser::new();

    // Feed content without final newline
    parser.feed_chunk(b"[Script Info]\nTitle: Test").unwrap();
    assert!(!parser.buffer.is_empty());

    let result = parser.finish();
    assert!(result.is_ok());
}

#[test]
fn streaming_result_accessors() {
    let result = StreamingResult {
        sections: vec!["Section1".to_string(), "Section2".to_string()],
        version: ScriptVersion::AssV4,
        issues: Vec::new(),
    };

    assert_eq!(result.sections().len(), 2);
    assert_eq!(result.sections()[0], "Section1");
    assert_eq!(result.version(), ScriptVersion::AssV4);
    assert_eq!(result.issues().len(), 0);
}

#[test]
fn streaming_result_debug_clone() {
    let result = StreamingResult {
        sections: vec!["Test".to_string()],
        version: ScriptVersion::AssV4,
        issues: Vec::new(),
    };

    let debug_str = format!("{result:?}");
    assert!(debug_str.contains("StreamingResult"));

    let cloned = result.clone();
    assert_eq!(cloned.sections().len(), result.sections().len());
    assert_eq!(cloned.version(), result.version());
}

#[test]
fn build_modified_source_basic() {
    let original = "Hello World";
    let result = build_modified_source(original, 0..5, "Hi");
    assert_eq!(result, "Hi World");

    // Test replacing in the middle
    let result = build_modified_source(original, 6..11, "Universe");
    assert_eq!(result, "Hello Universe");

    // Test replacing entire string
    let result = build_modified_source(original, 0..11, "Goodbye");
    assert_eq!(result, "Goodbye");
}
