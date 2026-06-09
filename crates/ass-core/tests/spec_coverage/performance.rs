//! Performance and memory-efficiency targets for the comprehensive script.
//!
//! Validates parse/analysis latency budgets and the zero-copy guarantee that
//! event text references the original source buffer.

use ass_core::{
    analysis::ScriptAnalysis,
    parser::ast::{Section, SectionType},
    Script,
};

use super::common::COMPREHENSIVE_SCRIPT;

#[test]
fn test_performance_targets() {
    use std::time::Instant;

    // Test parsing performance
    let start = Instant::now();
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");
    let parse_duration = start.elapsed();

    // Should parse within 5ms target
    assert!(
        parse_duration.as_millis() < 5,
        "Parsing took {}ms, should be <5ms",
        parse_duration.as_millis()
    );

    // Test analysis performance
    let start = Instant::now();
    let _analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze script");
    let analysis_duration = start.elapsed();

    // Analysis should complete reasonably quickly
    assert!(
        analysis_duration.as_millis() < 50,
        "Analysis took {}ms, should be <50ms",
        analysis_duration.as_millis()
    );
}

#[test]
fn test_memory_efficiency() {
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");

    // Verify zero-copy design by checking string references
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        if let Some(first_event) = events.first() {
            // Text should reference original source, not be copied
            let original_ptr = COMPREHENSIVE_SCRIPT.as_ptr() as usize;
            let text_ptr = first_event.text.as_ptr() as usize;

            assert!(
                text_ptr >= original_ptr,
                "Text should reference original source for zero-copy design"
            );
            assert!(
                text_ptr < original_ptr + COMPREHENSIVE_SCRIPT.len(),
                "Text should reference original source for zero-copy design"
            );
        }
    }
}
