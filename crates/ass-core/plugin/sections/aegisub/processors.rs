//! Section processor implementations for Aegisub-specific sections.
//!
//! Contains [`AegisubProjectProcessor`] and [`AegisubExtradataProcessor`],
//! which validate `[Aegisub Project]` and `[Aegisub Extradata]` sections
//! that extend the standard ASS format.

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

            // Aegisub project lines should be in key=value or key: value format
            if !line.contains('=') && !line.contains(':') {
                return SectionResult::Failed(String::from(
                    "Invalid Aegisub project line format (expected key=value or key: value)",
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
