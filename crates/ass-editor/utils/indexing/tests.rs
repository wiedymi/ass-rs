//! Tests for the FST and linear search index backends.

use super::*;
use crate::core::EditorDocument;
use crate::utils::search::{DocumentSearch, SearchOptions, SearchScope};

#[test]
fn test_linear_search_index() {
    let content = r#"[Script Info]
Title: Test Movie
Author: Test Author

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello world
Dialogue: 0,0:00:12.00,0:00:15.00,Default,Jane,0,0,0,,How are you"#;

    let document = EditorDocument::from_content(content).unwrap();
    let mut search_index = LinearSearchIndex::new();

    search_index.build_index(&document).unwrap();

    let results = search_index
        .search("Hello", &SearchOptions::default())
        .unwrap();
    assert!(!results.is_empty());

    let stats = search_index.stats();
    assert!(stats.match_count > 0);
}

#[cfg(feature = "search-index")]
#[test]
fn test_fst_search_index() {
    let content = r#"[Script Info]
Title: Test Movie

[Events]  
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello world"#;

    let document = EditorDocument::from_content(content).unwrap();
    let mut search_index = FstSearchIndex::new();

    search_index.build_index(&document).unwrap();

    let results = search_index
        .search("hello", &SearchOptions::default())
        .unwrap();
    assert!(!results.is_empty());

    let stats = search_index.stats();
    assert!(stats.index_size > 0);
}

#[test]
fn test_search_factory() {
    let index = create_search_index();

    let content = "[Script Info]\nTitle: Test";
    let _document = EditorDocument::from_content(content).unwrap();

    // Just test that we can create and use the index
    let stats = index.stats();
    assert_eq!(stats.match_count, 0); // No index built yet
}

#[test]
fn test_scope_filtering() {
    let content = r#"[Script Info]
Title: Test

[Events]
Dialogue: Hello world"#;

    let document = EditorDocument::from_content(content).unwrap();
    let mut index = LinearSearchIndex::new();
    index.build_index(&document).unwrap();

    // Test different scopes
    let line_scope = SearchOptions {
        scope: SearchScope::Lines { start: 0, end: 2 },
        ..Default::default()
    };

    let _results = index.search("Test", &line_scope).unwrap();
    // Should find results in the specified line range
}
