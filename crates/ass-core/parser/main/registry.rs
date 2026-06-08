//! Plugin-driven processing for unknown sections.
//!
//! Implements the `Parser::try_process_with_registry` helper, which dispatches
//! unknown section content to registered section processors and records the
//! outcome as parse issues.

#[cfg(feature = "plugins")]
use super::Parser;
#[cfg(feature = "plugins")]
use crate::{
    parser::{
        ast::Section,
        errors::{IssueCategory, IssueSeverity, ParseIssue},
    },
    plugin::SectionResult,
    Result,
};
#[cfg(feature = "plugins")]
use alloc::{format, vec::Vec};

#[cfg(feature = "plugins")]
impl<'a> Parser<'a> {
    /// Try to process unknown section using registered processors
    pub(super) fn try_process_with_registry(
        &mut self,
        section_name: &str,
        start_line: usize,
    ) -> Option<Result<Section<'a>>> {
        let registry = self.registry?;

        // Collect section lines
        let mut lines = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            let line_start = self.position;
            let line_end = self.source[self.position..]
                .find('\n')
                .map_or(self.source.len(), |i| self.position + i);

            if line_end > line_start {
                let line = &self.source[line_start..line_end];
                lines.push(line);
            }

            self.skip_line();
        }

        // Try to process with registry
        match registry.process_section(section_name, section_name, &lines) {
            Some(SectionResult::Processed) => {
                // Create a custom section for processed content
                // For now, we'll create a generic unknown section
                // In a full implementation, this would create a proper custom section type
                self.issues.push(ParseIssue::new(
                    IssueSeverity::Info,
                    IssueCategory::Structure,
                    format!("Section '{section_name}' processed by plugin"),
                    start_line,
                ));

                // Return success but skip the section content since we don't have
                // a proper custom section AST type yet
                None
            }
            Some(SectionResult::Failed(msg)) => {
                self.issues.push(ParseIssue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Structure,
                    format!("Plugin failed to process section '{section_name}': {msg}"),
                    start_line,
                ));
                None
            }
            Some(SectionResult::Ignored) | None => None,
        }
    }
}
