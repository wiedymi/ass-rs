//! Behavioural tests for the Aegisub section processor implementations.

use super::*;
use crate::plugin::{SectionProcessor, SectionResult};
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn aegisub_project_processor_valid() {
    let processor = AegisubProjectProcessor;
    let lines = vec!["Active Line: 0", "Video Position: 0"];

    assert_eq!(
        processor.process("Aegisub Project", &lines),
        SectionResult::Processed
    );
}

#[test]
fn aegisub_project_processor_invalid_header() {
    let processor = AegisubProjectProcessor;
    let lines = vec!["Active Line: 0"];

    assert_eq!(
        processor.process("Wrong Header", &lines),
        SectionResult::Ignored
    );
}

#[test]
fn aegisub_project_processor_invalid_format() {
    let processor = AegisubProjectProcessor;
    let lines = vec!["Invalid line without equals"];

    assert!(matches!(
        processor.process("Aegisub Project", &lines),
        SectionResult::Failed(_)
    ));
}

#[test]
fn aegisub_extradata_processor_valid() {
    let processor = AegisubExtradataProcessor;
    let lines = vec!["Data: some_binary_data", "More: extra_info"];

    assert_eq!(
        processor.process("Aegisub Extradata", &lines),
        SectionResult::Processed
    );
}

#[test]
fn aegisub_extradata_processor_long_line() {
    let processor = AegisubExtradataProcessor;
    let long_line = "x".repeat(20000);
    let lines = vec![long_line.as_str()];

    assert!(matches!(
        processor.process("Aegisub Extradata", &lines),
        SectionResult::Failed(_)
    ));
}

#[test]
fn processor_names_correct() {
    assert_eq!(AegisubProjectProcessor.name(), "Aegisub Project");
    assert_eq!(AegisubExtradataProcessor.name(), "Aegisub Extradata");
}

#[test]
fn create_aegisub_processors_returns_two() {
    let processors = create_aegisub_processors();
    assert_eq!(processors.len(), 2);
}

#[test]
fn case_insensitive_headers() {
    let processor = AegisubProjectProcessor;
    let lines = vec!["Active Line: 0"];

    assert_eq!(
        processor.process("aegisub project", &lines),
        SectionResult::Processed
    );
    assert_eq!(
        processor.process("AEGISUB PROJECT", &lines),
        SectionResult::Processed
    );
}
