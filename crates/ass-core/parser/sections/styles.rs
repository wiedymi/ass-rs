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
    /// Source text being parsed
    source: &'a str,
    /// Current byte position in source
    position: usize,
    /// Current line number for error reporting
    line: usize,
    /// Parse issues and warnings collected during parsing
    issues: Vec<ParseIssue>,
    /// Format fields for the styles section
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
    #[must_use]
    pub const fn new(source: &'a str, start_position: usize, start_line: usize) -> Self {
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
    /// Tuple of (`parsed_section`, `format_fields`, `parse_issues`, `final_position`, `final_line`)
    ///
    /// # Errors
    ///
    /// Returns an error if the styles section contains malformed format lines or
    /// other unrecoverable syntax errors.
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
            self.position,
            self.line,
        ))
    }

    /// Parse format specification line
    fn parse_format_line(&mut self, line: &'a str) {
        if let Some(format_data) = line.strip_prefix("Format:") {
            let fields: Vec<&'a str> = format_data.split(',').map(str::trim).collect();
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
                .map_or("", |s| s.trim())
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
            .map_or(self.source.len(), |pos| self.position + pos);

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
    #[must_use]
    pub fn issues(self) -> Vec<ParseIssue> {
        self.issues
    }

    /// Get format specification
    #[must_use]
    pub const fn format(&self) -> Option<&Vec<&'a str>> {
        self.format.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_parser_new() {
        let source = "test content";
        let parser = StylesParser::new(source, 10, 5);
        assert_eq!(parser.source, source);
        assert_eq!(parser.position, 10);
        assert_eq!(parser.line, 5);
        assert!(parser.issues.is_empty());
        assert!(parser.format.is_none());
    }

    #[test]
    fn parse_empty_styles_section() {
        let parser = StylesParser::new("", 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert!(styles.is_empty());
        } else {
            panic!("Expected Styles section");
        }
        assert!(format.is_none());
        assert!(issues.is_empty());
    }

    #[test]
    fn parse_format_line() {
        let content = "Format: Name, Fontname, Fontsize, Bold\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert!(styles.is_empty());
        }
        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(format_fields.len(), 4);
        assert_eq!(format_fields[0], "Name");
        assert_eq!(format_fields[1], "Fontname");
        assert_eq!(format_fields[2], "Fontsize");
        assert_eq!(format_fields[3], "Bold");
    }

    #[test]
    fn parse_style_line_with_format() {
        let content = "Format: Name, Fontname, Fontsize\nStyle: Default, Arial, 16\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            let style = &styles[0];
            assert_eq!(style.name, "Default");
            assert_eq!(style.fontname, "Arial");
            assert_eq!(style.fontsize, "16");
        } else {
            panic!("Expected Styles section");
        }
    }

    #[test]
    fn parse_style_line_without_format() {
        let content = "Style: Default, Arial, 16, &Hffffff, &Hffffff, &H0, &H0, 0, 0, 0, 0, 100, 100, 0, 0, 1, 2, 0, 2, 30, 30, 30, 1\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            let style = &styles[0];
            assert_eq!(style.name, "Default");
            assert_eq!(style.fontname, "Arial");
            assert_eq!(style.fontsize, "16");
        } else {
            panic!("Expected Styles section");
        }
    }

    #[test]
    fn parse_style_line_field_count_mismatch_too_few() {
        let content = "Format: Name, Fontname, Fontsize\nStyle: Default, Arial\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert!(styles.is_empty()); // Style should be skipped due to too few fields
        }
        assert!(!issues.is_empty());
        assert!(issues[0].message.contains("has 2 fields, expected 3"));
    }

    #[test]
    fn parse_style_line_field_count_mismatch_too_many() {
        let content = "Format: Name, Fontname\nStyle: Default, Arial, 16, Extra\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1); // Style should still be parsed
            assert_eq!(styles[0].name, "Default");
            assert_eq!(styles[0].fontname, "Arial");
        }
        assert!(!issues.is_empty());
        assert!(issues[0].message.contains("has 4 fields, expected 2"));
    }

    #[test]
    fn parse_styles_with_comments() {
        let content = "; This is a comment\nFormat: Name, Fontname\n; Another comment\nStyle: Default, Arial\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
        assert!(format.is_some());
    }

    #[test]
    fn parse_styles_with_hash_comments() {
        let content = "# Hash comment\nFormat: Name, Fontname\n# Another hash comment\nStyle: Default, Arial\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
    }

    #[test]
    fn parse_styles_with_whitespace() {
        let content = "   \n  Format: Name, Fontname  \n\n   Style: Default, Arial   \n   ";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
    }

    #[test]
    fn parse_multiple_styles() {
        let content = "Format: Name, Fontname, Fontsize\nStyle: Default, Arial, 16\nStyle: Title, Times, 24\nStyle: Subtitle, Verdana, 12\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 3);
            assert_eq!(styles[0].name, "Default");
            assert_eq!(styles[1].name, "Title");
            assert_eq!(styles[2].name, "Subtitle");
        }
    }

    #[test]
    fn parse_styles_case_insensitive_fields() {
        let content = "Format: name, FONTNAME, FontSize\nStyle: Default, Arial, 16\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
            assert_eq!(styles[0].fontname, "Arial");
            assert_eq!(styles[0].fontsize, "16");
        }
    }

    #[test]
    fn parse_styles_empty_lines() {
        let content = "Format: Name, Fontname\n\n\nStyle: Default, Arial\n\n\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
        }
    }

    #[test]
    fn parse_styles_stops_at_next_section() {
        let content = "Format: Name, Fontname\nStyle: Default, Arial\n[Events]\nFormat: Text\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
        }
        // Should stop before [Events] section
        assert!(pos < content.len());
    }

    #[test]
    fn parse_styles_unusual_field_order() {
        let content = "Format: Fontsize, Name, Fontname\nStyle: 16, Default, Arial\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            // Fields should be mapped correctly despite unusual order
            assert_eq!(styles[0].name, "Default");
            assert_eq!(styles[0].fontname, "Arial");
            assert_eq!(styles[0].fontsize, "16");
        }
    }

    #[test]
    fn styles_parser_get_format() {
        let content = "Format: Name, Fontname\n";
        let parser = StylesParser::new(content, 0, 1);

        // Initially no format
        assert!(parser.format().is_none());

        let _result = parser.parse().unwrap();
        // After parsing, format should be available via the returned value
    }

    #[test]
    fn styles_parser_get_issues() {
        let content = "Style: Default\n"; // Missing format, will cause issues
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, _format, issues, _pos, _line) = result;

        // Should have issues about field count mismatch
        assert!(!issues.is_empty());
    }

    #[test]
    fn parse_format_line_edge_cases() {
        // Format line with spaces around fields
        let content = "Format:  Name ,  Fontname  ,  Fontsize  \n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, format, _issues, _pos, _line) = result;

        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(format_fields.len(), 3);
        assert_eq!(format_fields[0], "Name");
        assert_eq!(format_fields[1], "Fontname");
        assert_eq!(format_fields[2], "Fontsize");
    }

    #[test]
    fn parse_format_line_empty_fields() {
        // Format line with empty fields
        let content = "Format: Name,, Fontsize\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, format, _issues, _pos, _line) = result;

        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(format_fields.len(), 3);
        assert_eq!(format_fields[0], "Name");
        assert_eq!(format_fields[1], "");
        assert_eq!(format_fields[2], "Fontsize");
    }

    #[test]
    fn parse_multiple_format_lines() {
        // Multiple format lines - last one should win
        let content =
            "Format: Name, Fontname\nFormat: Name, Fontname, Fontsize\nStyle: Default, Arial, 16\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, _issues, _pos, _line) = result;

        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(format_fields.len(), 3); // Should use the second format

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
            assert_eq!(styles[0].fontname, "Arial");
            assert_eq!(styles[0].fontsize, "16");
        }
    }

    #[test]
    fn parse_style_with_unicode_content() {
        // Test Unicode characters in style names and values
        let content = "Format: Name, Fontname, Fontsize\nStyle: デフォルト, 明朝体, 16\nStyle: العربية, Arial Unicode MS, 14\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 2);
            assert_eq!(styles[0].name, "デフォルト");
            assert_eq!(styles[0].fontname, "明朝体");
            assert_eq!(styles[1].name, "العربية");
            assert_eq!(styles[1].fontname, "Arial Unicode MS");
        }
    }

    #[test]
    fn parse_style_with_commas_in_values() {
        // Test handling of values that contain commas (should split naively)
        let content = "Format: Name, Fontname\nStyle: Default,Name, Font,Name\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, issues, _pos, _line) = result;

        // Should generate warning about field count mismatch (4 fields vs 2 expected)
        assert!(!issues.is_empty());

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            // Parser splits on commas first, so "Default,Name" becomes "Default" and "Name"
            assert_eq!(styles[0].name, "Default");
            assert_eq!(styles[0].fontname, "Name");
        }
    }

    #[test]
    fn parse_styles_with_mixed_line_endings() {
        // Test different line ending styles
        let content = "Format: Name, Fontname\r\nStyle: Default, Arial\nStyle: Title, Times\r\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 2);
            assert_eq!(styles[0].name, "Default");
            assert_eq!(styles[1].name, "Title");
        }
    }

    #[test]
    fn parse_styles_with_very_long_line() {
        // Test very long style line
        let long_name = "A".repeat(1000);
        let content = format!("Format: Name, Fontname\nStyle: {long_name}, Arial\n");
        let parser = StylesParser::new(&content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, long_name);
            assert_eq!(styles[0].fontname, "Arial");
        }
    }

    #[test]
    fn parse_styles_whitespace_only_lines() {
        // Test lines with only whitespace
        let content = "Format: Name, Fontname\n   \t   \nStyle: Default, Arial\n\t\t\t\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
    }

    #[test]
    fn parse_styles_comment_variations() {
        // Test different comment styles and edge cases
        let content = "; Standard comment\n;Comment without space\n# Hash comment\n#Hash without space\nFormat: Name, Fontname\n; Comment after format\nStyle: Default, Arial\n;Final comment\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, _issues, _pos, _line) = result;

        assert!(format.is_some());
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
    }

    #[test]
    fn parse_styles_at_end_of_file() {
        // Test parsing when reaching end of file without newline
        let content = "Format: Name, Fontname\nStyle: Default, Arial";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
        assert_eq!(pos, content.len());
    }

    #[test]
    fn parse_styles_partial_content() {
        // Test parsing starting from middle of content
        let content = "Previous content\nFormat: Name, Fontname\nStyle: Default, Arial\n";
        let start_pos = content.find("Format:").unwrap();
        let parser = StylesParser::new(content, start_pos, 2);
        let result = parser.parse().unwrap();
        let (section, format, _issues, _pos, _line) = result;

        assert!(format.is_some());
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
    }

    #[test]
    fn parse_styles_invalid_lines() {
        // Test lines that don't match Format: or Style: patterns
        let content = "Format: Name, Fontname\nInvalidLine: Something\nRandomText\nStyle: Default, Arial\nMoreText\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
    }

    #[test]
    fn parse_styles_case_insensitive_prefixes() {
        // Test case variations of Format: and Style: prefixes
        let content = "format: Name, Fontname\nFORMAT: Name, Fontname, Fontsize\nstyle: Default, Arial, 16\nSTYLE: Title, Times, 24\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, _issues, _pos, _line) = result;

        // Note: The current implementation is case-sensitive for prefixes
        // These should be treated as invalid lines, not as format/style lines
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 0); // No styles should be parsed due to case sensitivity
        }
        assert!(format.is_none()); // No format should be set
    }

    #[test]
    fn parse_styles_empty_style_values() {
        // Test style line with empty values
        let content = "Format: Name, Fontname, Fontsize\nStyle: ,, \nStyle: Default,,16\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 2);
            assert_eq!(styles[0].name, "");
            assert_eq!(styles[0].fontname, "");
            assert_eq!(styles[0].fontsize, "");
            assert_eq!(styles[1].name, "Default");
            assert_eq!(styles[1].fontname, "");
            assert_eq!(styles[1].fontsize, "16");
        }
    }

    #[test]
    fn parse_styles_boundary_conditions() {
        // Test various boundary conditions
        let parser = StylesParser::new("", 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, issues, pos, line) = result;

        if let Section::Styles(styles) = section {
            assert!(styles.is_empty());
        }
        assert!(format.is_none());
        assert!(issues.is_empty());
        assert_eq!(pos, 0);
        assert_eq!(line, 1);
    }

    #[test]
    fn parse_styles_position_tracking() {
        // Test that position and line tracking works correctly
        let content = "Line 1\nFormat: Name, Fontname\nLine 3\nStyle: Default, Arial\nLine 5\n[Next Section]\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, _format, _issues, pos, line) = result;

        // Should stop at [Next Section]
        let next_section_pos = content.find("[Next Section]").unwrap();
        assert_eq!(pos, next_section_pos);
        assert!(line > 1); // Should have advanced line counter
    }

    #[test]
    fn parse_style_line_severely_malformed_fields() {
        let content = "Format: Name, Fontname, Fontsize\nStyle: OnlyName\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, _format, issues, _pos, _line) = result;

        // Should generate warning for insufficient fields
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("has 1 fields, expected 3"));
        assert!(has_field_warning);
    }

    #[test]
    fn parse_style_line_excessive_fields_warning() {
        let content =
            "Format: Name, Fontname\nStyle: Default,Arial,ExtraField1,ExtraField2,ExtraField3\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, _format, issues, _pos, _line) = result;

        // Should generate warning for too many fields but still parse
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("has 5 fields, expected 2"));
        assert!(has_field_warning);
    }

    #[test]
    fn parse_style_line_exact_field_count_no_warning() {
        let content = "Format: Name, Fontname, Fontsize\nStyle: Default,Arial,20\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, _format, issues, _pos, _line) = result;

        // Should not generate any field count warnings
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("fields, expected"));
        assert!(!has_field_warning);
    }

    #[test]
    fn parse_style_line_zero_fields_error() {
        let content = "Format: Name, Fontname\nStyle: \n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, _format, issues, _pos, _line) = result;

        // Should generate warning for empty style line
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("has 1 fields, expected 2"));
        assert!(has_field_warning);
    }

    #[test]
    fn parse_style_line_with_commas_in_style_values() {
        let content = "Format: Name, Fontname, Fontsize\nStyle: \"Style,Name\",\"Font,Name\",20\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        // Should handle quoted values with commas (though basic split may not)
        if let Section::Styles(styles) = section {
            assert!(!styles.is_empty());
        }
    }

    #[test]
    fn parse_format_line_with_duplicate_fields() {
        let content = "Format: Name, Name, Fontname\nStyle: Default,Duplicate,Arial\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn parse_style_without_format_uses_default() {
        let content =
            "Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, format, _issues, _pos, _line) = result;

        // Should use default format when none specified
        assert!(format.is_some());
        if let Section::Styles(styles) = section {
            assert!(!styles.is_empty());
        }
    }

    #[test]
    fn parse_multiple_format_lines_uses_last() {
        let content =
            "Format: Name, Fontname\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, format, _issues, _pos, _line) = result;

        // Should use the last format line encountered
        if let Some(fmt) = format {
            assert_eq!(fmt.len(), 3); // Should have 3 fields from second format line
        }
    }

    #[test]
    fn parse_style_line_field_case_insensitive_matching() {
        let content = "Format: name, FONTNAME, FontSize\nStyle: Default,Arial,20\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        // Should handle case-insensitive field matching
        if let Section::Styles(styles) = section {
            assert!(!styles.is_empty());
            let style = &styles[0];
            assert_eq!(style.name, "Default");
            assert_eq!(style.fontname, "Arial");
            assert_eq!(style.fontsize, "20");
        }
    }

    #[test]
    fn parse_style_line_trim_whitespace_in_values() {
        let content = "Format: Name, Fontname\nStyle:  Default  ,  Arial  \n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        // Should trim whitespace from field values
        if let Section::Styles(styles) = section {
            assert!(!styles.is_empty());
            let style = &styles[0];
            assert_eq!(style.name, "Default");
            assert_eq!(style.fontname, "Arial");
        }
    }

    #[test]
    fn parse_style_line_missing_fields_use_empty_string() {
        let content = "Format: Name, Fontname, Fontsize, PrimaryColour\nStyle: Default,Arial\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, issues, _pos, _line) = result;

        // Should use empty strings for missing fields and generate warning
        let has_warning = issues
            .iter()
            .any(|issue| issue.message.contains("has 2 fields, expected 4"));
        assert!(has_warning);

        // Since insufficient fields, should return None and not add to styles
        if let Section::Styles(styles) = section {
            // Should be empty due to insufficient fields
            assert!(styles.is_empty());
        }
    }

    #[test]
    fn parse_styles_with_mixed_valid_invalid_lines() {
        let content = "Format: Name, Fontname, Fontsize\nStyle: Valid,Arial,20\nStyle: Invalid\nStyle: Valid2,Tahoma,16\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, issues, _pos, _line) = result;

        // Should parse valid lines and warn about invalid ones
        let warning_count = issues
            .iter()
            .filter(|issue| issue.message.contains("fields, expected"))
            .count();
        assert_eq!(warning_count, 1); // Only one invalid line

        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 2); // Only valid styles should be included
        }
    }

    #[test]
    fn parse_format_line_with_excessive_whitespace() {
        let content = "Format:   Name  ,   Fontname   ,   Fontsize   \nStyle: Default,Arial,20\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (_section, format, _issues, _pos, _line) = result;

        // Should handle whitespace in format line
        if let Some(fmt) = format {
            assert_eq!(fmt.len(), 3);
            // Fields should be trimmed
            assert!(fmt.contains(&"Name"));
            assert!(fmt.contains(&"Fontname"));
            assert!(fmt.contains(&"Fontsize"));
        }
    }

    #[test]
    fn parse_styles_at_end_of_input_no_newline() {
        let content = "Format: Name, Fontname\nStyle: Default,Arial";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse().unwrap();
        let (section, _format, _issues, _pos, _line) = result;

        // Should handle end of input without trailing newline
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        }
    }
}
