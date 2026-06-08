//! Per-section content processing and integration tests for [`LineProcessor`].

use super::super::state::SectionKind;
use super::*;

#[test]
fn format_line_processing() {
    let mut processor = LineProcessor::new();
    processor.state.enter_section(SectionKind::Events);
    processor.context.enter_section(SectionKind::Events);

    let result = processor
        .process_line("Format: Layer, Start, End, Style, Text")
        .unwrap();
    assert!(result.is_empty());
    assert!(processor.context.events_format.is_some());
}

#[test]
fn script_info_line_processing() {
    let mut processor = LineProcessor::new();
    processor.state.enter_section(SectionKind::ScriptInfo);

    let result = processor.process_line("Title: Test Script").unwrap();
    assert!(result.is_empty());

    let result = processor.process_line("Author: Test Author").unwrap();
    assert!(result.is_empty());

    let result = processor.process_line("ScriptType: v4.00+").unwrap();
    assert!(result.is_empty());

    // Malformed line without colon
    let result = processor.process_line("Malformed line").unwrap();
    assert!(result.is_empty());
}

#[test]
fn styles_line_processing() {
    let mut processor = LineProcessor::new();
    processor.state.enter_section(SectionKind::Styles);

    let result = processor
        .process_line("Format: Name, Fontname, Fontsize")
        .unwrap();
    assert!(result.is_empty());
    assert!(processor.context.styles_format.is_some());

    let result = processor.process_line("Style: Default,Arial,20").unwrap();
    assert!(result.is_empty());
}

#[test]
fn events_line_processing() {
    let mut processor = LineProcessor::new();
    processor.state.enter_section(SectionKind::Events);

    let result = processor
        .process_line("Format: Layer, Start, End, Style, Text")
        .unwrap();
    assert!(result.is_empty());
    assert!(processor.context.events_format.is_some());

    let result = processor
        .process_line("Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello")
        .unwrap();
    assert!(result.is_empty());

    let result = processor
        .process_line("Comment: 0,0:00:05.00,0:00:10.00,Default,Note")
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn binary_line_processing() {
    let mut processor = LineProcessor::new();

    // Test Fonts section
    processor.state.enter_section(SectionKind::Fonts);
    let result = processor.process_line("fontname: Arial.ttf").unwrap();
    assert!(result.is_empty());

    let result = processor
        .process_line("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD")
        .unwrap();
    assert!(result.is_empty());

    // Test Graphics section
    processor.state.enter_section(SectionKind::Graphics);
    let result = processor.process_line("graphic: logo.png").unwrap();
    assert!(result.is_empty());

    let result = processor
        .process_line("0123456789ABCDEF0123456789ABCDEF")
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn event_continuation_processing() {
    let mut processor = LineProcessor::new();
    processor.state.enter_event(SectionKind::Events);

    let result = processor.process_line("  continuation data").unwrap();
    assert!(result.is_empty());
    // Should return to section state
    assert!(processor.state.is_in_section());
    assert_eq!(processor.state.current_section(), Some(SectionKind::Events));

    // Test with empty continuation
    processor.state.enter_event(SectionKind::Events);
    let result = processor.process_line("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn unknown_section_processing() {
    let mut processor = LineProcessor::new();
    processor.state.enter_section(SectionKind::Unknown);

    let result = processor
        .process_line("Any content in unknown section")
        .unwrap();
    assert!(result.is_empty());

    let result = processor.process_line("Key: Value").unwrap();
    assert!(result.is_empty());
}

#[test]
fn format_context_updates() {
    let mut processor = LineProcessor::new();

    // Test styles format
    processor.state.enter_section(SectionKind::Styles);
    processor.context.enter_section(SectionKind::Styles);

    assert!(processor.context.styles_format.is_none());
    processor
        .process_line("Format: Name, Fontname, Fontsize, Bold")
        .unwrap();
    assert!(processor.context.styles_format.is_some());

    // Test events format
    processor.state.enter_section(SectionKind::Events);
    processor.context.enter_section(SectionKind::Events);

    assert!(processor.context.events_format.is_none());
    processor
        .process_line("Format: Layer, Start, End, Style, Text")
        .unwrap();
    assert!(processor.context.events_format.is_some());
}

#[test]
fn complex_processing_sequence() {
    let mut processor = LineProcessor::new();

    // Process a complete mini-script
    let lines = [
        "[Script Info]",
        "Title: Test",
        "Author: Tester",
        "",
        "[V4+ Styles]",
        "Format: Name, Fontname, Fontsize",
        "Style: Default,Arial,20",
        "",
        "[Events]",
        "Format: Layer, Start, End, Style, Text",
        "Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello World",
        "; End of script",
    ];

    for line in &lines {
        let result = processor.process_line(line).unwrap();
        assert!(result.is_empty());
    }

    assert_eq!(processor.context.line_number, lines.len());
    assert!(processor.context.events_format.is_some());
    assert!(processor.context.styles_format.is_some());
}
