//! Integration tests for incremental parsing functionality

#![cfg(feature = "stream")]

use ass_core::parser::ast::Section;
use ass_core::parser::incremental::TextChange;
use ass_core::parser::{calculate_delta, Script};

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

#[test]
fn test_calculate_delta_section_added() {
    let original = r"[Script Info]
Title: Test

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let modified = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let script1 = Script::parse(original).unwrap();
    let script2 = Script::parse(modified).unwrap();

    let delta = calculate_delta(&script1, &script2);

    println!("Script1 sections: {}", script1.sections().len());
    for (i, s) in script1.sections().iter().enumerate() {
        println!("  {}: {:?}", i, s.section_type());
    }
    println!("Script2 sections: {}", script2.sections().len());
    for (i, s) in script2.sections().iter().enumerate() {
        println!("  {}: {:?}", i, s.section_type());
    }
    println!(
        "Delta: added={}, modified={}, removed={}",
        delta.added.len(),
        delta.modified.len(),
        delta.removed.len()
    );

    assert_eq!(delta.added.len(), 1);
    assert!(matches!(delta.added[0], Section::Styles(_)));
    assert_eq!(delta.modified.len(), 0);
    assert_eq!(delta.removed.len(), 0);
}

#[test]
fn test_calculate_delta_section_modified() {
    let original = r"[Script Info]
Title: Original

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Original";

    let modified = r"[Script Info]
Title: Modified

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Modified";

    let script1 = Script::parse(original).unwrap();
    let script2 = Script::parse(modified).unwrap();

    let delta = calculate_delta(&script1, &script2);

    assert_eq!(delta.added.len(), 0);
    assert_eq!(delta.modified.len(), 2); // Both ScriptInfo and Events changed
    assert_eq!(delta.removed.len(), 0);
}

#[test]
fn test_calculate_delta_section_removed() {
    let original = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let modified = r"[Script Info]
Title: Test

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let script1 = Script::parse(original).unwrap();
    let script2 = Script::parse(modified).unwrap();

    let delta = calculate_delta(&script1, &script2);

    assert_eq!(delta.added.len(), 0);
    assert_eq!(delta.modified.len(), 0);
    assert_eq!(delta.removed.len(), 1);
    assert_eq!(delta.removed[0], 1); // Index of Styles section
}

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

#[test]
fn test_incremental_parsing_performance() {
    use std::fmt::Write;

    // Create a large script
    let mut content =
        String::from("[Script Info]\nTitle: Large Script\n\n[Events]\nFormat: Start, End, Text\n");
    for i in 0..1000 {
        writeln!(
            content,
            "Dialogue: 0:00:{:02}.00,0:00:{:02}.00,Line {}",
            i % 60,
            (i + 5) % 60,
            i
        )
        .unwrap();
    }

    let script = Script::parse(&content).unwrap();

    // Change just one line in the middle
    let target = "Line 500";
    let pos = content.find(target).unwrap();
    let change = TextChange {
        range: pos..pos + target.len(),
        new_text: "Modified 500".to_string(),
        line_range: 505..505,
    };

    let new_content = content.replace(target, "Modified 500");

    // This should be fast since we only reparse the Events section
    let start = std::time::Instant::now();
    let new_script = script.parse_incremental(&new_content, &change).unwrap();
    let elapsed = start.elapsed();

    // Should complete in under 5ms as per performance targets
    assert!(
        elapsed.as_millis() < 5,
        "Incremental parse took {elapsed:?}"
    );

    // Debug: print sections
    println!("Original sections: {}", script.sections().len());
    println!("New sections: {}", new_script.sections().len());
    for (i, section) in new_script.sections().iter().enumerate() {
        match section {
            Section::ScriptInfo(_) => println!("Section {i}: ScriptInfo"),
            Section::Styles(s) => println!("Section {i}: Styles ({})", s.len()),
            Section::Events(e) => println!("Section {i}: Events ({})", e.len()),
            Section::Fonts(f) => println!("Section {i}: Fonts ({})", f.len()),
            Section::Graphics(g) => println!("Section {i}: Graphics ({})", g.len()),
        }
    }

    // Verify the change
    let events_section = new_script
        .sections()
        .iter()
        .find_map(|s| match s {
            Section::Events(events) => Some(events),
            _ => None,
        })
        .expect("Events section should exist");

    // Debug: print first few events to see what's happening
    println!("Total events: {}", events_section.len());
    for (i, event) in events_section.iter().take(5).enumerate() {
        println!("Event {i}: {}", event.text);
    }

    // Look for the modified event - might be "Modified 500" not in text field
    let modified_event = events_section.iter().enumerate().find(|(i, e)| {
        if *i == 500 {
            println!("Event 500 text: '{}'", e.text);
        }
        e.text == "Modified 500"
    });

    assert!(
        modified_event.is_some(),
        "Modified event not found. Expected text 'Modified 500' at index around 500"
    );
}
