//! Dispatch, lifecycle, and section-header tests for [`LineProcessor`].

use super::super::state::SectionKind;
use super::*;

#[cfg(not(feature = "std"))]
#[test]
fn processor_creation() {
    let processor = LineProcessor::new();
    assert_eq!(processor.context.line_number, 0);
    assert!(!processor.state.is_in_section());
}

#[test]
fn section_header_processing() {
    let mut processor = LineProcessor::new();
    let result = processor.process_line("[Script Info]").unwrap();
    assert!(result.is_empty());
    assert!(processor.state.is_in_section());
    assert_eq!(
        processor.state.current_section(),
        Some(SectionKind::ScriptInfo)
    );
}

#[test]
fn comment_line_skipping() {
    let mut processor = LineProcessor::new();
    let result = processor.process_line("; This is a comment").unwrap();
    assert!(result.is_empty());
    assert_eq!(processor.context.line_number, 1);
}

#[test]
fn processor_reset() {
    let mut processor = LineProcessor::new();
    processor.state.enter_section(SectionKind::Events);
    processor.context.next_line();

    processor.reset();
    assert!(!processor.state.is_in_section());
    assert_eq!(processor.context.line_number, 0);
}

#[test]
fn processor_default() {
    let processor = LineProcessor::default();
    assert_eq!(processor.context.line_number, 0);
    assert!(!processor.state.is_in_section());
}

#[test]
fn empty_line_processing() {
    let mut processor = LineProcessor::new();
    let result = processor.process_line("").unwrap();
    assert!(result.is_empty());
    assert_eq!(processor.context.line_number, 1);

    let result = processor.process_line("   \t  ").unwrap();
    assert!(result.is_empty());
    assert_eq!(processor.context.line_number, 2);
}

#[test]
fn different_comment_formats() {
    let mut processor = LineProcessor::new();

    let result = processor.process_line("; Standard comment").unwrap();
    assert!(result.is_empty());

    let result = processor.process_line("!: Aegisub comment").unwrap();
    assert!(result.is_empty());

    assert_eq!(processor.context.line_number, 2);
}

#[test]
fn all_section_headers() {
    let mut processor = LineProcessor::new();

    let sections = [
        "[Script Info]",
        "[V4+ Styles]",
        "[Events]",
        "[Fonts]",
        "[Graphics]",
        "[Unknown Section]",
    ];

    for section in &sections {
        let result = processor.process_line(section).unwrap();
        assert!(result.is_empty());
        assert!(processor.state.is_in_section());
    }
}

#[test]
fn content_outside_sections() {
    let mut processor = LineProcessor::new();
    // Start in ExpectingSection state
    assert!(!processor.state.is_in_section());

    let result = processor
        .process_line("Random content outside sections")
        .unwrap();
    assert!(result.is_empty());
    // Should still not be in a section
    assert!(!processor.state.is_in_section());
}

#[test]
fn section_header_edge_cases() {
    let mut processor = LineProcessor::new();

    // Section header with spaces
    let result = processor.process_line("[ Script Info ]").unwrap();
    assert!(result.is_empty());
    assert!(processor.state.is_in_section());

    // Empty section header
    let result = processor.process_line("[]").unwrap();
    assert!(result.is_empty());

    // Malformed section headers should not crash
    let result = processor.process_line("[Unclosed section").unwrap();
    assert!(result.is_empty());

    let result = processor.process_line("Unclosed section]").unwrap();
    assert!(result.is_empty());
}

#[test]
fn line_counter_increments() {
    let mut processor = LineProcessor::new();
    assert_eq!(processor.context.line_number, 0);

    processor.process_line("Line 1").unwrap();
    assert_eq!(processor.context.line_number, 1);

    processor.process_line("Line 2").unwrap();
    assert_eq!(processor.context.line_number, 2);

    processor.process_line("").unwrap();
    assert_eq!(processor.context.line_number, 3);
}

#[test]
fn whitespace_handling() {
    let mut processor = LineProcessor::new();

    // Test various whitespace scenarios
    processor.process_line("   [Script Info]   ").unwrap();
    assert!(processor.state.is_in_section());

    processor.process_line("\t\tTitle: Test\t\t").unwrap();

    processor
        .process_line("   ; Comment with spaces   ")
        .unwrap();

    processor.process_line("\t\n").unwrap(); // Tab followed by what looks like newline
}
