//! ASS format-specific edge case tests
//!
//! Tests focusing on ASS/SSA subtitle format edge cases and parsing scenarios

use ass_editor::*;

// ===== Section Parsing Edge Cases =====

#[test]
fn test_malformed_section_headers() {
    let test_cases = vec![
        "[Script Info",          // Missing closing bracket
        "Script Info]",          // Missing opening bracket
        "[ Script Info ]",       // Extra spaces
        "[SCRIPT INFO]",         // Different case
        "[Script  Info]",        // Double space
        "[]",                    // Empty section
        "[Script Info][Events]", // Multiple sections on one line
        "[\tScript Info\t]",     // Tabs instead of spaces
    ];

    for header in test_cases {
        let content = format!("{header}\nTitle: Test");
        let result = EditorDocument::from_content(&content);

        // Parser should either handle gracefully or fail predictably
        if let Ok(doc) = result {
            // If it parses, content should be preserved
            assert!(doc.text().contains(header));
        }
    }
}

#[test]
fn test_section_order_variations() {
    // ASS files can have sections in different orders
    let variations = vec![
        // Standard order
        "[Script Info]\n[V4+ Styles]\n[Events]\n",
        // Reversed order
        "[Events]\n[V4+ Styles]\n[Script Info]\n",
        // Missing middle section
        "[Script Info]\n[Events]\n",
        // Duplicate sections
        "[Script Info]\n[Script Info]\n[Events]\n",
        // Unknown sections
        "[Script Info]\n[Custom Section]\n[Events]\n",
    ];

    for content in variations {
        let result = EditorDocument::from_content(content);
        if let Ok(doc) = result {
            // Should preserve the original structure
            assert_eq!(doc.text(), content);
        }
    }
}

// ===== Script Info Edge Cases =====

#[test]
fn test_script_info_field_variations() {
    let content = r#"[Script Info]
Title: Test Title
;Comment: This is a comment
title: lowercase field
Title : Extra spaces
Title:No space after colon
Collisions: 1000000
PlayResX:    1920    
PlayResY:	1080	
Custom Field: Custom Value
Title: Duplicate field
: Empty field name
No colon field
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // All content should be preserved
    assert!(doc.text().contains(";Comment"));
    assert!(doc.text().contains("title: lowercase"));
    assert!(doc.text().contains("Title : Extra spaces"));
    assert!(doc.text().contains("Custom Field"));
}

