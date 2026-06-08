//! Tests for regex-based search and feature gating

use super::*;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
#[cfg(all(feature = "formats", feature = "std"))]
fn test_regex_search_basic() {
    use crate::core::EditorDocument;

    let doc = EditorDocument::from_content(
        "[Script Info]\nTitle: Test123\nPlayResX: 1920\nPlayResY: 1080",
    )
    .unwrap();

    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    // Test basic regex pattern
    let options = SearchOptions {
        use_regex: true,
        ..Default::default()
    };

    // Search for numbers
    let results = search.search(r"\d+", &options).unwrap();
    assert_eq!(results.len(), 3); // 123, 1920, 1080

    // Search for "Play" followed by any word characters
    let results = search.search(r"Play\w+", &options).unwrap();
    assert_eq!(results.len(), 2); // PlayResX, PlayResY
}

#[test]
#[cfg(all(feature = "formats", feature = "std"))]
fn test_regex_search_case_insensitive() {
    use crate::core::EditorDocument;

    let doc = EditorDocument::from_content("Hello WORLD\nhello world\nHeLLo WoRlD").unwrap();

    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    let options = SearchOptions {
        use_regex: true,
        case_sensitive: false,
        ..Default::default()
    };

    // Case-insensitive regex search
    let results = search.search(r"hello\s+world", &options).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
#[cfg(all(feature = "formats", feature = "std", feature = "search-index"))]
fn test_fst_regex_search() {
    use crate::core::EditorDocument;

    let doc = EditorDocument::from_content(
        "[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test dialogue",
    )
    .unwrap();

    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    let options = SearchOptions {
        use_regex: true,
        ..Default::default()
    };

    // Search for time codes pattern
    let results = search.search(r"\d:\d{2}:\d{2}\.\d{2}", &options).unwrap();
    assert_eq!(results.len(), 2); // Two time codes

    // Verify the matches are correct
    assert_eq!(results[0].text, "0:00:00.00");
    assert_eq!(results[1].text, "0:00:05.00");
}

#[test]
#[cfg(all(feature = "formats", feature = "std"))]
fn test_regex_search_with_scope() {
    use crate::core::EditorDocument;

    let doc =
        EditorDocument::from_content("Line 1: ABC\nLine 2: DEF\nLine 3: ABC\nLine 4: GHI").unwrap();

    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    let options = SearchOptions {
        use_regex: true,
        scope: SearchScope::Lines { start: 1, end: 2 },
        ..Default::default()
    };

    // Search for ABC in lines 1-2 only (0-based: lines at index 1 and 2)
    let results = search.search("ABC", &options).unwrap();
    assert_eq!(results.len(), 1); // ABC is on line 3 (index 2)

    // Search for DEF in lines 1-2
    let results = search.search("DEF", &options).unwrap();
    assert_eq!(results.len(), 1); // DEF is on line 2 (index 1)
}

#[test]
#[cfg(not(all(feature = "formats", feature = "std")))]
fn test_regex_search_feature_disabled() {
    use crate::core::EditorDocument;

    let doc = EditorDocument::from_content("Test content").unwrap();

    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    let options = SearchOptions {
        use_regex: true,
        ..Default::default()
    };

    // Should return error when regex feature is not enabled
    let result = search.search("test", &options);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Regex") || error_msg.contains("regex"),
        "Expected error to contain 'regex', but got: {error_msg}"
    );
}
