//! Tests for batched style and event insertion.

use super::*;
use crate::parser::ast::{Event, Section, SectionType, Style};
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn batch_add_styles() {
    use crate::parser::ast::Span;

    let content = "[Script Info]\nTitle: Test";
    let mut script = Script::parse(content).unwrap();

    // Create batch of styles
    let styles = vec![
        Style {
            name: "Style1",
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
        },
        Style {
            name: "Style2",
            parent: None,
            fontname: "Times",
            fontsize: "18",
            primary_colour: "&H00FFFFFF",
            secondary_colour: "&H000000FF",
            outline_colour: "&H00000000",
            back_colour: "&H00000000",
            bold: "1",
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
        },
    ];

    let batch = StyleBatch { styles };
    let indices = script.batch_add_styles(batch);

    // Verify indices
    assert_eq!(indices, vec![0, 1]);

    // Verify styles were added
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        assert_eq!(styles.len(), 2);
        assert_eq!(styles[0].name, "Style1");
        assert_eq!(styles[1].name, "Style2");
    }
}

#[test]
fn batch_add_events() {
    use crate::parser::ast::{EventType, Span};

    let content = "[Script Info]\nTitle: Test\n\n[Events]\nFormat: Layer, Start, End, Style, Text";
    let mut script = Script::parse(content).unwrap();

    // Create batch of events
    let events = vec![
        Event {
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
            text: "Event 1",
            span: Span::new(0, 0, 0, 0),
        },
        Event {
            event_type: EventType::Comment,
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
            text: "Comment 1",
            span: Span::new(0, 0, 0, 0),
        },
    ];

    let batch = EventBatch { events };
    let indices = script.batch_add_events(batch);

    // Verify indices
    assert_eq!(indices, vec![0, 1]);

    // Verify events were added
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].text, "Event 1");
        assert_eq!(events[1].text, "Comment 1");
    }
}
