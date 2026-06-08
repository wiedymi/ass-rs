//! Unit tests for [`StreamingContext`] and its interaction with [`ParserState`].

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

#[test]
fn streaming_context_operations() {
    let mut context = StreamingContext::new();
    assert_eq!(context.line_number, 0);
    assert_eq!(context.current_section, None);

    context.next_line();
    assert_eq!(context.line_number, 1);

    context.enter_section(SectionKind::Events);
    assert_eq!(context.current_section, Some(SectionKind::Events));

    context.set_events_format("Layer,Start,End,Text".to_string());
    assert!(context.events_format.is_some());

    context.reset();
    assert_eq!(context.line_number, 0);
    assert_eq!(context.current_section, None);
    assert!(context.events_format.is_none());
}

#[test]
fn streaming_context_default() {
    let context = StreamingContext::default();
    assert_eq!(context.line_number, 0);
    assert_eq!(context.current_section, None);
    assert!(context.events_format.is_none());
    assert!(context.styles_format.is_none());
}

#[test]
fn streaming_context_debug_and_clone() {
    let context = StreamingContext::new();
    let debug_str = format!("{context:?}");
    assert!(debug_str.contains("StreamingContext"));
    assert!(debug_str.contains("line_number"));

    let mut context_with_data = StreamingContext::new();
    context_with_data.next_line();
    context_with_data.enter_section(SectionKind::Events);
    context_with_data.set_events_format("Test Format".to_string());

    let cloned = context_with_data.clone();
    assert_eq!(cloned.line_number, context_with_data.line_number);
    assert_eq!(cloned.current_section, context_with_data.current_section);
    assert_eq!(cloned.events_format, context_with_data.events_format);
}

#[test]
fn streaming_context_format_management() {
    let mut context = StreamingContext::new();

    // Test events format
    assert!(context.events_format.is_none());
    context.set_events_format("Layer, Start, End, Style, Text".to_string());
    assert!(context.events_format.is_some());
    assert_eq!(
        context.events_format.as_ref().unwrap(),
        "Layer, Start, End, Style, Text"
    );

    // Test styles format
    assert!(context.styles_format.is_none());
    context.set_styles_format("Name, Fontname, Fontsize".to_string());
    assert!(context.styles_format.is_some());
    assert_eq!(
        context.styles_format.as_ref().unwrap(),
        "Name, Fontname, Fontsize"
    );

    // Test reset clears formats
    context.reset();
    assert!(context.events_format.is_none());
    assert!(context.styles_format.is_none());
}

#[test]
fn streaming_context_section_management() {
    let mut context = StreamingContext::new();
    assert_eq!(context.current_section, None);

    context.enter_section(SectionKind::ScriptInfo);
    assert_eq!(context.current_section, Some(SectionKind::ScriptInfo));

    context.enter_section(SectionKind::Events);
    assert_eq!(context.current_section, Some(SectionKind::Events));

    context.exit_section();
    assert_eq!(context.current_section, None);
}

#[test]
fn streaming_context_line_tracking() {
    let mut context = StreamingContext::new();
    assert_eq!(context.line_number, 0);

    for expected_line in 1..=100 {
        context.next_line();
        assert_eq!(context.line_number, expected_line);
    }

    context.reset();
    assert_eq!(context.line_number, 0);
}

#[test]
fn complex_state_context_interaction() {
    let mut state = ParserState::ExpectingSection;
    let mut context = StreamingContext::new();

    // Simulate processing script
    context.next_line(); // Line 1
    state.enter_section(SectionKind::ScriptInfo);
    context.enter_section(SectionKind::ScriptInfo);

    context.next_line(); // Line 2
    context.next_line(); // Line 3

    state.enter_section(SectionKind::Events);
    context.enter_section(SectionKind::Events);
    context.set_events_format("Layer, Start, End, Text".to_string());

    context.next_line(); // Line 4
    state.enter_event(SectionKind::Events);

    assert_eq!(context.line_number, 4);
    assert!(context.events_format.is_some());
    assert_eq!(context.current_section, Some(SectionKind::Events));
    assert_eq!(state.current_section(), Some(SectionKind::Events));
}
