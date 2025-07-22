//! Aegisub-specific section processors for ASS compatibility
//!
//! Implements section processors for Aegisub-specific sections that extend
//! the standard ASS format. These processors handle project metadata and
//! additional data storage used by the Aegisub subtitle editor.
//!
//! # Supported Sections
//!
//! - `[Aegisub Project]`: Project-specific metadata and settings
//! - `[Aegisub Extradata]`: Additional data storage for extended functionality
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - O(n) processing where n = number of lines
//! - Minimal memory footprint per processor

use crate::plugin::{SectionProcessor, SectionResult};
use alloc::string::String;

/// Handler for Aegisub Project section
///
/// Processes `[Aegisub Project]` sections containing project-specific metadata
/// such as active line tracking, scroll position, and editor state.
pub struct AegisubProjectProcessor;

impl SectionProcessor for AegisubProjectProcessor {
    fn name(&self) -> &'static str {
        "Aegisub Project"
    }

    fn process(&self, header: &str, lines: &[&str]) -> SectionResult {
        if !header.eq_ignore_ascii_case("Aegisub Project") {
            return SectionResult::Ignored;
        }

        // Validate Aegisub project format
        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('!') {
                continue;
            }

            // Aegisub project lines should be in key=value format
            if !line.contains('=') {
                return SectionResult::Failed(String::from(
                    "Invalid Aegisub project line format (expected key=value)",
                ));
            }
        }

        SectionResult::Processed
    }

    fn validate(&self, header: &str, lines: &[&str]) -> bool {
        header.eq_ignore_ascii_case("Aegisub Project") && !lines.is_empty()
    }
}

/// Handler for Aegisub Extradata section
///
/// Processes `[Aegisub Extradata]` sections containing additional data storage
/// for extended functionality beyond standard ASS format.
pub struct AegisubExtradataProcessor;

impl SectionProcessor for AegisubExtradataProcessor {
    fn name(&self) -> &'static str {
        "Aegisub Extradata"
    }

    fn process(&self, header: &str, lines: &[&str]) -> SectionResult {
        if !header.eq_ignore_ascii_case("Aegisub Extradata") {
            return SectionResult::Ignored;
        }

        // Validate extradata format - typically binary data or key-value pairs
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Extradata can be various formats, so we're permissive
            // Just ensure it's not completely malformed
            if line.len() > 10000 {
                return SectionResult::Failed(String::from(
                    "Extradata line exceeds maximum length",
                ));
            }
        }

        SectionResult::Processed
    }

    fn validate(&self, header: &str, _lines: &[&str]) -> bool {
        header.eq_ignore_ascii_case("Aegisub Extradata")
    }
}

/// Create all Aegisub section processors
///
/// Returns a vector of boxed section processors for all Aegisub-specific sections.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, sections::aegisub::create_aegisub_processors};
///
/// let mut registry = ExtensionRegistry::new();
/// for processor in create_aegisub_processors() {
///     registry.register_section_processor(processor).unwrap();
/// }
/// ```
pub fn create_aegisub_processors() -> alloc::vec::Vec<alloc::boxed::Box<dyn SectionProcessor>> {
    alloc::vec![
        alloc::boxed::Box::new(AegisubProjectProcessor),
        alloc::boxed::Box::new(AegisubExtradataProcessor),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
