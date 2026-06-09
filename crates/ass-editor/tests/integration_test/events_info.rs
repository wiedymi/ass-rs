//! Integration tests for the Event and Script Info editing APIs.

use ass_editor::EditorDocument;

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
