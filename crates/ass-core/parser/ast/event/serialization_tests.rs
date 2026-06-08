//! Tests for [`Event`] ASS serialization and span validation.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(all(not(feature = "std"), debug_assertions))]
use alloc::vec::Vec;

#[cfg(debug_assertions)]
#[test]
fn event_validate_spans() {
    let source = "Dialogue,0,0:00:05.00,0:00:10.00,Default,Character,0,0,0,,Hello world";
    let source_start = source.as_ptr() as usize;
    let source_end = source_start + source.len();
    let source_range = source_start..source_end;

    let fields: Vec<&str> = source.split(',').collect();
    let event = Event {
        event_type: EventType::Dialogue,
        layer: fields[1],
        start: fields[2],
        end: fields[3],
        style: fields[4],
        name: fields[5],
        margin_l: fields[6],
        margin_r: fields[7],
        margin_v: fields[8],
        margin_t: None,
        margin_b: None,
        effect: fields[9],
        text: fields[10],
        span: Span::new(0, 0, 0, 0),
    };

    assert!(event.validate_spans(&source_range));
    assert_eq!(event.layer, "0");
    assert_eq!(event.start, "0:00:05.00");
    assert_eq!(event.end, "0:00:10.00");
    assert_eq!(event.style, "Default");
    assert_eq!(event.name, "Character");
    assert_eq!(event.text, "Hello world");
}

#[cfg(debug_assertions)]
#[test]
fn event_validate_spans_invalid() {
    let source1 = "Dialogue,0,0:00:05.00,0:00:10.00,Default";
    let source2 = "Other,Character,Hello";
    let source1_start = source1.as_ptr() as usize;
    let source1_end = source1_start + source1.len();
    let source1_range = source1_start..source1_end;

    let event = Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start: "0:00:05.00",
        end: "0:00:10.00",
        style: "Default",
        name: &source2[6..15],  // "Character" from different source
        text: &source2[16..21], // "Hello" from different source
        ..Event::default()
    };

    // Should fail since some fields are from different source
    assert!(!event.validate_spans(&source1_range));
}

#[test]
fn event_to_ass_string() {
    let event = Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start: "0:00:05.00",
        end: "0:00:10.00",
        style: "Default",
        name: "Speaker",
        margin_l: "10",
        margin_r: "20",
        margin_v: "15",
        effect: "fade",
        text: "Hello world",
        ..Event::default()
    };

    let ass_string = event.to_ass_string();
    assert_eq!(
        ass_string,
        "Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,10,20,15,fade,Hello world"
    );
}

#[test]
fn event_to_ass_string_with_format() {
    let event = Event {
        event_type: EventType::Comment,
        start: "0:00:00.00",
        end: "0:00:05.00",
        text: "Test comment",
        ..Event::default()
    };

    // V4+ format
    let v4_format = vec![
        "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect", "Text",
    ];
    let v4_string = event.to_ass_string_with_format(&v4_format);
    assert_eq!(
        v4_string,
        "Comment: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test comment"
    );

    // Custom minimal format
    let min_format = vec!["Start", "End", "Text"];
    let min_string = event.to_ass_string_with_format(&min_format);
    assert_eq!(min_string, "Comment: 0:00:00.00,0:00:05.00,Test comment");

    // V4++ format with margin_t/margin_b
    let event_v4pp = Event {
        event_type: EventType::Dialogue,
        margin_t: Some("5"),
        margin_b: Some("10"),
        text: "V4++ test",
        ..Event::default()
    };
    let v4pp_format = vec![
        "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginT", "MarginB",
        "Effect", "Text",
    ];
    let v4pp_string = event_v4pp.to_ass_string_with_format(&v4pp_format);
    assert_eq!(
        v4pp_string,
        "Dialogue: 0,0:00:00.00,0:00:00.00,Default,,0,0,5,10,,V4++ test"
    );
}
