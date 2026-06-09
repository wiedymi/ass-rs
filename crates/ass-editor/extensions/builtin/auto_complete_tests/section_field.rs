//! Section and field completion tests for the auto-completion extension.

use crate::extensions::builtin::auto_complete::{
    AutoCompleteExtension, CompletionContext, CompletionType,
};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn test_section_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "[Scr".to_string(),
        column: 4,
        section: None,
        in_override_tag: false,
        current_tag: None,
    };

    let completions = ext.get_section_completions(&context);

    // Should have Script Info completion
    assert!(!completions.is_empty());
    let script_info = completions
        .iter()
        .find(|c| c.label == "[Script Info]")
        .unwrap();
    assert_eq!(script_info.completion_type, CompletionType::Section);
    assert!(script_info.description.is_some());
}

#[test]
fn test_empty_line_section_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "".to_string(),
        column: 0,
        section: None,
        in_override_tag: false,
        current_tag: None,
    };

    let completions = ext.get_section_completions(&context);

    // Should show all sections
    assert_eq!(completions.len(), 6);
    assert!(completions.iter().any(|c| c.label == "[Script Info]"));
    assert!(completions.iter().any(|c| c.label == "[V4+ Styles]"));
    assert!(completions.iter().any(|c| c.label == "[Events]"));
}

#[test]
fn test_script_info_field_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "Ti".to_string(),
        column: 2,
        section: Some("Script Info".to_string()),
        in_override_tag: false,
        current_tag: None,
    };

    let completions = ext.get_field_completions("Script Info", &context);

    // Should have Title completion
    let title = completions.iter().find(|c| c.label == "Title:").unwrap();
    assert_eq!(title.completion_type, CompletionType::Field);
}

#[test]
fn test_playres_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "PlayRes".to_string(),
        column: 7,
        section: Some("Script Info".to_string()),
        in_override_tag: false,
        current_tag: None,
    };

    let completions = ext.get_field_completions("Script Info", &context);

    // Should have both PlayResX and PlayResY
    assert!(completions.iter().any(|c| c.label == "PlayResX:"));
    assert!(completions.iter().any(|c| c.label == "PlayResY:"));
}

#[test]
fn test_events_field_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "".to_string(),
        column: 0,
        section: Some("Events".to_string()),
        in_override_tag: false,
        current_tag: None,
    };

    let completions = ext.get_field_completions("Events", &context);

    // Should have all event types
    assert!(completions.iter().any(|c| c.label == "Dialogue:"));
    assert!(completions.iter().any(|c| c.label == "Comment:"));
    assert!(completions.iter().any(|c| c.label == "Format:"));
}

#[test]
fn test_style_section_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "".to_string(),
        column: 0,
        section: Some("V4+ Styles".to_string()),
        in_override_tag: false,
        current_tag: None,
    };

    let completions = ext.get_field_completions("V4+ Styles", &context);

    // Should have Format and Style
    assert_eq!(completions.len(), 2);
    assert!(completions.iter().any(|c| c.label == "Format:"));
    assert!(completions.iter().any(|c| c.label == "Style:"));
}
