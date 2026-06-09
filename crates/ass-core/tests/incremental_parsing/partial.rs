//! Tests for the `parse_partial` range-based reparsing API.

use ass_core::parser::Script;

#[test]
fn test_parse_partial_title_change() {
    let original_content = r"[Script Info]
Title: Original Title
Author: Test Author

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let script = Script::parse(original_content).unwrap();

    // Use parse_partial to change the title
    let title_start = original_content.find("Original Title").unwrap();
    let title_end = title_start + "Original Title".len();

    let delta = script
        .parse_partial(title_start..title_end, "New Title")
        .unwrap();

    // Verify delta contains the changes
    assert!(!delta.modified.is_empty());
    assert!(delta.added.is_empty());
    assert!(delta.removed.is_empty());
}

#[test]
fn test_parse_partial_add_event() {
    let original_content = r"[Script Info]
Title: Test

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,First";

    let script = Script::parse(original_content).unwrap();

    // Add a new dialogue line at the end
    let insert_pos = original_content.len();
    let new_line = "\nDialogue: 0:00:05.00,0:00:10.00,Second";

    let delta = script
        .parse_partial(insert_pos..insert_pos, new_line)
        .unwrap();

    // The Events section should be marked as modified
    assert!(!delta.modified.is_empty());
}
