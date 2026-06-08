//! Top-level script parsing driver.
//!
//! Implements [`Parser::parse`], which validates input limits and BOM handling,
//! then iterates over sections, recording issues and recovering from errors.

use super::Parser;
use crate::{
    parser::{
        errors::{IssueCategory, IssueSeverity, ParseIssue},
        script::Script,
    },
    utils::errors::{encoding::validate_bom_handling, resource::check_input_size_limit},
};
use alloc::{format, vec::Vec};

impl<'a> Parser<'a> {
    /// Parse complete script
    pub fn parse(mut self) -> Script<'a> {
        // Check input size limit to prevent DoS attacks (50MB limit)
        const MAX_INPUT_SIZE: usize = 50 * 1024 * 1024; // 50MB
        if let Err(e) = check_input_size_limit(self.source.len(), MAX_INPUT_SIZE) {
            self.issues.push(ParseIssue::new(
                IssueSeverity::Error,
                IssueCategory::Security,
                format!("Input size limit exceeded: {e}"),
                self.line,
            ));
            // Return early with empty script for security
            return Script::from_parts(
                self.source,
                self.version,
                Vec::new(),
                self.issues,
                self.styles_format,
                self.events_format,
            );
        }

        // Validate and handle BOM if present
        if let Err(e) = validate_bom_handling(self.source.as_bytes()) {
            self.issues.push(ParseIssue::new(
                IssueSeverity::Warning,
                IssueCategory::Format,
                format!("BOM validation warning: {e}"),
                self.line,
            ));
        }

        // Skip UTF-8 BOM if present
        if self.source.starts_with('\u{FEFF}') {
            self.position = 3;
        }

        while self.position < self.source.len() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() {
                break;
            }

            match self.parse_section() {
                Ok(section) => self.sections.push(section),
                Err(e) => {
                    let (severity, message) = if e.to_string().contains("Unknown section") {
                        (IssueSeverity::Warning, e.to_string())
                    } else {
                        (
                            IssueSeverity::Error,
                            format!("Failed to parse section: {e}"),
                        )
                    };

                    self.issues.push(ParseIssue::new(
                        severity,
                        IssueCategory::Structure,
                        message,
                        self.line,
                    ));

                    self.skip_to_next_section();
                }
            }
        }

        Script::from_parts(
            self.source,
            self.version,
            self.sections,
            self.issues,
            self.styles_format,
            self.events_format,
        )
    }
}
