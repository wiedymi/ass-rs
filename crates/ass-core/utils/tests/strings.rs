//! String trimming, CSV splitting, and line-ending tests for the utils module.

use crate::utils::{normalize_line_endings, split_csv_line, trim_ass_whitespace};
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn string_trimming() {
    assert_eq!(trim_ass_whitespace("  hello  "), "hello");
    assert_eq!(trim_ass_whitespace("\thello\t"), "hello");
    assert_eq!(trim_ass_whitespace("\nhello\n"), "hello");
    assert_eq!(trim_ass_whitespace(" \t\nhello\n\t "), "hello");
}

#[test]
fn string_trimming_unicode() {
    assert_eq!(trim_ass_whitespace("  こんにちは  "), "こんにちは");
    assert_eq!(trim_ass_whitespace("\t🎵\t"), "🎵");
}

#[test]
fn string_splitting_csv() {
    let result = split_csv_line("a,b,c");
    assert_eq!(result, vec!["a", "b", "c"]);

    let result = split_csv_line("field1, field2 , field3");
    assert_eq!(result, vec!["field1", "field2", "field3"]);
}

#[test]
fn string_splitting_csv_empty() {
    let result = split_csv_line("");
    assert_eq!(result, vec![""]);

    let result = split_csv_line(",");
    assert_eq!(result, vec!["", ""]);

    let result = split_csv_line("a,,c");
    assert_eq!(result, vec!["a", "", "c"]);
}

#[test]
fn string_splitting_csv_quoted() {
    let result = split_csv_line("\"quoted field\",normal,\"another quoted\"");
    assert_eq!(result, vec!["quoted field", "normal", "another quoted"]);

    let result = split_csv_line("\"field with, comma\",other");
    assert_eq!(result, vec!["field with, comma", "other"]);
}

#[test]
fn line_ending_normalization() {
    assert_eq!(normalize_line_endings("line1\r\nline2"), "line1\nline2");
    assert_eq!(normalize_line_endings("line1\rline2"), "line1\nline2");
    assert_eq!(normalize_line_endings("line1\nline2"), "line1\nline2");

    let mixed = "line1\r\nline2\rline3\nline4";
    assert_eq!(normalize_line_endings(mixed), "line1\nline2\nline3\nline4");
}
