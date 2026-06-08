//! Tests for adding/removing sections, styles, and events, plus batch updates.

use super::*;
use crate::parser::ast::{Event, Section, SectionType, Style};
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

#[test]
fn add_and_remove_sections() {
    let content = "[Script Info]\nTitle: Test";
    let mut script = Script::parse(content).unwrap();

    // Add a styles section
    let styles_section = Section::Styles(vec![]);
    let index = script.add_section(styles_section);
    assert_eq!(index, 1);
    assert_eq!(script.sections().len(), 2);

    // Remove the section
    let removed = script.remove_section(index);
    assert!(removed.is_ok());
    assert_eq!(script.sections().len(), 1);

    // Try to remove invalid index
    let invalid = script.remove_section(10);
    assert!(invalid.is_err());
}

#[test]
fn add_style_creates_section() {
    use crate::parser::ast::Span;

    let content = "[Script Info]\nTitle: Test";
    let mut script = Script::parse(content).unwrap();

    // Add a style when no styles section exists
    let style = Style {
        name: "NewStyle",
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
        span: Span::new(0, 0, 0, 0),
    };

    let index = script.add_style(style);
    assert_eq!(index, 0);

    // Verify section was created
    assert!(script.find_section(SectionType::Styles).is_some());
}

#[test]
fn add_event_to_existing_section() {
    use crate::parser::ast::{EventType, Span};

    let content = "[Script Info]\nTitle: Test\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";
    let mut script = Script::parse(content).unwrap();

    // Add an event to existing section
    let event = Event {
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
        text: "New Event",
        span: Span::new(0, 0, 0, 0),
    };

    let index = script.add_event(event);
    assert_eq!(index, 1);

    // Verify event was added
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].text, "New Event");
    }
}

#[test]
fn update_formats() {
    let content = "[Script Info]\nTitle: Test";
    let mut script = Script::parse(content).unwrap();

    // Set custom formats
    let styles_format = vec!["Name", "Fontname", "Bold"];
    script.set_styles_format(styles_format);

    let events_format = vec!["Start", "End", "Text"];
    script.set_events_format(events_format);

    // Verify formats were set
    assert!(script.styles_format().is_some());
    assert_eq!(script.styles_format().unwrap().len(), 3);
    assert_eq!(script.styles_format().unwrap()[2], "Bold");

    assert!(script.events_format().is_some());
    assert_eq!(script.events_format().unwrap().len(), 3);
    assert_eq!(script.events_format().unwrap()[0], "Start");
}

#[test]
fn batch_update_lines() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20\nStyle: Alt,Times,18\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello\nDialogue: 0,0:00:05.00,0:00:10.00,Default,World";
    let mut script = Script::parse(content).unwrap();

    // Get offsets for updates
    let mut operations = Vec::new();

    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        if styles.len() >= 2 {
            operations.push(UpdateOperation {
                offset: styles[0].span.start,
                new_line: "Style: Default,Helvetica,24",
                line_number: 10,
            });
            operations.push(UpdateOperation {
                offset: styles[1].span.start,
                new_line: "Style: Alt,Courier,16",
                line_number: 11,
            });
        }
    }

    let result = script.batch_update_lines(operations);

    // Check results
    assert_eq!(result.updated.len(), 2);
    assert_eq!(result.failed.len(), 0);

    // Verify updates were applied
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        assert_eq!(styles[0].fontname, "Helvetica");
        assert_eq!(styles[0].fontsize, "24");
        assert_eq!(styles[1].fontname, "Courier");
        assert_eq!(styles[1].fontsize, "16");
    }
}
