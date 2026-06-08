//! Construction, driving, and scanning helpers for [`StylesParser`].
//!
//! Hosts the parser constructors, the top-level [`StylesParser::parse`] driver,
//! and the whitespace/section scanning helpers used while iterating lines.

use super::StylesParser;
use crate::parser::{
    ast::Section, position_tracker::PositionTracker, sections::SectionParseResult, ParseResult,
};
use alloc::{vec, vec::Vec};

impl<'a> StylesParser<'a> {
    /// Create new styles parser for source text
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to parse
    /// * `start_position` - Starting byte position in source
    /// * `start_line` - Starting line number for error reporting
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Can't be const due to Vec::new()
    pub fn new(source: &'a str, start_position: usize, start_line: usize) -> Self {
        Self {
            tracker: PositionTracker::new_at(
                source,
                start_position,
                u32::try_from(start_line).unwrap_or(u32::MAX),
                1,
            ),
            issues: Vec::new(),
            format: None,
        }
    }

    /// Create a new parser with a pre-known format for incremental parsing
    #[must_use]
    pub fn with_format(
        source: &'a str,
        format: &[&'a str],
        start_position: usize,
        start_line: u32,
    ) -> Self {
        Self {
            tracker: PositionTracker::new_at(source, start_position, start_line, 1),
            issues: Vec::new(),
            format: Some(format.to_vec()),
        }
    }

    /// Parse styles section content
    ///
    /// Returns the parsed section and any issues encountered during parsing.
    /// Handles Format line parsing and style entry validation.
    ///
    /// # Returns
    ///
    /// Tuple of (`parsed_section`, `format_fields`, `parse_issues`, `final_position`, `final_line`)
    ///
    /// # Errors
    ///
    /// Returns an error if the styles section contains malformed format lines or
    /// other unrecoverable syntax errors.
    pub fn parse(mut self) -> ParseResult<SectionParseResult<'a>> {
        let mut styles = Vec::new();

        while !self.tracker.is_at_end() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.tracker.is_at_end() || self.at_next_section() {
                break;
            }

            let line_start = self.tracker.checkpoint();
            let line = self.current_line().trim();

            if line.is_empty() {
                self.tracker.skip_line();
                continue;
            }

            if line.starts_with("Format:") {
                self.parse_format_line(line);
            } else if line.starts_with("Style:") {
                if let Some(style_data) = line.strip_prefix("Style:") {
                    if let Some(style) =
                        self.parse_style_line_internal(style_data.trim(), &line_start)
                    {
                        styles.push(style);
                    }
                }
            }

            self.tracker.skip_line();
        }

        // If no explicit format was provided but styles were parsed, use default format
        let format_to_return = if self.format.is_none() && !styles.is_empty() {
            Some(vec![
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
            ])
        } else {
            self.format
        };

        Ok((
            Section::Styles(styles),
            format_to_return,
            self.issues,
            self.tracker.offset(),
            self.tracker.line() as usize,
        ))
    }

    /// Parse format specification line
    fn parse_format_line(&mut self, line: &'a str) {
        if let Some(format_data) = line.strip_prefix("Format:") {
            let fields: Vec<&'a str> = format_data.split(',').map(str::trim).collect();
            self.format = Some(fields);
        }
    }

    /// Get current line from source
    pub(super) fn current_line(&self) -> &'a str {
        let remaining = self.tracker.remaining();
        let end = remaining.find('\n').unwrap_or(remaining.len());
        &remaining[..end]
    }

    /// Check if at start of next section
    fn at_next_section(&self) -> bool {
        self.tracker.remaining().trim_start().starts_with('[')
    }

    /// Skip whitespace and comment lines
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.tracker.skip_whitespace();

            let remaining = self.tracker.remaining();
            if remaining.is_empty() {
                break;
            }

            if remaining.starts_with(';') || remaining.starts_with('#') {
                self.tracker.skip_line();
                continue;
            }

            // Check for newlines in whitespace
            if remaining.starts_with('\n') {
                self.tracker.advance(1);
                continue;
            }

            break;
        }
    }
}
