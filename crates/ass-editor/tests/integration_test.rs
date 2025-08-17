//! Integration tests for ass-editor components
//!
//! These tests are designed to work regardless of features enabled.
//! Feature-specific functionality is tested only when the feature is available.

use ass_editor::*;

#[test]
fn test_basic_document_operations() {
    // These should always work
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    // Basic operations
    doc.insert(Position::new(doc.len_bytes()), "\nAuthor: Test")
        .unwrap();
    assert!(doc.text().contains("Author: Test"));

    // Undo/redo
    doc.undo().unwrap();
    assert!(!doc.text().contains("Author: Test"));
    doc.redo().unwrap();
    assert!(doc.text().contains("Author: Test"));
}

#[test]
fn test_basic_editing() {
    let mut doc = EditorDocument::new();

    // Insert
    doc.insert(Position::new(0), "Hello World").unwrap();
    assert_eq!(doc.text(), "Hello World");

    // Delete
    doc.delete(Range::new(Position::new(0), Position::new(5)))
        .unwrap();
    assert_eq!(doc.text(), " World");

    // Replace
    doc.replace(Range::new(Position::new(0), Position::new(6)), "Goodbye")
        .unwrap();
    assert_eq!(doc.text(), "Goodbye");
}

#[test]
fn test_extension_manager_basic() {
    let manager = ExtensionManager::new();
    let extensions = manager.list_extensions();
    assert_eq!(extensions.len(), 0);
}

// Advanced tests that require specific features
#[cfg(all(feature = "std", feature = "multi-thread", feature = "plugins"))]
mod advanced_tests {
    use super::*;
    use ass_editor::events::DocumentEvent;
    use std::sync::mpsc;
    use std::thread;
    // Thread-safe managers are now unified - no need for separate imports

    #[test]
    fn test_extension_manager_with_document_integration() {
        let mut manager = ExtensionManager::new();
        let mut doc = EditorDocument::from_content(
            "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello"
        ).unwrap();

        let mut context = manager
            .create_context("test_extension".to_string(), Some(&mut doc))
            .unwrap();

        context
            .show_message("Test message", MessageLevel::Info)
            .unwrap();
        context
            .set_config("test_key".to_string(), "test_value".to_string())
            .unwrap();
        assert_eq!(
            context.get_config("test_key"),
            Some("test_value".to_string())
        );

        assert!(context.current_document().is_some());
        if let Some(doc) = context.current_document_mut() {
            doc.insert(Position::new(0), "# ").unwrap();
        }
    }

