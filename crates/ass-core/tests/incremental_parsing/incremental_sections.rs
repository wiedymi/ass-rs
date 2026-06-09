//! Incremental reparsing tests for individual sections.

use ass_core::parser::ast::Section;
use ass_core::parser::incremental::TextChange;
use ass_core::parser::Script;

#[test]
fn test_parse_incremental_script_info_change() {
    let original_content = r"[Script Info]
Title: Original Title
Author: Original Author
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Hello World";

    let script = Script::parse(original_content).unwrap();
    assert_eq!(script.sections().len(), 3);

    // Change the title
    let change = TextChange {
        range: 14..28, // "Original Title"
        new_text: "Modified Title".to_string(),
        line_range: 2..2,
    };

    let new_content = original_content.replace("Original Title", "Modified Title");
    let new_script = script.parse_incremental(&new_content, &change).unwrap();

    // Verify the change was applied
    assert_eq!(new_script.sections().len(), 3);

    // Check that only the ScriptInfo section was reparsed
    if let Section::ScriptInfo(info) = &new_script.sections()[0] {
        let title_field = info
            .fields
            .iter()
            .find(|(key, _)| *key == "Title")
            .expect("Title field should exist");
        assert_eq!(title_field.1, "Modified Title");
    } else {
        panic!("First section should be ScriptInfo");
    }
}

#[test]
fn test_parse_incremental_event_change() {
    let original_content = r"[Script Info]
Title: Test

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Original text
Dialogue: 0:00:05.00,0:00:10.00,Second line
Dialogue: 0:00:10.00,0:00:15.00,Third line";

    let script = Script::parse(original_content).unwrap();

    // Find the position of "Original text"
    let change_start = original_content.find("Original text").unwrap();
    let change = TextChange {
        range: change_start..change_start + 13,
        new_text: "Modified text".to_string(),
        line_range: 5..5,
    };

    let new_content = original_content.replace("Original text", "Modified text");
    let new_script = script.parse_incremental(&new_content, &change).unwrap();

    // Verify the change
    if let Section::Events(events) = &new_script.sections()[1] {
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].text, "Modified text");
        assert_eq!(events[1].text, "Second line");
        assert_eq!(events[2].text, "Third line");
    } else {
        panic!("Second section should be Events");
    }
}

#[test]
fn test_parse_incremental_style_change() {
    let original_content = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,20
Style: Bold,Arial Bold,24

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let script = Script::parse(original_content).unwrap();

    // Change font size from 20 to 30
    let change_start = original_content.find(",20").unwrap() + 1;
    let change = TextChange {
        range: change_start..change_start + 2,
        new_text: "30".to_string(),
        line_range: 5..5,
    };

    let new_content = original_content.replace(",20", ",30");
    let new_script = script.parse_incremental(&new_content, &change).unwrap();

    // Verify the change
    if let Section::Styles(styles) = &new_script.sections()[1] {
        assert_eq!(styles.len(), 2);
        assert_eq!(styles[0].fontsize, "30");
        assert_eq!(styles[1].fontsize, "24");
    } else {
        panic!("Second section should be Styles");
    }
}

#[test]
fn test_parse_incremental_multiline_change() {
    let original_content = r"[Script Info]
Title: Test
Author: Someone

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Line 1";

    let script = Script::parse(original_content).unwrap();

    // Add a new field after Title
    let change_start = original_content.find("\nAuthor:").unwrap();
    let change = TextChange {
        range: change_start..change_start,
        new_text: "\nVersion: 1.0".to_string(),
        line_range: 2..2,
    };

    let mut new_content = original_content[..change_start].to_string();
    new_content.push_str("\nVersion: 1.0");
    new_content.push_str(&original_content[change_start..]);

    let new_script = script.parse_incremental(&new_content, &change).unwrap();

    // Verify the new field was added
    if let Section::ScriptInfo(info) = &new_script.sections()[0] {
        assert_eq!(info.fields.len(), 3);
        let version_field = info
            .fields
            .iter()
            .find(|(key, _)| *key == "Version")
            .expect("Version field should exist");
        assert_eq!(version_field.1, "1.0");
    } else {
        panic!("First section should be ScriptInfo");
    }
}
