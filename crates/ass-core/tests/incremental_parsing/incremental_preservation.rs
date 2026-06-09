//! Incremental parsing tests covering section preservation and cross-section edits.

use ass_core::parser::ast::Section;
use ass_core::parser::incremental::TextChange;
use ass_core::parser::Script;

#[test]
fn test_parse_incremental_preserves_untouched_sections() {
    let original_content = r"[Script Info]
Title: Original
PlayResX: 1280
PlayResY: 720

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&H00FFFFFF
Style: Alt,Times,18,&H00FF0000

[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Line 1
Dialogue: 0,0:00:05.00,0:00:10.00,Alt,Line 2
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Line 3";

    let script = Script::parse(original_content).unwrap();

    // Store original section details
    let original_styles_count = if let Section::Styles(styles) = &script.sections()[1] {
        styles.len()
    } else {
        0
    };

    let original_events_count = if let Section::Events(events) = &script.sections()[2] {
        events.len()
    } else {
        0
    };

    // Change only the title in ScriptInfo
    let title_start = original_content.find("Original").unwrap();
    let change = TextChange {
        range: title_start..title_start + 8,
        new_text: "Modified".to_string(),
        line_range: 2..2,
    };

    let new_content = original_content.replace("Original", "Modified");
    let new_script = script.parse_incremental(&new_content, &change).unwrap();

    // Verify all sections are present
    assert_eq!(new_script.sections().len(), 3);

    // Verify ScriptInfo was updated
    if let Section::ScriptInfo(info) = &new_script.sections()[0] {
        let title = info.fields.iter().find(|(key, _)| *key == "Title").unwrap();
        assert_eq!(title.1, "Modified");
    }

    // Verify Styles section is unchanged
    if let Section::Styles(styles) = &new_script.sections()[1] {
        assert_eq!(styles.len(), original_styles_count);
        assert_eq!(styles[0].name, "Default");
        assert_eq!(styles[1].name, "Alt");
    }

    // Verify Events section is unchanged
    if let Section::Events(events) = &new_script.sections()[2] {
        assert_eq!(events.len(), original_events_count);
        assert_eq!(events[0].text, "Line 1");
        assert_eq!(events[1].text, "Line 2");
        assert_eq!(events[2].text, "Line 3");
    }
}

#[test]
fn test_parse_incremental_cross_section_change() {
    let original_content = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial";

    let script = Script::parse(original_content).unwrap();

    // Try to change text that spans across sections (should reparse both)
    let change_start = original_content.find("Test\n\n[V4+").unwrap() + 4; // After "Test"
    let change = TextChange {
        range: change_start..change_start + 2, // Replace "\n\n"
        new_text: "\nAuthor: Someone\n\n".to_string(),
        line_range: 2..4,
    };

    let mut new_content = original_content.to_string();
    new_content.replace_range(change.range.clone(), &change.new_text);

    let new_script = script.parse_incremental(&new_content, &change).unwrap();

    // Both sections should be present and updated
    assert_eq!(new_script.sections().len(), 2);

    if let Section::ScriptInfo(info) = &new_script.sections()[0] {
        assert_eq!(info.fields.len(), 2); // Title and Author
    }
}