    #[test]
    fn test_session_manager_with_extensions_integration() {
        let mut session_manager = EditorSessionManager::new();

        session_manager
            .create_session("session1".to_string())
            .unwrap();
        session_manager
            .create_session("session2".to_string())
            .unwrap();

        session_manager
            .with_document_mut("session1", |doc| {
                doc.replace(
                    Range::new(Position::new(0), Position::new(0)),
                    "[Script Info]\nTitle: Session 1",
                )
            })
            .unwrap();

        session_manager
            .with_document_mut("session2", |doc| {
                doc.replace(
                    Range::new(Position::new(0), Position::new(0)),
                    "[Script Info]\nTitle: Session 2",
                )
            })
            .unwrap();

        session_manager
            .with_document("session1", |doc| {
                assert!(doc.text().contains("Session 1"));
                assert!(!doc.text().contains("Session 2"));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_thread_safe_extension_manager() {
        let manager = ExtensionManager::new();
        let manager1 = manager.clone();
        let manager2 = manager.clone();

        let handle1 = thread::spawn(move || {
            let mut manager_mut = manager1;
            manager_mut.set_config("thread1_key".to_string(), "thread1_value".to_string());
        });

        let handle2 = thread::spawn(move || {
            let mut manager_mut = manager2;
            manager_mut.set_config("thread2_key".to_string(), "thread2_value".to_string());
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // The unified manager is already thread-safe internally
        assert_eq!(
            manager.get_config("thread1_key"),
            Some("thread1_value".to_string())
        );
        assert_eq!(
            manager.get_config("thread2_key"),
            Some("thread2_value".to_string())
        );
    }

    #[test]
    fn test_thread_safe_session_manager() {
        let manager = EditorSessionManager::new();
        let manager1 = manager.clone();
        let manager2 = manager.clone();

        let handle1 = thread::spawn(move || {
            let mut manager_mut = manager1;
            manager_mut
                .create_session("thread1_session".to_string())
                .unwrap();
        });

        let handle2 = thread::spawn(move || {
            let mut manager_mut = manager2;
            manager_mut
                .create_session("thread2_session".to_string())
                .unwrap();
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // The unified manager is already thread-safe internally
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.contains(&"thread1_session".to_string()));
        assert!(sessions.contains(&"thread2_session".to_string()));
    }

    #[test]
    fn test_document_events_integration() {
        let (tx, rx) = mpsc::channel();
        let mut doc = EditorDocument::with_event_channel(tx);

        doc.insert(Position::new(0), "Hello").unwrap();
        doc.delete(Range::new(Position::new(0), Position::new(5)))
            .unwrap();
        doc.replace(Range::new(Position::new(0), Position::new(0)), "World")
            .unwrap();

        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], DocumentEvent::TextInserted { .. }));
        assert!(matches!(events[1], DocumentEvent::TextDeleted { .. }));
        assert!(matches!(events[2], DocumentEvent::TextReplaced { .. }));
    }

    #[test]
    fn test_extension_loading_and_state() {
        let mut manager = ExtensionManager::new();

        manager
            .load_extension(Box::new(
                ass_editor::extensions::builtin::syntax_highlight::SyntaxHighlightExtension::new(),
            ))
            .unwrap();

        manager
            .load_extension(Box::new(
                ass_editor::extensions::builtin::auto_complete::AutoCompleteExtension::new(),
            ))
            .unwrap();

        let extensions = manager.list_extensions();
        assert!(extensions.contains(&"syntax-highlight".to_string()));
        assert!(extensions.contains(&"auto-complete".to_string()));

        assert_eq!(
            manager.get_extension_state("syntax-highlight"),
            Some(ExtensionState::Uninitialized)
        );
    }
}

// ============================================================================
// Integration tests for new Event, Script Info, Fonts, and Graphics APIs
// ============================================================================

#[test]
fn test_event_deletion_integration() {
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker1,0,0,0,,First dialogue
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker2,0,0,0,,Comment line
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker3,0,0,0,,Second dialogue
Comment: 0,0:00:15.00,0:00:20.00,Default,Speaker4,0,0,0,,Another comment
Dialogue: 0,0:00:20.00,0:00:25.00,Default,Speaker5,0,0,0,,Third dialogue
"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Test single deletion
    let initial_count = doc.events().count().unwrap();
    assert_eq!(initial_count, 5);

    doc.events().delete(1).unwrap(); // Delete first comment
    assert_eq!(doc.events().count().unwrap(), 4);
    assert!(!doc.text().contains("Comment line"));

    // Test batch deletion
    doc.events().delete_multiple(vec![0, 2]).unwrap(); // Delete first and third remaining events
    assert_eq!(doc.events().count().unwrap(), 2);

    // Test query-based deletion (delete all comments)
    let mut doc2 = EditorDocument::from_content(content).unwrap();
    doc2.events().delete_query().comments().unwrap();
    assert_eq!(doc2.events().count().unwrap(), 3);
    assert_eq!(doc2.events().dialogues().execute().unwrap().len(), 3);
    assert_eq!(doc2.events().comments().execute().unwrap().len(), 0);

    // Test undo/redo with event deletion
    let mut doc3 = EditorDocument::from_content(content).unwrap();
    doc3.events().delete(0).unwrap();
    assert_eq!(doc3.events().count().unwrap(), 4);
    doc3.undo().unwrap();
    assert_eq!(doc3.events().count().unwrap(), 5);
    doc3.redo().unwrap();
    assert_eq!(doc3.events().count().unwrap(), 4);
}

#[test]
fn test_script_info_crud_integration() {
    let content = r#"[Script Info]
Title: Original Title
ScriptType: v4.00+

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Test reading existing properties
    assert_eq!(
        doc.info().get_title().unwrap(),
        Some("Original Title".to_string())
    );
    assert_eq!(
        doc.info().get("ScriptType").unwrap(),
        Some("v4.00+".to_string())
    );

    // Test updating existing property
    doc.info().title("New Title").unwrap();
    assert_eq!(
        doc.info().get_title().unwrap(),
        Some("New Title".to_string())
    );
    assert!(doc.text().contains("Title: New Title"));

    // Test adding new properties
    doc.info().author("John Doe").unwrap();
    doc.info().resolution(1920, 1080).unwrap();
    doc.info().wrap_style(2).unwrap();
    doc.info().scaled_border_and_shadow(true).unwrap();

    // Verify all properties
    let all_props = doc.info().all().unwrap();
    assert!(all_props.contains(&("Title".to_string(), "New Title".to_string())));
    assert!(all_props.contains(&("Original Script".to_string(), "John Doe".to_string())));
    assert!(all_props.contains(&("PlayResX".to_string(), "1920".to_string())));
    assert!(all_props.contains(&("PlayResY".to_string(), "1080".to_string())));
    assert!(all_props.contains(&("WrapStyle".to_string(), "2".to_string())));
    assert!(all_props.contains(&("ScaledBorderAndShadow".to_string(), "yes".to_string())));

    // Test deleting property
    doc.info().delete("ScriptType").unwrap();
    assert_eq!(doc.info().get("ScriptType").unwrap(), None);
    assert!(!doc.text().contains("ScriptType"));

    // Test undo/redo with script info
    doc.undo().unwrap(); // Undo delete
    assert_eq!(
        doc.info().get("ScriptType").unwrap(),
        Some("v4.00+".to_string())
    );
}

#[test]
fn test_fonts_management_integration() {
    let mut doc = EditorDocument::new();

    // Start with empty document
    assert_eq!(doc.fonts().count().unwrap(), 0);
    assert!(doc.fonts().list().unwrap().is_empty());

    // Add fonts using UU-encoded data
    let font_data1 = vec![
        "begin 644 arial.ttf".to_string(),
        "M1234567890ABCDEF".to_string(),
        "end".to_string(),
    ];

    let font_data2 = vec![
        "begin 644 times.ttf".to_string(),
        "MFEDCBA0987654321".to_string(),
        "end".to_string(),
    ];

    doc.fonts().add("arial.ttf", font_data1.clone()).unwrap();
    doc.fonts().add("times.ttf", font_data2).unwrap();

    // Verify fonts were added
    assert_eq!(doc.fonts().count().unwrap(), 2);
    let fonts_list = doc.fonts().list().unwrap();
    assert_eq!(fonts_list.len(), 2);
    assert!(fonts_list.contains(&"arial.ttf".to_string()));
    assert!(fonts_list.contains(&"times.ttf".to_string()));

    // Test exists check
    assert!(doc.fonts().exists("arial.ttf").unwrap());
    assert!(doc.fonts().exists("times.ttf").unwrap());
    assert!(!doc.fonts().exists("comic.ttf").unwrap());

    // Add font from binary data
    let binary_data = b"This is fake font binary data for testing!";
    doc.fonts().add_binary("custom.otf", binary_data).unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 3);
    assert!(doc.fonts().exists("custom.otf").unwrap());

    // Verify the document structure
    assert!(doc.text().contains("[Fonts]"));
    assert!(doc.text().contains("fontname: arial.ttf"));
    assert!(doc.text().contains("fontname: times.ttf"));
    assert!(doc.text().contains("fontname: custom.otf"));
    assert!(doc.text().contains("begin 644"));
    assert!(doc.text().contains("end"));

    // Remove a specific font
    doc.fonts().remove("times.ttf").unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 2);
    assert!(!doc.fonts().exists("times.ttf").unwrap());
    assert!(doc.fonts().exists("arial.ttf").unwrap());
    assert!(doc.fonts().exists("custom.otf").unwrap());

    // Clear all fonts
    doc.fonts().clear().unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 0);
    assert!(doc.fonts().list().unwrap().is_empty());

