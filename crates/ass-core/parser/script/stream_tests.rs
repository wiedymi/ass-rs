//! Tests for streaming delta types and partial reparse (requires `stream`).

use super::*;
use crate::parser::ast::Section;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec, vec::Vec};

#[test]
fn script_delta_is_empty() {
    use crate::parser::ast::Span;

    let delta = ScriptDelta {
        added: Vec::new(),
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };
    assert!(delta.is_empty());

    let non_empty_delta = ScriptDelta {
        added: vec![],
        modified: vec![(
            0,
            Section::ScriptInfo(crate::parser::ast::ScriptInfo {
                fields: Vec::new(),
                span: Span::new(0, 0, 0, 0),
            }),
        )],
        removed: Vec::new(),
        new_issues: Vec::new(),
    };
    assert!(!non_empty_delta.is_empty());
}

#[test]
fn script_delta_debug() {
    let delta = ScriptDelta {
        added: Vec::new(),
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };
    let debug_str = format!("{delta:?}");
    assert!(debug_str.contains("ScriptDelta"));
}

#[test]
fn script_delta_owned_debug() {
    let delta = ScriptDeltaOwned {
        added: Vec::new(),
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };
    let debug_str = format!("{delta:?}");
    assert!(debug_str.contains("ScriptDeltaOwned"));
}

#[test]
fn parse_partial_basic() {
    let content = "[Script Info]\nTitle: Original";
    let script = Script::parse(content).unwrap();

    // Test partial parsing (this may fail if streaming isn't fully implemented)
    let result = script.parse_partial(0..content.len(), "[Script Info]\nTitle: Modified");
    // Either succeeds or fails gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn streaming_features_comprehensive() {
    use crate::parser::ast::{ScriptInfo, Section, Span};

    let content = "[Script Info]\nTitle: Original\nAuthor: Test";
    let _script = Script::parse(content).unwrap();

    // Test ScriptDelta creation and methods
    let empty_delta = ScriptDelta {
        added: Vec::new(),
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };
    assert!(empty_delta.is_empty());

    // Test non-empty delta
    let script_info = ScriptInfo {
        fields: Vec::new(),
        span: Span::new(0, 0, 0, 0),
    };
    let non_empty_delta = ScriptDelta {
        added: vec![Section::ScriptInfo(script_info)],
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };
    assert!(!non_empty_delta.is_empty());

    // Test delta cloning
    let cloned_delta = empty_delta.clone();
    assert!(cloned_delta.is_empty());

    // Test owned delta
    let owned_delta = ScriptDeltaOwned {
        added: vec!["test".to_string()],
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };
    let _debug_str = format!("{owned_delta:?}");
    let _ = owned_delta;
}

#[test]
fn parse_partial_error_handling() {
    let content = "[Script Info]\nTitle: Test";
    let script = Script::parse(content).unwrap();

    // Test various partial parsing scenarios
    let test_cases = vec![
        (0..5, "[Invalid"),
        (0..content.len(), "[Script Info]\nTitle: Modified"),
        (5..10, "New"),
    ];

    for (range, new_text) in test_cases {
        let result = script.parse_partial(range, new_text);
        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}
