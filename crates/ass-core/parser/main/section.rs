//! Single-section header parsing and dispatch.
//!
//! Implements [`Parser::parse_section`], which reads a section header and
//! delegates to the appropriate specialized parser, updating parser state and
//! recovering from unknown or malformed sections.

use super::Parser;
use crate::{
    parser::{
        ast::Section,
        binary_data::{FontsParser, GraphicsParser},
        errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue},
        sections::{EventsParser, ScriptInfoParser, StylesParser},
    },
    utils::CoreError,
    Result,
};
use alloc::string::ToString;

impl<'a> Parser<'a> {
    /// Parse a single section (e.g., [Script Info])
    pub(super) fn parse_section(&mut self) -> Result<Section<'a>> {
        if !self.source[self.position..].starts_with('[') {
            return Err(CoreError::from(ParseError::ExpectedSectionHeader {
                line: self.line,
            }));
        }

        let header_end = self.source[self.position..].find(']').ok_or_else(|| {
            CoreError::from(ParseError::UnclosedSectionHeader { line: self.line })
        })? + self.position;

        let section_name = &self.source[self.position + 1..header_end];
        self.position = header_end + 1;
        self.skip_line();

        let start_line = self.line;

        match section_name.trim() {
            "Script Info" => {
                let parser = ScriptInfoParser::new(self.source, self.position, start_line);
                let (section, detected_version, issues, final_position, final_line) =
                    parser.parse().map_err(CoreError::from)?;

                // Update parser state
                if let Some(version) = detected_version {
                    self.version = version;
                }
                self.issues.extend(issues);
                self.position = final_position;
                self.line = final_line;

                Ok(section)
            }
            "V4+ Styles" | "V4 Styles" | "V4++ Styles" => {
                let parser = StylesParser::new(self.source, self.position, start_line);
                let (section, format, issues, final_position, final_line) =
                    parser.parse().map_err(CoreError::from)?;

                // Update parser state
                self.styles_format = format;
                self.issues.extend(issues);
                self.position = final_position;
                self.line = final_line;

                Ok(section)
            }
            "Events" => {
                let parser = EventsParser::new(self.source, self.position, start_line);
                let (section, format, issues, final_position, final_line) =
                    parser.parse().map_err(CoreError::from)?;

                // Update parser state
                self.events_format = format;
                self.issues.extend(issues);
                self.position = final_position;
                self.line = final_line;

                Ok(section)
            }
            "Fonts" => {
                let (section, final_position, final_line) =
                    FontsParser::parse(self.source, self.position, start_line);

                // Update parser state
                self.position = final_position;
                self.line = final_line;

                Ok(section)
            }
            "Graphics" => {
                let (section, final_position, final_line) =
                    GraphicsParser::parse(self.source, self.position, start_line);

                // Update parser state
                self.position = final_position;
                self.line = final_line;

                Ok(section)
            }
            _ => {
                #[cfg(feature = "plugins")]
                if self.registry.is_some() {
                    // Try to process unknown section with registered processors
                    if let Some(result) = self.try_process_with_registry(section_name, start_line) {
                        return result;
                    }
                }

                let suggestion = self.skip_to_next_section();
                let error = ParseError::UnknownSection {
                    section: section_name.to_string(),
                    line: self.line,
                };

                // Add suggestion to issues if we found one
                if let Some(suggestion_text) = suggestion {
                    self.issues.push(ParseIssue {
                        severity: IssueSeverity::Info,
                        category: IssueCategory::Structure,
                        message: suggestion_text,
                        line: self.line,
                        column: Some(0),
                        span: None,
                        suggestion: None,
                    });
                }

                Err(CoreError::from(error))
            }
        }
    }
}