    // Section should still exist but be empty
    assert!(doc.text().contains("[Fonts]"));
}

#[test]
fn test_graphics_management_integration() {
    let mut doc = EditorDocument::new();

    // Start with empty document
    assert_eq!(doc.graphics().count().unwrap(), 0);
    assert!(doc.graphics().list().unwrap().is_empty());

    // Add graphics using UU-encoded data
    let graphic_data1 = vec![
        "begin 644 logo.png".to_string(),
        "M89PNG0D0A1A0A0000".to_string(),
        "end".to_string(),
    ];

    let graphic_data2 = vec![
        "begin 644 banner.jpg".to_string(),
        "MFFD8FFE000104A464946".to_string(),
        "end".to_string(),
    ];

    doc.graphics().add("logo.png", graphic_data1).unwrap();
    doc.graphics().add("banner.jpg", graphic_data2).unwrap();

    // Verify graphics were added
    assert_eq!(doc.graphics().count().unwrap(), 2);
    let graphics_list = doc.graphics().list().unwrap();
    assert_eq!(graphics_list.len(), 2);
    assert!(graphics_list.contains(&"logo.png".to_string()));
    assert!(graphics_list.contains(&"banner.jpg".to_string()));

    // Test exists check
    assert!(doc.graphics().exists("logo.png").unwrap());
    assert!(doc.graphics().exists("banner.jpg").unwrap());
    assert!(!doc.graphics().exists("icon.gif").unwrap());

    // Add graphic from binary data
    let binary_data = b"GIF89a fake image data for testing!";
    doc.graphics().add_binary("icon.gif", binary_data).unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 3);
    assert!(doc.graphics().exists("icon.gif").unwrap());

    // Verify the document structure
    assert!(doc.text().contains("[Graphics]"));
    assert!(doc.text().contains("filename: logo.png"));
    assert!(doc.text().contains("filename: banner.jpg"));
    assert!(doc.text().contains("filename: icon.gif"));

    // Remove a specific graphic
    doc.graphics().remove("banner.jpg").unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 2);
    assert!(!doc.graphics().exists("banner.jpg").unwrap());

    // Clear all graphics
    doc.graphics().clear().unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 0);
    assert!(doc.graphics().list().unwrap().is_empty());
}

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
