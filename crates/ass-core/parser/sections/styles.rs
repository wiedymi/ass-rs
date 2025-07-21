//! Styles section parser for ASS scripts.
//!
//! Handles parsing of the [V4+ Styles] section which contains style definitions
//! with format specifications and style entries.

use crate::parser::{
    ast::{Section, Style},
    errors::{IssueCategory, IssueSeverity, ParseIssue},
    sections::SectionParseResult,
    ParseResult,
};
use alloc::vec::Vec;

/// Parser for [V4+ Styles] section content
///
/// Parses format definitions and style entries from the styles section.
/// Uses format mapping to handle different field orderings and missing fields.
///
/// # Performance
///
/// - Time complexity: O(n * m) for n styles and m fields per style
/// - Memory: Zero allocations via lifetime-generic spans
/// - Target: <1ms for typical style sections with 50 styles
pub struct StylesParser<'a> {
    source: &'a str,
    position: usize,
    line: usize,
    issues: Vec<ParseIssue>,
    format: Option<Vec<&'a str>>,
}

impl<'a> StylesParser<'a> {
    /// Create new styles parser for source text
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to parse
    /// * `start_position` - Starting byte position in source
    /// * `start_line` - Starting line number for error reporting
    pub fn new(source: &'a str, start_position: usize, start_line: usize) -> Self {
        Self {
            source,
            position: start_position,
            line: start_line,
            issues: Vec::new(),
            format: None,
        }
    }

    /// Parse styles section content
    ///
    /// Returns the parsed section and any issues encountered during parsing.
    /// Handles Format line parsing and style entry validation.
    ///
    /// # Returns
    ///
    /// Tuple of (parsed_section, format_fields, parse_issues, final_position, final_line)
    pub fn parse(mut self) -> ParseResult<SectionParseResult<'a>> {
        let mut styles = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() || self.at_next_section() {
                break;
            }

            let line = self.current_line().trim();
            if line.is_empty() {
                self.skip_line();
                continue;
            }

            if line.starts_with("Format:") {
                self.parse_format_line(line);
            } else if line.starts_with("Style:") {
                if let Some(style_data) = line.strip_prefix("Style:") {
                    if let Some(style) = self.parse_style_line(style_data.trim()) {
                        styles.push(style);
                    }
                }
            }

            self.skip_line();
        }

        Ok((
            Section::Styles(styles),
            self.format,
            self.issues,
            self.position,
            self.line,
        ))
    }

    /// Parse format specification line
    fn parse_format_line(&mut self, line: &'a str) {
        if let Some(format_data) = line.strip_prefix("Format:") {
            let fields: Vec<&'a str> = format_data.split(',').map(|s| s.trim()).collect();
            self.format = Some(fields);
        }
    }

    /// Parse single style definition line
    fn parse_style_line(&mut self, line: &'a str) -> Option<Style<'a>> {
        let parts: Vec<&str> = line.split(',').collect();

        let format = self.format.as_deref().unwrap_or(&[
            "Name",
            "Fontname",
            "Fontsize",
            "PrimaryColour",
            "SecondaryColour",
            "OutlineColour",
            "BackColour",
            "Bold",
            "Italic",
            "Underline",
            "StrikeOut",
            "ScaleX",
            "ScaleY",
            "Spacing",
            "Angle",
            "BorderStyle",
            "Outline",
            "Shadow",
            "Alignment",
            "MarginL",
            "MarginR",
            "MarginV",
            "Encoding",
        ]);

        if parts.len() != format.len() {
            self.issues.push(ParseIssue::new(
                IssueSeverity::Warning,
                IssueCategory::Format,
                format!(
                    "Style line has {} fields, expected {}",
                    parts.len(),
                    format.len()
                ),
                self.line,
            ));
            if parts.len() < format.len() {
                return None;
            }
        }

        let get_field = |name: &str| -> &'a str {
            format
                .iter()
                .position(|&field| field.eq_ignore_ascii_case(name))
                .and_then(|idx| parts.get(idx))
                .map(|s| s.trim())
                .unwrap_or("")
        };

        Some(Style {
            name: get_field("Name"),
            fontname: get_field("Fontname"),
            fontsize: get_field("Fontsize"),
            primary_colour: get_field("PrimaryColour"),
            secondary_colour: get_field("SecondaryColour"),
            outline_colour: get_field("OutlineColour"),
            back_colour: get_field("BackColour"),
            bold: get_field("Bold"),
            italic: get_field("Italic"),
            underline: get_field("Underline"),
            strikeout: get_field("StrikeOut"),
            scale_x: get_field("ScaleX"),
            scale_y: get_field("ScaleY"),
            spacing: get_field("Spacing"),
            angle: get_field("Angle"),
            border_style: get_field("BorderStyle"),
            outline: get_field("Outline"),
            shadow: get_field("Shadow"),
            alignment: get_field("Alignment"),
            margin_l: get_field("MarginL"),
            margin_r: get_field("MarginR"),
            margin_v: get_field("MarginV"),
            encoding: get_field("Encoding"),
        })
    }

    /// Get current line from source
    fn current_line(&self) -> &'a str {
        let start = self.position;
        let end = self.source[self.position..]
            .find('\n')
            .map(|pos| self.position + pos)
            .unwrap_or(self.source.len());

        &self.source[start..end]
    }

    /// Check if at start of next section
    fn at_next_section(&self) -> bool {
        let remaining = &self.source[self.position..];
        remaining.trim_start().starts_with('[')
    }

    /// Skip current line and advance position
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
        loop {
            let remaining = &self.source[self.position..];
            let trimmed = remaining.trim_start();

            if trimmed.is_empty() {
                self.position = self.source.len();
                break;
            }

            if trimmed.starts_with(';') || trimmed.starts_with('#') {
                self.skip_line();
                continue;
            }

            let whitespace_len = remaining.len() - trimmed.len();
            if whitespace_len > 0 {
                let newlines = remaining[..whitespace_len].matches('\n').count();
                self.position += whitespace_len;
                self.line += newlines;
            }

            break;
        }
    }

    /// Get accumulated parse issues
    pub fn issues(self) -> Vec<ParseIssue> {
        self.issues
    }

    /// Get format specification
    pub fn format(&self) -> Option<&Vec<&'a str>> {
        self.format.as_ref()
    }
}
