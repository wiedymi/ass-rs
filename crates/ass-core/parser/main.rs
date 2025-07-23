//! Main parser coordination and dispatch logic
//!
//! Contains the core `Parser` struct that orchestrates parsing of different
//! ASS script sections and handles error recovery.

use crate::{utils::CoreError, Result, ScriptVersion};
use alloc::{format, string::ToString, vec::Vec};

use super::{
    ast::Section,
    binary_data::{FontsParser, GraphicsParser},
    errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue},
    script::Script,
    sections::{EventsParser, ScriptInfoParser, StylesParser},
};

/// Internal parser state for coordinating section parsing
pub(super) struct Parser<'a> {
    /// Source text being parsed
    source: &'a str,
    /// Current byte position in source
    position: usize,
    /// Current line number for error reporting
    line: usize,
    /// Detected script version
    version: ScriptVersion,
    /// Parsed sections accumulated so far
    sections: Vec<Section<'a>>,
    /// Parse issues and warnings
    issues: Vec<ParseIssue>,
    /// Format fields for [V4+ Styles] section
    styles_format: Option<Vec<&'a str>>,
    /// Format fields for [Events] section
    events_format: Option<Vec<&'a str>>,
}

impl<'a> Parser<'a> {
    /// Create new parser for source text
    pub const fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            version: ScriptVersion::AssV4, // Default, updated when ScriptType found
            sections: Vec::new(),
            issues: Vec::new(),
            styles_format: None,
            events_format: None,
        }
    }

    /// Parse complete script
    pub fn parse(mut self) -> Script<'a> {
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

        Script::from_parts(self.source, self.version, self.sections, self.issues)
    }

    /// Parse a single section (e.g., [Script Info])
    fn parse_section(&mut self) -> Result<Section<'a>> {
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
            "V4+ Styles" | "V4 Styles" => {
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
                let parser = FontsParser::new(self.source, self.position, start_line);
                let section = parser.parse();

                // Update position to end of fonts section
                self.position = self.find_section_end();

                Ok(section)
            }
            "Graphics" => {
                let parser = GraphicsParser::new(self.source, self.position, start_line);
                let section = parser.parse();

                // Update position to end of graphics section
                self.position = self.find_section_end();

                Ok(section)
            }
            _ => {
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

    /// Find end of current section for binary data sections
    fn find_section_end(&mut self) -> usize {
        while self.position < self.source.len() {
            if self.at_next_section() {
                break;
            }
            self.skip_line();
        }

        self.position
    }

    /// Check if at start of next section
    fn at_next_section(&self) -> bool {
        self.source[self.position..].trim_start().starts_with('[')
    }

    /// Skip to next line
    fn skip_line(&mut self) {
        if let Some(newline_pos) = self.source[self.position..].find('\n') {
            self.position += newline_pos + 1;
            self.line += 1;
        } else {
            self.position = self.source.len();
        }
    }

    /// Skip whitespace and comment lines
    fn skip_whitespace_and_comments(&mut self) {
        while self.position < self.source.len() {
            let remaining = &self.source[self.position..];
            let trimmed = remaining.trim_start();

            if trimmed.starts_with(';') || trimmed.starts_with("!:") {
                self.skip_line();
            } else if trimmed != remaining {
                self.position += remaining.len() - trimmed.len();
            } else {
                break;
            }
        }
    }

    /// Skip to next section for error recovery
    fn skip_to_next_section(&mut self) -> Option<String> {
        let mut suggestion = None;

        while self.position < self.source.len() {
            if self.at_next_section() {
                break;
            }

            // Look for patterns that suggest what section this might be
            let line_start = self.position;
            let line_end = self.source[self.position..]
                .find('\n')
                .map_or(self.source.len(), |i| self.position + i);

            if line_end > line_start {
                let line = &self.source[line_start..line_end];

                // Check for common section entry patterns
                if suggestion.is_none() {
                    if line.trim_start().starts_with("Style:") {
                        suggestion = Some("Did you mean '[V4+ Styles]'?".to_string());
                    } else if line.trim_start().starts_with("Dialogue:")
                        || line.trim_start().starts_with("Comment:")
                    {
                        suggestion = Some("Did you mean '[Events]'?".to_string());
                    } else if line.trim_start().starts_with("Title:")
                        || line.trim_start().starts_with("ScriptType:")
                    {
                        suggestion = Some("Did you mean '[Script Info]'?".to_string());
                    } else if line.trim_start().starts_with("Format:") {
                        // Format lines could be in styles or events
                        let remaining = &self.source[self.position..];
                        if remaining.contains("Dialogue:") {
                            suggestion = Some("Did you mean '[Events]'?".to_string());
                        } else if remaining.contains("Style:") {
                            suggestion = Some("Did you mean '[V4+ Styles]'?".to_string());
                        }
                    }
                }
            }

            self.skip_line();
        }

        suggestion
    }
}
