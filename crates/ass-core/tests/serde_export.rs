//! Serialization (JSON export) tests for the zero-copy AST.
//!
//! The AST borrows from the source text via `&'a str` spans, so the `serde`
//! feature provides [`serde::Serialize`] for *export* only. Borrowed
//! deserialization is intentionally not supported: a `&'a str` can only be
//! deserialized from an unescaped, contiguous JSON string, which never holds
//! for ASS dialogue text (it routinely contains `\` override tags). These
//! tests pin the export contract.
#![cfg(feature = "serde")]

use ass_core::parser::ast::{Event, EventType, Section, Span, Style};
use ass_core::parser::Script;

const SAMPLE: &str = "[Script Info]\n\
Title: Export Test\n\
ScriptType: v4.00+\n\
\n\
[V4+ Styles]\n\
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\
\n\
[Events]\n\
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
Dialogue: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,Hello {\\b1}world{\\b0}\n";

#[test]
fn script_serializes_to_json() {
    let script = Script::parse(SAMPLE).expect("parse sample script");

    let value: serde_json::Value =
        serde_json::to_value(&script).expect("serialize script to JSON value");

    assert_eq!(value["version"], "AssV4");

    let sections = value["sections"].as_array().expect("sections array");
    assert_eq!(sections.len(), script.sections().len());

    // Transient diagnostics and incremental state are skipped on export.
    assert!(value.get("issues").is_none());
    assert!(value.get("change_tracker").is_none());

    // Dialogue text (with `\` override tags) survives export verbatim.
    let events = sections
        .iter()
        .find_map(|s| s.get("Events"))
        .and_then(serde_json::Value::as_array)
        .expect("events section in serialized output");
    assert_eq!(events[0]["text"], "Hello {\\b1}world{\\b0}");
}

#[test]
fn event_serializes_all_fields() {
    let event = Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start: "0:00:00.00",
        end: "0:00:05.00",
        style: "Default",
        name: "Narrator",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        margin_t: Some("5"),
        margin_b: None,
        effect: "",
        text: "Hello {\\i1}world{\\i0}",
        span: Span::new(0, 21, 1, 1),
    };

    let value = serde_json::to_value(&event).expect("serialize event");

    assert_eq!(value["event_type"], "Dialogue");
    assert_eq!(value["style"], "Default");
    assert_eq!(value["name"], "Narrator");
    assert_eq!(value["margin_t"], "5");
    assert_eq!(value["margin_b"], serde_json::Value::Null);
    assert_eq!(value["text"], "Hello {\\i1}world{\\i0}");
    assert_eq!(value["span"]["start"], 0);
    assert_eq!(value["span"]["end"], 21);
}

#[test]
fn style_serializes_optional_fields() {
    let style = Style {
        name: "Default",
        fontname: "Arial",
        fontsize: "20",
        relative_to: Some("video"),
        ..Style::default()
    };

    let value = serde_json::to_value(&style).expect("serialize style");

    assert_eq!(value["name"], "Default");
    assert_eq!(value["fontname"], "Arial");
    assert_eq!(value["relative_to"], "video");
    assert_eq!(value["parent"], serde_json::Value::Null);
}

#[test]
fn section_enum_serializes_with_variant_tag() {
    let script = Script::parse(SAMPLE).expect("parse sample script");
    let events = script
        .sections()
        .iter()
        .find(|s| matches!(s, Section::Events(_)))
        .expect("events section present");

    let value = serde_json::to_value(events).expect("serialize section");

    // Externally tagged enum: the variant name keys the payload.
    let payload = value["Events"].as_array().expect("Events variant payload");
    assert_eq!(payload.len(), 1);
    assert_eq!(payload[0]["style"], "Default");
}
