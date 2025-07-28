//! Styles section parser for ASS scripts.
//!
//! Handles parsing of the [V4+ Styles] section which contains style definitions
//! with format specifications and style entries.

use crate::parser::{
    ast::{Section, Span, Style},
    errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue},
    position_tracker::PositionTracker,
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
    /// Position tracker for accurate span generation
    tracker: PositionTracker<'a>,
    /// Parse issues and warnings collected during parsing
    issues: Vec<ParseIssue>,
    /// Format fields for the styles section
    format: Option<Vec<&'a str>>,
}

impl<'a> StylesParser<'a> {
    /// Parse a single style line
    ///
    /// Parses a single style definition line using the provided format specification.
    /// This method is exposed for incremental parsing support.
    ///
    /// # Arguments
    ///
    /// * `line` - The style line to parse (without "Style:" prefix)
    /// * `format` - The format fields from the Format line
    /// * `line_number` - The line number for error reporting
    ///
    /// # Returns
    ///
    /// Parsed Style or error if the line is malformed
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::InsufficientFields`] if the line has fewer fields than expected by format
    pub fn parse_style_line(
        line: &'a str,
        format: &[&'a str],
        line_number: u32,
    ) -> core::result::Result<Style<'a>, ParseError> {
        // First check if this is an inheritance style
        let (adjusted_line, parent_style) = if line.trim_start().starts_with('*') {
            // Find the first comma after the asterisk to extract parent style
            line.find(',').map_or((line, None), |first_comma| {
                let parent_part = &line[0..first_comma];
                let parent_name = parent_part.trim_start().trim_start_matches('*').trim();
                let remaining = &line[first_comma + 1..];
                (remaining, Some(parent_name))
            })
        } else {
            (line, None)
        };

        let parts: Vec<&str> = adjusted_line.split(',').collect();

        let format = if format.is_empty() {
            &[
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
            ]
        } else {
            format
        };

        if parts.len() < format.len() {
            return Err(ParseError::InsufficientFields {
                expected: format.len(),
                found: parts.len(),
                line: line_number as usize,
            });
        }

        let get_field = |name: &str| -> &'a str {
            format
                .iter()
                .position(|&field| field.eq_ignore_ascii_case(name))
                .and_then(|idx| parts.get(idx))
                .map_or("", |s| s.trim())
        };

        // Create span for the style (caller will need to adjust this)
        let span = Span::new(0, 0, line_number, 1);

        Ok(Style {
            name: get_field("Name"),
            parent: parent_style,
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
            margin_t: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginT"))
                .then(|| get_field("MarginT")),
            margin_b: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginB"))
                .then(|| get_field("MarginB")),
            encoding: get_field("Encoding"),
            relative_to: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("RelativeTo"))
                .then(|| get_field("RelativeTo")),
            span,
        })
    }
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

    /// Parse single style definition line
    fn parse_style_line_internal(
        &mut self,
        line: &'a str,
        line_start: &PositionTracker<'a>,
    ) -> Option<Style<'a>> {
        // First check if this is an inheritance style
        let (adjusted_line, parent_style) = if line.trim_start().starts_with('*') {
            // Find the first comma after the asterisk to extract parent style
            line.find(',').map_or((line, None), |first_comma| {
                let parent_part = &line[0..first_comma];
                let parent_name = parent_part.trim_start().trim_start_matches('*').trim();
                let remaining = &line[first_comma + 1..];
                (remaining, Some(parent_name))
            })
        } else {
            (line, None)
        };

        let parts: Vec<&str> = adjusted_line.split(',').collect();

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
                line_start.line() as usize,
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

        // Calculate span for this style line
        let full_line = self.current_line();
        let span = line_start.span_for(full_line.len());

        Some(Style {
            name: get_field("Name"),
            parent: parent_style,
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
            margin_t: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginT"))
                .then(|| get_field("MarginT")),
            margin_b: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginB"))
                .then(|| get_field("MarginB")),
            encoding: get_field("Encoding"),
            relative_to: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("RelativeTo"))
                .then(|| get_field("RelativeTo")),
            span,
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Section;

    #[test]
    fn parse_empty_section() {
        let parser = StylesParser::new("", 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());
        let (section, ..) = result.unwrap();
        if let Section::Styles(styles) = section {
            assert!(styles.is_empty());
        } else {
            panic!("Expected Styles section");
        }
    }

    #[test]
    fn parse_basic_style() {
        let content = "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            let style = &styles[0];
            assert_eq!(style.name, "Default");
            assert_eq!(style.fontname, "Arial");
            assert_eq!(style.fontsize, "20");
            // Check span
            assert!(style.span.start > 0);
            assert!(style.span.end > style.span.start);
        } else {
            panic!("Expected Styles section");
        }
    }

    #[test]
    fn parse_without_format_line() {
        let content = "Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "Default");
        } else {
            panic!("Expected Styles section");
        }
    }

    #[test]
    fn parse_with_inheritance() {
        let content = "Style: *Default,NewStyle,Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
        let parser = StylesParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].name, "NewStyle");
            assert_eq!(styles[0].parent, Some("Default"));
        } else {
            panic!("Expected Styles section");
        }
    }

    #[test]
    fn parse_with_position_tracking() {
        // Create a larger content that simulates a full file
        let prefix = "a".repeat(100); // 100 bytes of padding
        let section_content = "Style: Test,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
        let full_content = format!("{prefix}{section_content}");

        // Parser starts at position 100
        let parser = StylesParser::new(&full_content, 100, 10);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _, _, final_pos, final_line) = result.unwrap();
        if let Section::Styles(styles) = section {
            assert_eq!(styles.len(), 1);
            let style = &styles[0];
            assert_eq!(style.span.start, 100);
            assert_eq!(style.span.line, 10);
        } else {
            panic!("Expected Styles section");
        }

        assert_eq!(final_pos, 100 + section_content.len());
        assert_eq!(final_line, 11);
    }

    #[test]
    fn test_public_parse_style_line() {
        let format = vec![
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
        ];
        let line = "Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1";

        let result = StylesParser::parse_style_line(line, &format, 1);
        assert!(result.is_ok());

        let style = result.unwrap();
        assert_eq!(style.name, "Default");
        assert_eq!(style.fontname, "Arial");
        assert_eq!(style.fontsize, "20");
        assert!(style.parent.is_none());
    }

    #[test]
    fn test_parse_style_line_with_inheritance() {
        let format = vec![
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
        ];
        let line = "*Default,NewStyle,Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1";

        let result = StylesParser::parse_style_line(line, &format, 1);
        assert!(result.is_ok());

        let style = result.unwrap();
        assert_eq!(style.name, "NewStyle");
        assert_eq!(style.parent, Some("Default"));
        assert_eq!(style.fontsize, "24");
    }

    #[test]
    fn test_parse_style_line_insufficient_fields() {
        let format = vec![
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
        ];
        let line = "Default,Arial,20"; // Missing fields

        let result = StylesParser::parse_style_line(line, &format, 1);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, ParseError::InsufficientFields { .. }));
        }
    }

    #[test]
    fn test_parse_style_line_with_empty_format() {
        // Test with empty format array - should use default
        let format = vec![];
        let line = "Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1";

        let result = StylesParser::parse_style_line(line, &format, 1);
        assert!(result.is_ok());

        let style = result.unwrap();
        assert_eq!(style.name, "Default");
        assert_eq!(style.fontname, "Arial");
    }
}
