//! Script Info section parser for ASS scripts.
//!
//! Handles parsing of the [Script Info] section which contains metadata
//! and configuration parameters for the subtitle script.

use crate::{
    parser::{
        ast::{ScriptInfo, Section},
        errors::{IssueCategory, IssueSeverity, ParseIssue},
        sections::ScriptInfoParseResult,
        ParseResult,
    },
    ScriptVersion,
};
use alloc::vec::Vec;

/// Parser for [Script Info] section content
///
/// Parses key-value pairs from the script info section and handles
/// special fields like ScriptType that affect parsing behavior.
///
/// # Performance
///
/// - Time complexity: O(n) for n lines in section
/// - Memory: Zero allocations via lifetime-generic spans
/// - Target: <0.5ms for typical script info sections
pub struct ScriptInfoParser<'a> {
    source: &'a str,
    position: usize,
    line: usize,
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
    pub fn new(source: &'a str, start_position: usize, start_line: usize) -> Self {
        Self {
            source,
            position: start_position,
            line: start_line,
            issues: Vec::new(),
        }
    }

    /// Parse script info section content
    ///
    /// Returns the parsed section and any issues encountered during parsing.
    /// Handles ScriptType field detection and version updating.
    ///
    /// # Returns
    ///
    /// Tuple of (parsed_section, detected_version, parse_issues, final_position, final_line)
    pub fn parse(mut self) -> ParseResult<ScriptInfoParseResult<'a>> {
        let mut fields = Vec::new();
        let mut detected_version = None;

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
                    self.line,
                ));
            }

            self.skip_line();
        }

        let section = Section::ScriptInfo(ScriptInfo { fields });
        Ok((
            section,
            detected_version,
            self.issues,
            self.position,
            self.line,
        ))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_section() {
        let parser = ScriptInfoParser::new("", 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, version, ..) = result.unwrap();
        if let Section::ScriptInfo(info) = section {
            assert!(info.fields.is_empty());
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

        let (section, ..) = result.unwrap();
        if let Section::ScriptInfo(info) = section {
            assert_eq!(info.fields.len(), 2);
            assert_eq!(info.get_field("Title"), Some("Test"));
            assert_eq!(info.get_field("Author"), Some("Someone"));
        } else {
            panic!("Expected ScriptInfo section");
        }
    }
}
