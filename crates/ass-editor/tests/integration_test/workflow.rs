//! End-to-end workflow and persistence integration tests.

use ass_editor::EditorDocument;

#[test]
fn test_complex_workflow_integration() {
    // Test a complex workflow combining multiple features
    let base_content = r#"[Script Info]
Title: Original Subtitle
ScriptType: v4.00+
PlayResX: 640
PlayResY: 480

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Line 1
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Line 2
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Comment line
Dialogue: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Line 3
"#;

    let mut doc = EditorDocument::from_content(base_content).unwrap();

    // Step 1: Update script info
    doc.info().title("Enhanced Subtitle").unwrap();
    doc.info().resolution(1920, 1080).unwrap();
    doc.info().set("Collisions", "Reverse").unwrap();

    // Step 2: Add embedded fonts
    let font_data = vec![
        "begin 644 custom_font.ttf".to_string(),
        "MABCDEF1234567890".to_string(),
        "end".to_string(),
    ];
    doc.fonts().add("custom_font.ttf", font_data).unwrap();

    // Step 3: Add embedded graphics
    let logo_data = b"PNG logo data here";
    doc.graphics()
        .add_binary("subtitle_logo.png", logo_data)
        .unwrap();

    // Step 4: Modify events
    doc.events().delete_query().comments().unwrap(); // Remove all comments
    assert_eq!(doc.events().count().unwrap(), 3);

    // Verify the complete document state
    let final_text = doc.text();

    // Check Script Info updates
    assert!(final_text.contains("Title: Enhanced Subtitle"));
    assert!(final_text.contains("PlayResX: 1920"));
    assert!(final_text.contains("PlayResY: 1080"));
    assert!(final_text.contains("Collisions: Reverse"));

    // Check Fonts section
    assert!(final_text.contains("[Fonts]"));
    assert!(final_text.contains("fontname: custom_font.ttf"));

    // Check Graphics section
    assert!(final_text.contains("[Graphics]"));
    assert!(final_text.contains("filename: subtitle_logo.png"));

    // Check Events (no comments)
    assert!(!final_text.contains("Comment line"));
    assert_eq!(doc.events().dialogues().execute().unwrap().len(), 3);

    // Test undo capability
    doc.undo().unwrap(); // Undo comment deletion
    assert_eq!(doc.events().count().unwrap(), 4);
    assert!(doc.text().contains("Comment line"));
}

#[test]
fn test_document_persistence_with_new_sections() {
    // Test that fonts and graphics sections persist correctly through save/load
    let content = r"[Script Info]
Title: Default

[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,20

[Events]
Format: Layer, Start, End, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default text
";
    let mut doc = EditorDocument::from_content(content).unwrap();

    // Add all types of content
    doc.info().title("Persistence Test").unwrap();
    doc.info().set("Custom Field", "Custom Value").unwrap();

    doc.fonts().add_binary("test.ttf", b"font data").unwrap();
    doc.graphics()
        .add_binary("test.png", b"image data")
        .unwrap();

    // Get the document as string
    let saved_content = doc.text().to_string();

    // Create a new document from the saved content
    let mut restored_doc = EditorDocument::from_content(&saved_content).unwrap();

    // Verify all content was preserved
    assert_eq!(
        restored_doc.info().get_title().unwrap(),
        Some("Persistence Test".to_string())
    );
    assert_eq!(
        restored_doc.info().get("Custom Field").unwrap(),
        Some("Custom Value".to_string())
    );

    assert_eq!(restored_doc.fonts().list().unwrap().len(), 1);
    assert!(restored_doc
        .fonts()
        .list()
        .unwrap()
        .contains(&"test.ttf".to_string()));

    assert_eq!(restored_doc.graphics().list().unwrap().len(), 1);
    assert!(restored_doc
        .graphics()
        .list()
        .unwrap()
        .contains(&"test.png".to_string()));

    // The text should be identical
    assert_eq!(doc.text(), restored_doc.text());
}
