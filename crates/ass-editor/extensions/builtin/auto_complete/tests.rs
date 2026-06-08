//! Core unit tests for the auto-completion extension.

use super::*;
use crate::extensions::{EditorExtension, ExtensionCapability};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn test_completion_item() {
    let item = CompletionItem::new(
        "\\pos(100,200)".to_string(),
        "\\pos".to_string(),
        CompletionType::Tag,
    )
    .with_description("Position tag".to_string())
    .with_sort_order(1);

    assert_eq!(item.insert_text, "\\pos(100,200)");
    assert_eq!(item.label, "\\pos");
    assert_eq!(item.sort_order, 1);
}

#[test]
fn test_auto_complete_extension_creation() {
    let ext = AutoCompleteExtension::new();
    assert_eq!(ext.info().name, "auto-complete");
    assert!(ext
        .info()
        .has_capability(&ExtensionCapability::CodeCompletion));
}

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
    assert!(!completions.is_empty());
    assert!(completions.iter().any(|c| c.label == "[Script Info]"));
}

#[test]
fn test_field_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "Ti".to_string(),
        column: 2,
        section: Some("Script Info".to_string()),
        in_override_tag: false,
        current_tag: None,
    };

    let completions = ext.get_field_completions("Script Info", &context);
    assert!(!completions.is_empty());
    assert!(completions.iter().any(|c| c.label == "Title:"));
}

#[test]
fn test_tag_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "{\\po".to_string(),
        column: 4,
        section: Some("Events".to_string()),
        in_override_tag: true,
        current_tag: Some("po".to_string()),
    };

    let completions = ext.get_tag_completions(&context);
    assert!(!completions.is_empty());
    assert!(completions.iter().any(|c| c.label == "\\pos"));
}
