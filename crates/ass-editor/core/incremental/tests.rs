//! Unit tests for the incremental parser and document change tracking.

use super::*;
use crate::core::{Position, Range};

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
#[test]
fn test_incremental_parser_creation() {
    let parser = IncrementalParser::new();
    assert!(parser.cached_script.is_none());
    assert!(parser.pending_changes.is_empty());
    assert_eq!(parser.bytes_changed, 0);
}

#[test]
fn test_document_change_tracking() {
    let change = DocumentChange {
        range: Range::new(Position::new(0), Position::new(5)),
        new_text: Cow::Borrowed("Hello"),
        old_text: Cow::Borrowed("World"),
        #[cfg(feature = "std")]
        timestamp: std::time::Instant::now(),
        change_id: 1,
    };

    assert_eq!(change.new_text, "Hello");
    assert_eq!(change.old_text, "World");
    assert_eq!(change.change_id, 1);
}

#[test]
fn test_should_reparse_threshold() {
    let mut parser = IncrementalParser::new();
    parser.set_reparse_threshold(100);

    assert!(!parser.should_reparse());

    parser.bytes_changed = 101;
    assert!(parser.should_reparse());
}

#[test]
fn test_clear_cache() {
    let mut parser = IncrementalParser::new();
    parser.cached_script = Some("test".to_string());
    parser.bytes_changed = 100;
    parser.next_change_id = 5;

    parser.clear_cache();

    assert!(parser.cached_script.is_none());
    assert_eq!(parser.bytes_changed, 0);
    assert_eq!(parser.next_change_id, 1);
}

#[test]
fn test_error_recovery() {
    let mut parser = IncrementalParser::new();

    // Test full reparse on first use (no cached script)
    let content = "[Script Info]\nTitle: Test";
    let result = parser.apply_change(
        content,
        Range::new(Position::new(0), Position::new(5)),
        "New",
    );
    assert!(result.is_ok());
    assert!(parser.cached_script.is_some());

    // Test threshold-based full reparse
    parser.set_reparse_threshold(10);
    parser.bytes_changed = 11;
    let result = parser.apply_change(
        content,
        Range::new(Position::new(0), Position::new(5)),
        "Changed",
    );
    assert!(result.is_ok());
    assert_eq!(parser.bytes_changed, 0); // Reset after full reparse
}
