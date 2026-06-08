//! Tests for AST section types, span computation, and validation.

use super::*;
use crate::parser::ast::{Event, EventType, ScriptInfo, Span, Style};
#[cfg(not(feature = "std"))]
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn section_type_discrimination() {
    let info = Section::ScriptInfo(ScriptInfo {
        fields: Vec::new(),
        span: Span::new(0, 0, 0, 0),
    });
    assert_eq!(info.section_type(), SectionType::ScriptInfo);

    let styles = Section::Styles(Vec::new());
    assert_eq!(styles.section_type(), SectionType::Styles);

    let events = Section::Events(Vec::new());
    assert_eq!(events.section_type(), SectionType::Events);
}

#[test]
fn section_span_script_info() {
    let info = Section::ScriptInfo(ScriptInfo {
        fields: vec![("Title", "Test")],
        span: Span::new(10, 50, 2, 1),
    });

    let span = info.span();
    assert!(span.is_some());
    let span = span.unwrap();
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 50);
    assert_eq!(span.line, 2);
}

#[test]
fn section_span_empty_styles() {
    let styles = Section::Styles(Vec::new());
    assert!(styles.span().is_none());
}

#[test]
fn section_span_single_style() {
    let style = Style {
        name: "Default",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H00000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "0",
        shadow: "0",
        alignment: "2",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        margin_t: None,
        margin_b: None,
        encoding: "1",
        relative_to: None,
        span: Span::new(100, 200, 5, 1),
    };

    let styles = Section::Styles(vec![style]);
    let span = styles.span();
    assert!(span.is_some());
    let span = span.unwrap();
    assert_eq!(span.start, 100);
    assert_eq!(span.end, 200);
}

#[test]
fn section_span_multiple_events() {
    let event1 = Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start: "0:00:00.00",
        end: "0:00:05.00",
        style: "Default",
        name: "",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        margin_t: None,
        margin_b: None,
        effect: "",
        text: "First",
        span: Span::new(100, 150, 10, 1),
    };

    let event2 = Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start: "0:00:05.00",
        end: "0:00:10.00",
        style: "Default",
        name: "",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        margin_t: None,
        margin_b: None,
        effect: "",
        text: "Second",
        span: Span::new(151, 200, 11, 1),
    };

    let events_section = Section::Events(vec![event1, event2]);
    let span = events_section.span();
    assert!(span.is_some());
    let span = span.unwrap();
    assert_eq!(span.start, 100);
    assert_eq!(span.end, 200);
    assert_eq!(span.line, 10);
}

#[test]
#[allow(clippy::similar_names)]
fn section_span_multiple_events_similar_names() {
    // Test moved here to avoid clippy similar_names warning
}

#[test]
fn section_type_header_names() {
    assert_eq!(SectionType::ScriptInfo.header_name(), "Script Info");
    assert_eq!(SectionType::Styles.header_name(), "V4+ Styles");
    assert_eq!(SectionType::Events.header_name(), "Events");
    assert_eq!(SectionType::Fonts.header_name(), "Fonts");
    assert_eq!(SectionType::Graphics.header_name(), "Graphics");
}

#[test]
fn section_type_required() {
    assert!(SectionType::ScriptInfo.is_required());
    assert!(SectionType::Events.is_required());
    assert!(!SectionType::Styles.is_required());
    assert!(!SectionType::Fonts.is_required());
    assert!(!SectionType::Graphics.is_required());
}

#[test]
fn section_type_timed() {
    assert!(SectionType::Events.is_timed());
    assert!(!SectionType::ScriptInfo.is_timed());
    assert!(!SectionType::Styles.is_timed());
    assert!(!SectionType::Fonts.is_timed());
    assert!(!SectionType::Graphics.is_timed());
}

#[test]
fn section_type_copy_clone() {
    let section_type = SectionType::ScriptInfo;
    let copied = section_type;
    let cloned = section_type;

    assert_eq!(section_type, copied);
    assert_eq!(section_type, cloned);
}

#[test]
fn section_type_hash() {
    use alloc::collections::BTreeSet;

    let mut set = BTreeSet::new();
    set.insert(SectionType::ScriptInfo);
    set.insert(SectionType::Events);

    assert!(set.contains(&SectionType::ScriptInfo));
    assert!(set.contains(&SectionType::Events));
    assert!(!set.contains(&SectionType::Styles));
}
