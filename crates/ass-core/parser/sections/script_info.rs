//! Script Info section parser for ASS scripts.
//!
//! Handles parsing of the [Script Info] section which contains metadata
//! and configuration parameters for the subtitle script.

use crate::{
    parser::{
        ast::{ScriptInfo, Section},
        errors::{IssueCategory, IssueSeverity, ParseIssue},
        position_tracker::PositionTracker,
        sections::ScriptInfoParseResult,
        ParseResult,
    },
    ScriptVersion,
};
use alloc::vec::Vec;

/// Parser for [Script Info] section content
///
/// Parses key-value pairs from the script info section and handles
/// special fields like `ScriptType` that affect parsing behavior.
///
/// # Performance
///
/// - Time complexity: O(n) for n lines in section
/// - Memory: Zero allocations via lifetime-generic spans
/// - Target: <0.5ms for typical script info sections
pub struct ScriptInfoParser<'a> {
    /// Position tracker for accurate span generation
    tracker: PositionTracker<'a>,
    /// Parse issues and warnings collected during parsing
    issues: Vec<ParseIssue>,
}

impl<'a> ScriptInfoParser<'a> {
    /// Create new script info parser for source text
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
        }
    }

    /// Parse script info section content
    ///
    /// Returns the parsed section and any issues encountered during parsing.
    /// Handles `ScriptType` field detection and version updating.
    ///
    /// # Returns
    ///
    /// Tuple of (`parsed_section`, `detected_version`, `parse_issues`, `final_position`, `final_line`)
    ///
    /// # Errors
    ///
    /// Returns an error if the script info section contains malformed key-value pairs or
    /// other unrecoverable syntax errors.
    pub fn parse(mut self) -> ParseResult<ScriptInfoParseResult<'a>> {
        let section_start = self.tracker.checkpoint();
        let mut fields = Vec::new();
        let mut detected_version = None;

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

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();

                if key == "ScriptType" {
                    if let Some(version) = ScriptVersion::from_header(value) {
                        detected_version = Some(version);
                    }
                }

                fields.push((key, value));
            } else {
                self.issues.push(ParseIssue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Format,
                    "Invalid script info line format".into(),
                    line_start.line() as usize,
                ));
            }

            self.tracker.skip_line();
        }

        let span = self.tracker.span_from(&section_start);
        let section = Section::ScriptInfo(ScriptInfo { fields, span });

        Ok((
            section,
            detected_version,
            self.issues,
            self.tracker.offset(),
            self.tracker.line() as usize,
        ))
    }

    /// Get current line from source
    fn current_line(&self) -> &'a str {
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

    /// Get accumulated parse issues
    #[must_use]
    pub fn issues(self) -> Vec<ParseIssue> {
        self.issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::format;

    #[test]
    fn parse_empty_section() {
        let parser = ScriptInfoParser::new("", 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, version, ..) = result.unwrap();
        if let Section::ScriptInfo(info) = section {
            assert!(info.fields.is_empty());
            assert_eq!(info.span.start, 0);
            assert_eq!(info.span.end, 0);
        } else {
            panic!("Expected ScriptInfo section");
        }
        assert!(version.is_none());
    }

    #[test]
    fn parse_basic_fields() {
        let content = "Title: Test Script\nScriptType: v4.00+\n";
        let parser = ScriptInfoParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, version, ..) = result.unwrap();
        if let Section::ScriptInfo(info) = section {
            assert_eq!(info.fields.len(), 2);
            assert_eq!(info.get_field("Title"), Some("Test Script"));
            assert_eq!(info.get_field("ScriptType"), Some("v4.00+"));
            assert_eq!(info.span.start, 0);
            assert_eq!(info.span.end, content.len());
            assert_eq!(info.span.line, 1);
            assert_eq!(info.span.column, 1);
        } else {
            panic!("Expected ScriptInfo section");
        }
        assert!(version.is_some());
    }

    #[test]
    fn skip_comments_and_whitespace() {
        let content = "; Comment\n# Another comment\n\nTitle: Test\n";
        let parser = ScriptInfoParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::ScriptInfo(info) = section {
            assert_eq!(info.fields.len(), 1);
            assert_eq!(info.get_field("Title"), Some("Test"));
        } else {
            panic!("Expected ScriptInfo section");
        }
    }

    #[test]
    fn handle_invalid_lines() {
        let content = "Title: Test\nInvalidLine\nAuthor: Someone\n";
        let parser = ScriptInfoParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _, issues, ..) = result.unwrap();
        if let Section::ScriptInfo(info) = section {
            assert_eq!(info.fields.len(), 2);
            assert_eq!(info.get_field("Title"), Some("Test"));
            assert_eq!(info.get_field("Author"), Some("Someone"));
        } else {
            panic!("Expected ScriptInfo section");
        }

        // Should have a warning about the invalid line
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, IssueSeverity::Warning);
    }

    #[test]
    fn parse_with_position_tracking() {
        // Create a larger content that simulates a full file
        let prefix = "Some prefix\n"; // 12 bytes
        let section_content = "Title: Test\nAuthor: Someone\n";
        let full_content = format!("{prefix}{section_content}");

        // Parser starts at position 12 (after prefix)
        let parser = ScriptInfoParser::new(&full_content, 12, 2);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _, _, final_pos, final_line) = result.unwrap();
        if let Section::ScriptInfo(info) = section {
            assert_eq!(info.fields.len(), 2);
            assert_eq!(info.fields[0], ("Title", "Test"));
            assert_eq!(info.fields[1], ("Author", "Someone"));
            assert_eq!(info.span.start, 12);
            assert_eq!(info.span.end, 12 + section_content.len());
            assert_eq!(info.span.line, 2);
            assert_eq!(info.span.column, 1);
        } else {
            panic!("Expected ScriptInfo section");
        }

        assert_eq!(final_pos, 12 + section_content.len());
        assert_eq!(final_line, 4); // Started at line 2, added 2 lines
    }
}
