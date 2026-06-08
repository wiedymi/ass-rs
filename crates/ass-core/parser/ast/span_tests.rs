//! Tests for [`Span`] and AST node integration

#[cfg(not(feature = "std"))]
extern crate alloc;

use super::*;
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn test_span_creation() {
    let span = Span::new(0, 10, 1, 1);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 10);
    assert_eq!(span.line, 1);
    assert_eq!(span.column, 1);
}

#[test]
fn test_span_contains() {
    let span = Span::new(0, 10, 1, 1);
    assert!(span.contains(0));
    assert!(span.contains(5));
    assert!(span.contains(9));
    assert!(!span.contains(10));
    assert!(!span.contains(15));
}

#[test]
fn test_span_merge() {
    let span1 = Span::new(0, 10, 1, 1);
    let span2 = Span::new(5, 15, 1, 6);
    let merged = span1.merge(&span2);

    assert_eq!(merged.start, 0);
    assert_eq!(merged.end, 15);
    assert_eq!(merged.line, 1);
    assert_eq!(merged.column, 1);

    // Test merge with different lines
    let span3 = Span::new(20, 30, 2, 5);
    let span4 = Span::new(25, 35, 3, 10);
    let merged2 = span3.merge(&span4);

    assert_eq!(merged2.start, 20);
    assert_eq!(merged2.end, 35);
    assert_eq!(merged2.line, 2);
    assert_eq!(merged2.column, 5);
}

#[test]
fn ast_integration_script_info() {
    let fields = vec![("Title", "Integration Test"), ("ScriptType", "v4.00+")];
    let info = ScriptInfo {
        fields,
        span: Span::new(0, 0, 0, 0),
    };
    let section = Section::ScriptInfo(info);

    assert_eq!(section.section_type(), SectionType::ScriptInfo);
}

#[test]
fn ast_integration_events() {
    let event = Event {
        event_type: EventType::Dialogue,
        start: "0:00:05.00",
        end: "0:00:10.00",
        style: "Default",
        text: "Test dialogue",
        ..Event::default()
    };

    let events = vec![event];
    let section = Section::Events(events);

    assert_eq!(section.section_type(), SectionType::Events);
}

#[test]
fn ast_integration_styles() {
    let style = Style {
        name: "TestStyle",
        fontname: "Arial",
        fontsize: "20",
        ..Style::default()
    };

    let styles = vec![style];
    let section = Section::Styles(styles);

    assert_eq!(section.section_type(), SectionType::Styles);
}

#[test]
fn ast_integration_fonts() {
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["encoded data line 1", "encoded data line 2"],
        span: Span::new(0, 0, 0, 0),
    };

    let fonts = vec![font];
    let section = Section::Fonts(fonts);

    assert_eq!(section.section_type(), SectionType::Fonts);
}

#[test]
fn ast_integration_graphics() {
    let graphic = Graphic {
        filename: "logo.png",
        data_lines: vec!["encoded image data"],
        span: Span::new(0, 0, 0, 0),
    };

    let graphics = vec![graphic];
    let section = Section::Graphics(graphics);

    assert_eq!(section.section_type(), SectionType::Graphics);
}

#[test]
fn event_type_round_trip() {
    let types = [
        EventType::Dialogue,
        EventType::Comment,
        EventType::Picture,
        EventType::Sound,
        EventType::Movie,
        EventType::Command,
    ];

    for event_type in types {
        let str_repr = event_type.as_str();
        let parsed = EventType::parse_type(str_repr);
        assert_eq!(parsed, Some(event_type));
    }
}

#[test]
fn section_type_properties() {
    assert!(SectionType::ScriptInfo.is_required());
    assert!(SectionType::Events.is_required());
    assert!(!SectionType::Styles.is_required());

    assert!(SectionType::Events.is_timed());
    assert!(!SectionType::ScriptInfo.is_timed());

    assert_eq!(SectionType::ScriptInfo.header_name(), "Script Info");
    assert_eq!(SectionType::Events.header_name(), "Events");
}
