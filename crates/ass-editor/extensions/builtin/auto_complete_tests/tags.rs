//! Override-tag completion tests for the auto-completion extension.

use crate::extensions::builtin::auto_complete::{
    AutoCompleteExtension, CompletionContext, CompletionType,
};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn test_override_tag_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "{\\b".to_string(),
        column: 3,
        section: Some("Events".to_string()),
        in_override_tag: true,
        current_tag: Some("b".to_string()),
    };

    let completions = ext.get_tag_completions(&context);

    // Should have bold tag
    let bold = completions.iter().find(|c| c.label == "\\b").unwrap();
    assert_eq!(bold.completion_type, CompletionType::Tag);
    assert_eq!(bold.insert_text, "\\b1");
}

#[test]
fn test_tag_completions_with_prefix() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "{\\po".to_string(),
        column: 4,
        section: Some("Events".to_string()),
        in_override_tag: true,
        current_tag: Some("po".to_string()),
    };

    let completions = ext.get_tag_completions(&context);

    // Should have pos tag
    let pos = completions.iter().find(|c| c.label == "\\pos").unwrap();
    assert_eq!(pos.insert_text, "\\pos(640,360)");
}

#[test]
fn test_color_tag_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "{\\".to_string(),
        column: 2,
        section: Some("Events".to_string()),
        in_override_tag: true,
        current_tag: None,
    };

    let completions = ext.get_tag_completions(&context);

    // Should have color tags
    assert!(completions.iter().any(|c| c.label == "\\c"));
    assert!(completions.iter().any(|c| c.label == "\\1c"));
    assert!(completions.iter().any(|c| c.label == "\\2c"));
    assert!(completions.iter().any(|c| c.label == "\\3c"));
    assert!(completions.iter().any(|c| c.label == "\\4c"));
}

#[test]
fn test_animation_tag_completions() {
    let ext = AutoCompleteExtension::new();
    let context = CompletionContext {
        line: "{\\t".to_string(),
        column: 3,
        section: Some("Events".to_string()),
        in_override_tag: true,
        current_tag: Some("t".to_string()),
    };

    let completions = ext.get_tag_completions(&context);

    // Should have animation tag
    let anim = completions.iter().find(|c| c.label == "\\t").unwrap();
    assert_eq!(anim.insert_text, "\\t(\\fs30)");
    assert_eq!(anim.completion_type, CompletionType::Tag);
}
