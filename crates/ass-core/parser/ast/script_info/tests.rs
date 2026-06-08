//! Unit tests for the [`ScriptInfo`] AST node

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

#[test]
fn script_info_field_access() {
    let fields = vec![("Title", "Test Script"), ("ScriptType", "v4.00+")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };

    assert_eq!(info.title(), "Test Script");
    assert_eq!(info.script_type(), Some("v4.00+"));
    assert_eq!(info.get_field("Unknown"), None);
}

#[test]
fn script_info_defaults() {
    let info = ScriptInfo {
        fields: Vec::new(),
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.title(), "<untitled>");
    assert_eq!(info.wrap_style(), 0);
    assert_eq!(info.layout_resolution(), None);
    assert_eq!(info.play_resolution(), None);
}

#[test]
fn script_info_play_resolution() {
    let fields = vec![("PlayResX", "1920"), ("PlayResY", "1080")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.play_resolution(), Some((1920, 1080)));
}

#[test]
fn script_info_partial_play_resolution() {
    let fields = vec![("PlayResX", "1920")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.play_resolution(), None);
}

#[test]
fn script_info_layout_resolution() {
    let fields = vec![("LayoutResX", "1920"), ("LayoutResY", "1080")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.layout_resolution(), Some((1920, 1080)));
}

#[test]
fn script_info_partial_layout_resolution() {
    let fields = vec![("LayoutResX", "1920")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.layout_resolution(), None);
}

#[test]
fn script_info_wrap_style() {
    let fields = vec![("WrapStyle", "2")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.wrap_style(), 2);
}

#[test]
fn script_info_invalid_wrap_style() {
    let fields = vec![("WrapStyle", "invalid")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.wrap_style(), 0); // Default fallback
}

#[test]
fn script_info_invalid_resolution() {
    let fields = vec![("PlayResX", "invalid"), ("PlayResY", "1080")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.play_resolution(), None);
}

#[test]
fn script_info_case_sensitive_keys() {
    let fields = vec![("title", "Test"), ("Title", "Correct")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    assert_eq!(info.get_field("Title"), Some("Correct"));
    assert_eq!(info.get_field("title"), Some("Test"));
}

#[test]
fn script_info_to_ass_string() {
    let fields = vec![
        ("Title", "Test Script"),
        ("ScriptType", "v4.00+"),
        ("WrapStyle", "0"),
        ("ScaledBorderAndShadow", "yes"),
        ("YCbCr Matrix", "None"),
    ];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };

    let ass_string = info.to_ass_string();
    assert!(ass_string.starts_with("[Script Info]\n"));
    assert!(ass_string.contains("Title: Test Script\n"));
    assert!(ass_string.contains("ScriptType: v4.00+\n"));
    assert!(ass_string.contains("WrapStyle: 0\n"));
    assert!(ass_string.contains("ScaledBorderAndShadow: yes\n"));
    assert!(ass_string.contains("YCbCr Matrix: None\n"));
}

#[test]
fn script_info_to_ass_string_empty() {
    let info = ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    };

    let ass_string = info.to_ass_string();
    assert_eq!(ass_string, "[Script Info]\n");
}
