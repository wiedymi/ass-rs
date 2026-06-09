//! Document-driven completion workflow tests for the auto-completion extension.

use crate::core::{EditorDocument, Position};
use crate::extensions::builtin::auto_complete::{
    AutoCompleteExtension, CompletionContext, CompletionItem, CompletionType,
};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn test_style_completions() {
    let mut ext = AutoCompleteExtension::new();
    let doc = EditorDocument::from_content(
        "[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1\nStyle: Title,Times New Roman,30,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,",
    )
    .unwrap();

    // Update style names
    ext.update_style_names(&doc).unwrap();

    let context = CompletionContext {
        line: "Dialogue: 0,0:00:00.00,0:00:05.00,".to_string(),
        column: 34,
        section: Some("Events".to_string()),
        in_override_tag: false,
        current_tag: None,
    };

    // Check if we should complete styles
    assert!(ext.should_complete_style(&context));

    let completions = ext.get_style_completions(&context);

    // Debug output if test fails
    if completions.is_empty() {
        // println!(
        //     "No style completions found. Style names: {:?}",
        //     ext.style_names
        // );
    }

    assert_eq!(completions.len(), 2);
    assert!(completions.iter().any(|c| c.insert_text == "Default"));
    assert!(completions.iter().any(|c| c.insert_text == "Title"));
}

#[test]
fn test_completion_context_parsing() {
    let ext = AutoCompleteExtension::new();
    let doc = EditorDocument::from_content(
        "[Script Info]\nTitle: Test\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1}text",
    )
    .unwrap();

    // Test context at different positions
    let context1 = ext.get_completion_context(&doc, Position::new(25)).unwrap(); // After Title:
    assert_eq!(context1.section, Some("Script Info".to_string()));
    assert!(!context1.in_override_tag);

    // In override tag
    let tag_pos = doc.text().find("{\\b").unwrap() + 2;
    let context2 = ext
        .get_completion_context(&doc, Position::new(tag_pos))
        .unwrap();
    assert!(context2.in_override_tag);
}

#[test]
fn test_get_completions_integration() {
    let mut ext = AutoCompleteExtension::new();
    let doc = EditorDocument::from_content("[Script Info]\nTi").unwrap();

    let completions = ext
        .get_completions(&doc, Position::new(doc.len_bytes()))
        .unwrap();

    // Should have Title completion
    assert!(!completions.is_empty());
    assert!(completions.iter().any(|c| c.label == "Title:"));
}

#[test]
fn test_completion_item_builder() {
    let item = CompletionItem::new(
        "\\pos(100,200)".to_string(),
        "\\pos".to_string(),
        CompletionType::Tag,
    )
    .with_description("Position override tag".to_string())
    .with_detail("Sets absolute position".to_string())
    .with_sort_order(1);

    assert_eq!(item.insert_text, "\\pos(100,200)");
    assert_eq!(item.label, "\\pos");
    assert_eq!(item.description, Some("Position override tag".to_string()));
    assert_eq!(item.detail, Some("Sets absolute position".to_string()));
    assert_eq!(item.sort_order, 1);
}

#[test]
fn test_max_suggestions_limit() {
    let mut ext = AutoCompleteExtension::new();
    ext.config.max_suggestions = 2;

    let doc = EditorDocument::from_content("[Script Info]\n").unwrap();
    let completions = ext
        .get_completions(&doc, Position::new(doc.len_bytes()))
        .unwrap();

    // Should respect max_suggestions
    assert_eq!(completions.len(), 2);
}
