//! End-to-end parsing and analysis of the comprehensive spec-coverage script.
//!
//! Validates that every section, event type, and embedded media block of the
//! shared sample parses correctly and yields the expected analysis output.

use ass_core::{
    analysis::ScriptAnalysis,
    parser::ast::{EventType, Section, SectionType},
    Script,
};

use super::common::COMPREHENSIVE_SCRIPT;

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_comprehensive_spec_coverage() {
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");

    // Verify script info parsing
    if let Some(Section::ScriptInfo(script_info)) = script.find_section(SectionType::ScriptInfo) {
        let title_field = script_info.fields.iter().find(|(key, _)| *key == "Title");
        assert!(title_field.is_some());
        let script_type_field = script_info
            .fields
            .iter()
            .find(|(key, _)| *key == "ScriptType");
        assert_eq!(script_type_field.map(|(_, value)| *value), Some("v4.00+"));
    } else {
        panic!("Script Info section should be present");
    }

    // Verify styles parsing
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        assert_eq!(styles.len(), 3);
        let default_style = styles.iter().find(|s| s.name == "Default");
        assert!(default_style.is_some());
    } else {
        panic!("Styles section should be present");
    }

    // Verify events parsing - should include all event types
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert!(!events.is_empty());

        // Count different event types
        let dialogue_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Dialogue))
            .count();
        let comment_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Comment))
            .count();
        let picture_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Picture))
            .count();
        let sound_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Sound))
            .count();
        let movie_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Movie))
            .count();
        let command_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Command))
            .count();

        assert!(dialogue_count > 0, "Should have dialogue events");
        assert!(comment_count > 0, "Should have comment events");
        assert!(picture_count > 0, "Should have picture events");
        assert!(sound_count > 0, "Should have sound events");
        assert!(movie_count > 0, "Should have movie events");
        assert!(command_count > 0, "Should have command events");
    } else {
        panic!("Events section should be present");
    }

    // Verify fonts section
    if let Some(Section::Fonts(fonts)) = script.find_section(SectionType::Fonts) {
        assert_eq!(fonts.len(), 2);
        assert_eq!(fonts[0].filename, "CustomFont.ttf");
        assert_eq!(fonts[1].filename, "AnotherFont.otf");
        assert!(!fonts[0].data_lines.is_empty());
        assert!(!fonts[1].data_lines.is_empty());
    } else {
        panic!("Fonts section should be present");
    }

    // Verify graphics section
    if let Some(Section::Graphics(graphics)) = script.find_section(SectionType::Graphics) {
        assert_eq!(graphics.len(), 2);
        assert_eq!(graphics[0].filename, "background.png");
        assert_eq!(graphics[1].filename, "overlay.jpg");
        assert!(!graphics[0].data_lines.is_empty());
        assert!(!graphics[1].data_lines.is_empty());
    } else {
        panic!("Graphics section should be present");
    }
}

#[test]
fn test_comprehensive_analysis() {
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");
    let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze script");

    // Verify dialogue analysis
    let dialogue_info = analysis.dialogue_info();
    assert!(!dialogue_info.is_empty());

    // Check for complex animations (should have high complexity scores)
    assert!(
        dialogue_info
            .iter()
            .any(|info| info.complexity_score() > 50),
        "Should have complex animation events"
    );

    // Check for events with override tags
    assert!(
        dialogue_info
            .iter()
            .any(|info| !info.text_analysis().override_tags().is_empty()),
        "Should have events with override tags"
    );

    // Check for bidirectional text
    assert!(
        dialogue_info
            .iter()
            .any(|info| info.text_analysis().has_bidi_text()),
        "Should have bidirectional text events"
    );

    // Check for Unicode complexity
    assert!(
        dialogue_info
            .iter()
            .any(|info| info.text_analysis().has_complex_unicode()),
        "Should have complex Unicode events"
    );

    // Verify overlap detection works with multiple events
    let perf_summary = analysis.performance_summary();
    // Some events should overlap (like the alignment test events)
    assert!(
        perf_summary.overlapping_events > 0,
        "Should detect overlapping events"
    );

    // Verify style analysis
    let resolved_styles = analysis.resolved_styles();
    assert!(!resolved_styles.is_empty());
}
