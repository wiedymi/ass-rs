//! Unit tests for search option types and index lifecycle

use super::*;
use crate::core::Position;

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, string::ToString, vec};

#[test]
fn search_options_default() {
    let options = SearchOptions::default();
    assert!(!options.case_sensitive);
    assert!(!options.whole_words);
    assert_eq!(options.max_results, 100);
    assert!(!options.use_regex);
    assert_eq!(options.scope, SearchScope::All);
}

#[test]
fn search_result_creation() {
    let result = SearchResult {
        start: Position::new(0),
        end: Position::new(5),
        text: Cow::Borrowed("hello"),
        context: Cow::Borrowed("hello world"),
        line: 0,
        column: 0,
    };

    assert_eq!(result.text, "hello");
    assert_eq!(result.line, 0);
    assert_eq!(result.column, 0);
}

#[test]
fn document_search_creation() {
    let search = DocumentSearchImpl::new();
    let stats = search.stats();
    assert_eq!(stats.index_size, 0);
    assert_eq!(search.document_version, 0);
}

#[test]
fn search_scope_variants() {
    let scope_all = SearchScope::All;
    let scope_lines = SearchScope::Lines { start: 0, end: 10 };
    let scope_sections = SearchScope::Sections(vec!["Events".to_string()]);

    assert_eq!(scope_all, SearchScope::All);
    assert!(matches!(scope_lines, SearchScope::Lines { .. }));
    assert!(matches!(scope_sections, SearchScope::Sections(_)));
}

#[test]
fn search_cache_settings() {
    let mut search = DocumentSearchImpl::new();
    assert_eq!(search.max_cache_entries, 100);
    search.set_cache_size(50);
    assert_eq!(search.max_cache_entries, 50);
}

#[test]
fn create_search_factory() {
    let search = create_search();
    let stats = search.stats();
    assert_eq!(stats.match_count, 0);
    assert_eq!(stats.search_time_us, 0);
    assert!(!stats.hit_limit);
}

#[test]
#[cfg(feature = "search-index")]
fn test_incremental_index_updates() {
    use crate::core::{EditorDocument, Range};

    // Create a document with initial content
    let mut doc = EditorDocument::from_content(
        "[Script Info]\nTitle: Test Search\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello world"
    ).unwrap();

    // Create a search index and build initial index
    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    // Search for "hello" - should find it
    let results = search.search("hello", &SearchOptions::default()).unwrap();
    assert_eq!(results.len(), 1);

    // Actually modify the document - change "Hello" to "Goodbye"
    let hello_pos = doc.text().find("Hello").unwrap();
    let change_range = Range::new(Position::new(hello_pos), Position::new(hello_pos + 5));

    // Delete "Hello" and insert "Goodbye"
    doc.delete(change_range).unwrap();
    doc.insert(Position::new(hello_pos), "Goodbye").unwrap();

    // Update the index incrementally with the change
    search.update_index(&doc, &[change_range]).unwrap();

    // Search for "hello" - should not find it anymore
    let results = search.search("hello", &SearchOptions::default()).unwrap();
    assert_eq!(results.len(), 0);

    // Search for "goodbye" - should find it
    let results = search.search("goodbye", &SearchOptions::default()).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_simple_document_search_rebuild() {
    use crate::core::{EditorDocument, Range};

    let mut doc =
        EditorDocument::from_content("The quick brown fox jumps over the lazy dog.").unwrap();

    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    // Verify initial search works
    let results = search.search("fox", &SearchOptions::default()).unwrap();
    assert_eq!(results.len(), 1);

    // Modify document
    let fox_pos = doc.text().find("fox").unwrap();
    let change_range = Range::new(Position::new(fox_pos), Position::new(fox_pos + 3));
    doc.replace(change_range, "cat").unwrap();

    // Update index with the change
    search.update_index(&doc, &[change_range]).unwrap();

    // Now search for "fox" should find nothing
    let results = search.search("fox", &SearchOptions::default()).unwrap();
    assert_eq!(results.len(), 0);

    // And "cat" should be found
    let results = search.search("cat", &SearchOptions::default()).unwrap();
    assert_eq!(results.len(), 1);
}
