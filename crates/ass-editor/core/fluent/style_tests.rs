//! Tests for style fluent operations.

use crate::core::{EditorDocument, StyleBuilder};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn test_fluent_style_operations() {
    const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Hello world!
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test create style
    doc.styles()
        .create(
            "NewStyle",
            StyleBuilder::new()
                .font("Comic Sans MS")
                .size(24)
                .bold(true),
        )
        .unwrap();

    assert!(doc.text().contains("Style: NewStyle"));
    assert!(doc.text().contains("Comic Sans MS"));

    // Test edit style
    doc.styles()
        .edit("Default")
        .font("Helvetica")
        .size(18)
        .bold(true)
        .apply()
        .unwrap();

    assert!(doc.text().contains("Helvetica"));
    assert!(doc.text().contains("18"));

    // Test clone style
    doc.styles().clone("Default", "DefaultCopy").unwrap();

    assert!(doc.text().contains("Style: DefaultCopy"));

    // Test apply style to events
    doc.styles().apply("Default", "NewStyle").apply().unwrap();

    // The dialogue event should now use NewStyle
    let text = doc.text();
    let events_section = text.split("[Events]").nth(1).unwrap();
    assert!(events_section.contains("NewStyle"));
}

#[test]
fn test_fluent_style_delete() {
    const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: ToDelete,Times,22,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Verify style exists
    assert!(doc.text().contains("Style: ToDelete"));

    // Delete the style
    doc.styles().delete("ToDelete").unwrap();

    // Verify style is gone
    assert!(!doc.text().contains("Style: ToDelete"));
    assert!(doc.text().contains("Style: Default")); // Other styles should remain
}

#[test]
fn test_fluent_style_apply_with_filter() {
    const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: FilterStyle,Times,22,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Hello world!
Dialogue: 0,0:00:06.00,0:00:10.00,Default,Speaker,0,0,0,,Goodbye world!
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Apply style only to events containing "Hello"
    doc.styles()
        .apply("Default", "FilterStyle")
        .with_filter("Hello")
        .apply()
        .unwrap();

    let content = doc.text();
    let lines: Vec<&str> = content.lines().collect();

    // Find the dialogue lines
    let hello_line = lines.iter().find(|line| line.contains("Hello")).unwrap();
    let goodbye_line = lines.iter().find(|line| line.contains("Goodbye")).unwrap();

    // Only the "Hello" line should use FilterStyle
    assert!(hello_line.contains("FilterStyle"));
    assert!(goodbye_line.contains("Default")); // Should still use Default
}