#[test]
fn test_special_characters_in_values() {
    let content = r#"[Script Info]
Title: Special: Characters; Test
Author: Name "Quote" Test
Description: Line1\nLine2\rLine3
Version: v1.0 (测试)
Comment: Emoji 🎭 test
URL: https://example.com/path?query=1&test=2
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // Special characters should be preserved
    assert!(doc.text().contains("Special: Characters; Test"));
    assert!(doc.text().contains(r#"Name "Quote" Test"#));
    assert!(doc.text().contains("🎭"));
    assert!(doc.text().contains("https://example.com"));
}

// ===== Style Section Edge Cases =====

#[test]
fn test_style_format_variations() {
    let content = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Alt,Comic Sans MS,18,&H00FFFF00,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: ,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Space Name ,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // Various style definitions should be preserved
    assert!(doc.text().contains("Comic Sans MS"));
    assert!(doc.text().contains("Style: ,Arial")); // Empty name
    assert!(doc.text().contains("Space Name ,")); // Trailing space in name
}

#[test]
fn test_color_format_variations() {
    let colors = vec![
        "&H00FFFFFF", // Standard ABGR
        "&HFFFFFF",   // Missing alpha
        "&H0xFF00FF", // With 0x prefix
        "&HFFFFFFFF", // Full ABGR
        "&H00",       // Too short
        "&HGGGGGG",   // Invalid hex
        "16777215",   // Decimal
        "0xFFFFFF",   // Hex with 0x
    ];

    for color in colors {
        let content = format!(
            "[V4+ Styles]\nFormat: Name, PrimaryColour\nStyle: Test,{color}"
        );

        let result = EditorDocument::from_content(&content);
        if let Ok(doc) = result {
            assert!(doc.text().contains(color));
        }
    }
}

// ===== Events Section Edge Cases =====

#[test]
fn test_dialogue_field_edge_cases() {
    let content = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,Normal text
Dialogue: -1,0:00:01.00,0:00:02.00,Default,,0,0,0,,Negative layer
Dialogue: 999,0:00:02.00,0:00:03.00,Default,,0,0,0,,High layer
Dialogue: 0,99:99:99.99,99:99:99.99,Default,,0,0,0,,Invalid time
Dialogue: 0,0:00:05.00,0:00:04.00,Default,,0,0,0,,End before start
Dialogue: 0,0:00:06.00,0:00:07.00,NonExistent,,0,0,0,,Missing style
Dialogue: 0,0:00:08.00,0:00:09.00,Default,Actor Name,0,0,0,,With actor
Dialogue: 0,0:00:10.00,0:00:11.00,Default,,-100,-200,-300,,Negative margins
Comment: 0,0:00:12.00,0:00:13.00,Default,,0,0,0,,This is commented
Dialogue: 0,0:00:14.00,0:00:15.00,Default,,0,0,0,Karaoke,Effect field used
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // All lines should be preserved
    assert!(doc.text().contains("Negative layer"));
    assert!(doc.text().contains("99:99:99.99"));
    assert!(doc.text().contains("NonExistent"));
    assert!(doc.text().contains("Actor Name"));
    assert!(doc.text().contains("Comment:"));
}

#[test]
fn test_dialogue_text_special_cases() {
    let test_texts = vec![
        r"Plain text",
        r"Text with\Nhard break",
        r"Text with\nsoft break",
        r"{\an8}Top aligned",
        r"{\pos(100,200)}Positioned",
        r"{\1c&HFF0000&}Blue text",
        r"Multiple{\i1}tags{\i0}here",
        r"Nested{\b1\i1}tags{\b0\i0}",
        r"Escaped \{ brace",
        r"Unicode: 你好 мир 🌍",
        r"Empty tags{}text",
        r"{\}Malformed tag",
        r"{No closing brace",
        r"Line1\NLine2\NLine3\N\N\NMany breaks",
        r"Special chars: <>&",
        r"Tab	character",
        r"", // Empty text
    ];

    let mut content = String::from("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for (i, text) in test_texts.iter().enumerate() {
        content.push_str(&format!(
            "Dialogue: 0,0:{i:02}:00.00,0:{i:02}:01.00,Default,,0,0,0,,{text}\n"
        ));
    }

    let doc = EditorDocument::from_content(&content).unwrap();

    // All special text cases should be preserved
    for text in test_texts {
        if !text.is_empty() {
            assert!(doc.text().contains(text));
        }
    }
}

#[test]
fn test_karaoke_timing_tags() {
    let content = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\k100}Simple{\k50}Karaoke
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,{\K100}Karaoke{\kf200}Fill{\ko300}Outline
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,Pre{\k50}Mid{\k0}Zero{\k-50}Negative
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // Karaoke tags should be preserved
    assert!(doc.text().contains(r"{\k100}"));
    assert!(doc.text().contains(r"{\K100}"));
    assert!(doc.text().contains(r"{\kf200}"));
    assert!(doc.text().contains(r"{\ko300}"));
    assert!(doc.text().contains(r"{\k0}"));
    assert!(doc.text().contains(r"{\k-50}"));
}

// ===== Time Format Edge Cases =====

#[test]
fn test_time_format_variations() {
    let times = vec![
        "0:00:00.00",   // Standard
        "00:00:00.00",  // Leading zero hour
        "0:0:0.0",      // Minimal digits
        "0:00:00.000",  // Three decimal places
        "9:59:59.99",   // Max single digit hour
        "10:00:00.00",  // Double digit hour
        "100:00:00.00", // Triple digit hour
        "0:60:00.00",   // Invalid minutes
        "0:00:60.00",   // Invalid seconds
        "0:00:00.100",  // Three decimal places
        "0:00:00",      // No decimals
        "-0:00:01.00",  // Negative time
    ];

    let mut content = String::from("[Events]\nFormat: Start, End, Text\n");

    for (i, time) in times.iter().enumerate() {
        content.push_str(&format!(
            "Dialogue: {time},0:{:02}:00.00,Time test {i}\n",
            i + 1
        ));
    }

    let result = EditorDocument::from_content(&content);
    if let Ok(doc) = result {
        // Parser should handle or preserve all time formats
        for time in times {
            assert!(doc.text().contains(time));
        }
    }
}

// ===== Line Ending and Encoding Edge Cases =====

#[test]
fn test_mixed_line_endings_in_ass() {
    let content = "[Script Info]\rTitle: CR Line\n[V4+ Styles]\r\nFormat: Name\r\nStyle: Default\n\r[Events]\nDialogue: Text\r";

    let doc = EditorDocument::from_content(content).unwrap();

    // Content should be preserved with original line endings
    assert!(doc.text().contains("Title: CR Line"));
    assert!(doc.text().contains("Style: Default"));
    assert!(doc.text().contains("Dialogue: Text"));
}

#[test]
fn test_bom_handling() {
    // UTF-8 BOM
    let bom = "\u{FEFF}";
    let content = format!("{bom}[Script Info]\nTitle: BOM Test");

    let result = EditorDocument::from_content(&content);
    if let Ok(doc) = result {
        // BOM might be stripped or preserved
        assert!(doc.text().contains("Title: BOM Test"));
    }
}

// ===== Large Field Values =====

#[test]
fn test_extremely_long_field_values() {
    let long_title = "A".repeat(10000);
    let long_text = "B".repeat(10000);

    let content = format!(
        "[Script Info]\nTitle: {long_title}\n[Events]\nDialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,{long_text}"
    );

    let doc = EditorDocument::from_content(&content).unwrap();

    // Long values should be preserved
    assert!(doc.text().contains(&long_title));
    assert!(doc.text().contains(&long_text));
}

// ===== Comment Handling =====

#[test]
fn test_comment_variations() {
    let content = r#"[Script Info]
; Standard comment
;No space after semicolon
    ; Indented comment
Title: Test ; Inline comment
; Multi-line comment \
  continuation

[Events]
Comment: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,Commented dialogue
Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,Normal ; with semicolon
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // All comment variations should be preserved
    assert!(doc.text().contains("; Standard comment"));
    assert!(doc.text().contains(";No space"));
    assert!(doc.text().contains("    ; Indented"));
    assert!(doc.text().contains("Test ; Inline"));
    assert!(doc.text().contains("Comment: 0,"));
}

// ===== Format Compatibility =====

#[test]
fn test_ssa_v4_compatibility() {
    // SSA v4 format (predecessor to ASS)
    let content = r#"[Script Info]
Title: SSA v4 Test
ScriptType: v4.00

[V4 Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, TertiaryColour, BackColour, Bold, Italic, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, AlphaLevel, Encoding
Style: Default,Arial,20,16777215,16777215,0,0,0,0,1,2,0,2,10,10,10,0,1

[Events]
Format: Marked, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: Marked=0,0:00:00.00,0:00:01.00,Default,,0,0,0,,SSA text
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // SSA format elements should be preserved
    assert!(doc.text().contains("ScriptType: v4.00"));
    assert!(doc.text().contains("[V4 Styles]"));
    assert!(doc.text().contains("TertiaryColour"));
    assert!(doc.text().contains("Marked=0"));
}

// ===== Error Recovery =====

#[test]
fn test_recovery_from_malformed_lines() {
    let content = r#"[Script Info]
Title: Test
This is not a valid line
Another invalid line without colon
:
Key:
:Value

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:01.00,Normal
Not enough fields
Dialogue: Too,Many,Fields,Here,Extra,More,Fields
Dialogue: 0:00:02.00,0:00:03.00,"Quoted text"
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // Should preserve all content, even malformed lines
    assert!(doc.text().contains("This is not a valid line"));
    assert!(doc.text().contains("Not enough fields"));
    assert!(doc.text().contains("Too,Many,Fields"));
}

#[test]
fn test_circular_references() {
    // Test styles or other references that might create cycles
    let content = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Style1,Arial,20
Style: Style2,{=Style1}Arial,20
Style: Style3,{=Style2}Arial,20
Style: Style1,{=Style3}Arial,20

[Events]
Dialogue: 0,0:00:00.00,0:00:01.00,Style1,,0,0,0,,Text1
Dialogue: 0,0:00:01.00,0:00:02.00,Style2,,0,0,0,,Text2
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // Should handle potential circular references gracefully
    assert!(doc.text().contains("Style1"));
    assert!(doc.text().contains("Style2"));
    assert!(doc.text().contains("Style3"));
}
