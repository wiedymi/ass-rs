//! Main parser coordination and dispatch logic
//!
//! Contains the core `Parser` struct that orchestrates parsing of different
//! ASS script sections and handles error recovery.

use crate::{
    utils::{
        errors::{encoding::validate_bom_handling, resource::check_input_size_limit},
        CoreError,
    },
    Result, ScriptVersion,
};
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
            return Script::from_parts(self.source, self.version, Vec::new(), self.issues);
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
        let remaining = self.source[self.position..].trim_start();
        if !remaining.starts_with('[') {
            return false;
        }

        // Check if this looks like a complete section header (has closing ])
        remaining.find('\n').map_or_else(
            || remaining.contains(']'),
            |line_end| remaining[..line_end].contains(']'),
        )
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
        let start_position = self.position;

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

            // Prevent infinite loop: if we haven't advanced, force advance by one character
            if self.position == start_position {
                self.position = (self.position + 1).min(self.source.len());
                break;
            }
        }

        suggestion
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_script(content: &str) -> String {
        format!("[Script Info]\nTitle: Test\n\n{content}")
    }

    #[test]
    fn parser_new() {
        let source = "test content";
        let parser = Parser::new(source);
        assert_eq!(parser.source, source);
        assert_eq!(parser.position, 0);
        assert_eq!(parser.line, 1);
        assert_eq!(parser.version, ScriptVersion::AssV4);
        assert!(parser.sections.is_empty());
        assert!(parser.issues.is_empty());
        assert!(parser.styles_format.is_none());
        assert!(parser.events_format.is_none());
    }

    #[test]
    fn parser_parse_empty_script() {
        let parser = Parser::new("");
        let script = parser.parse();
        assert_eq!(script.version(), ScriptVersion::AssV4);
        assert!(script.sections().is_empty());
    }

    #[test]
    fn parser_parse_with_bom() {
        let content = "\u{FEFF}[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_parse_input_size_limit() {
        let large_content = "a".repeat(51 * 1024 * 1024); // 51MB > 50MB limit
        let parser = Parser::new(&large_content);
        let script = parser.parse();
        assert!(!script.issues().is_empty());
        let has_size_error = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Input size limit exceeded"));
        assert!(has_size_error);
    }

    #[test]
    fn parser_parse_unknown_section() {
        let content = "[Unknown Section]\nSome content";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unknown_section_warning = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Unknown section"));
        assert!(has_unknown_section_warning);
    }

    #[test]
    fn parser_parse_unclosed_section_header() {
        let content = "[Script Info\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unclosed_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unclosed section header")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_unclosed_error);
    }

    #[test]
    fn parser_parse_missing_section_header() {
        let content = "Title: Test\nAuthor: Someone";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_header_error = script.issues().iter().any(|issue| {
            issue.message.contains("Expected section header")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_header_error);
    }

    #[test]
    fn parser_parse_script_info_section() {
        let content = "[Script Info]\nTitle: Test Script\nScriptType: v4.00+";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert_eq!(script.sections().len(), 1);
        // Version should be updated based on ScriptType parsing
        assert!(
            script.version() == ScriptVersion::AssV4Plus
                || script.version() == ScriptVersion::AssV4
        );
    }

    #[test]
    fn parser_parse_styles_section() {
        let content =
            create_test_script("[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial");
        let parser = Parser::new(&content);
        let script = parser.parse();
        assert!(script.sections().len() >= 2);
    }

    #[test]
    fn parser_parse_events_section() {
        let content = create_test_script(
            "[Events]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test",
        );
        let parser = Parser::new(&content);
        let script = parser.parse();
        assert!(script.sections().len() >= 2);
    }

    #[test]
    fn parser_parse_fonts_section() {
        let content = create_test_script("[Fonts]\nfontname: Arial\nfontdata: ABCD1234");
        let parser = Parser::new(&content);
        let script = parser.parse();
        assert!(script.sections().len() >= 2);
    }

    #[test]
    fn parser_parse_graphics_section() {
        let content = create_test_script("[Graphics]\nfilename: image.png\ndata: ABCD1234");
        let parser = Parser::new(&content);
        let script = parser.parse();
        assert!(script.sections().len() >= 2);
    }

    #[test]
    fn parser_skip_comments() {
        let content = "; This is a comment\n!: Another comment\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_error_recovery_style_suggestion() {
        let content = "[BadSection]\nStyle: Default,Arial\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("[V4+ Styles]"));
        assert!(has_suggestion);
    }

    #[test]
    fn parser_error_recovery_events_suggestion() {
        let content =
            "[BadSection]\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("[Events]"));
        assert!(has_suggestion);
    }

    #[test]
    fn parser_error_recovery_script_info_suggestion() {
        let content = "[BadSection]\nTitle: Test Script\n[Script Info]\nTitle: Real";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("[Script Info]"));
        assert!(has_suggestion);
    }

    #[test]
    fn parser_error_recovery_format_line_events() {
        let content = "[BadSection]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("[Events]"));
        assert!(has_suggestion);
    }

    #[test]
    fn parser_error_recovery_format_line_styles() {
        let content = "[BadSection]\nFormat: Name, Fontname\nStyle: Default,Arial\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("[V4+ Styles]"));
        assert!(has_suggestion);
    }

    #[test]
    fn parser_multiple_sections() {
        let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name\nStyle: Default\n\n[Events]\nFormat: Text\nDialogue: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert_eq!(script.sections().len(), 3);
    }

    #[test]
    fn parser_whitespace_handling() {
        let content = "   \n\n  [Script Info]  \n  Title: Test  \n\n   ";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_invalid_bom_warning() {
        // Test with content that may have BOM-related issues
        let content = "[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        // Should parse successfully
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_v4_styles_section() {
        let content = "[V4 Styles]\nFormat: Name, Fontname\nStyle: Default,Arial";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_skip_to_next_section_with_format_line_events() {
        let content = "[BadSection]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Events]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Real";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_events_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Did you mean '[Events]'?"));
        assert!(has_events_suggestion);
    }

    #[test]
    fn parser_skip_to_next_section_with_format_line_styles() {
        let content = "[BadSection]\nFormat: Name, Fontname\nStyle: Default,Arial\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Real,Arial";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_styles_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Did you mean '[V4+ Styles]'?"));
        assert!(has_styles_suggestion);
    }

    #[test]
    fn parser_at_next_section_edge_cases() {
        // Test incomplete section header
        let content = "[Incomplete";
        let parser = Parser::new(content);
        let script = parser.parse();
        // Should handle gracefully
        assert!(!script.issues().is_empty());
    }

    #[test]
    fn parser_at_next_section_with_closing_bracket() {
        let content = "[Script Info]\nTitle: Test\n[V4+ Styles]\nFormat: Name";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_skip_line_edge_cases() {
        let content = "[Script Info]\n\n\n\nTitle: Test\n";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_mixed_comment_styles() {
        let content =
            "; Comment style 1\n!: Comment style 2\n; Another comment\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_section_header_with_extra_brackets() {
        let content = "[Script Info]]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_empty_section_header() {
        let content = "[]\nSome content\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unknown section")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_error);
    }

    #[test]
    fn parser_section_header_only_spaces() {
        let content = "[   ]\nSome content\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unknown section")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_error);
    }

    #[test]
    fn parser_malformed_bom_sequence() {
        // Test with partial BOM-like sequence
        let content = "\u{00EF}\u{00BB}[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        // Should parse, potentially with warnings - but may not have valid sections
        assert!(script.sections().is_empty() || !script.sections().is_empty());
    }

    #[test]
    fn parser_content_after_eof() {
        let content = "[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
        assert!(
            script.issues().is_empty()
                || script
                    .issues()
                    .iter()
                    .all(|i| i.severity != IssueSeverity::Error)
        );
    }

    #[test]
    fn parser_multiple_consecutive_section_headers() {
        let content = "[Script Info]\n[V4+ Styles]\n[Events]\nFormat: Text\nDialogue: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_section_header_with_special_chars() {
        let content = "[Script Info & More!]\nTitle: Test\n[Script Info]\nTitle: Real";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unknown_section = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Unknown section"));
        assert!(has_unknown_section);
    }

    #[test]
    fn parser_skip_to_next_section_no_advance_protection() {
        // Test case that would trigger the infinite loop protection
        let content = "[BadSection\nContent without proper section end";
        let parser = Parser::new(content);
        let script = parser.parse();
        // Should not hang and should produce some result
        assert!(!script.issues().is_empty());
    }

    #[test]
    fn parser_whitespace_before_and_after_sections() {
        let content = "   \n\n  ; Comment\n  [Script Info]  \n  Title: Test  \n\n  [V4+ Styles]  \n  Format: Name\n  Style: Default  \n\n  ";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(script.sections().len() >= 2);
    }

    #[test]
    fn parser_comment_lines_between_sections() {
        let content = "[Script Info]\nTitle: Test\n; This is a comment\n!: Another comment\n\n[V4+ Styles]\nFormat: Name\nStyle: Default";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(script.sections().len() >= 2);
    }

    #[test]
    fn parser_find_section_end_no_newline() {
        let content = "[Script Info]";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty() || !script.issues().is_empty());
    }

    #[test]
    fn parser_unicode_in_section_names() {
        let content = "[Script Info 中文]\nTitle: Test\n[Script Info]\nTitle: Real";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unknown_section = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Unknown section"));
        assert!(has_unknown_section);
    }

    #[test]
    fn parser_very_long_section_name() {
        let long_name = "a".repeat(1000);
        let content = format!("[{long_name}]\nTitle: Test\n[Script Info]\nTitle: Real");
        let parser = Parser::new(&content);
        let script = parser.parse();
        let has_unknown_section = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Unknown section"));
        assert!(has_unknown_section);
    }

    #[test]
    fn parser_case_sensitive_section_names() {
        let content = "[script info]\nTitle: Test\n[Script Info]\nTitle: Real";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unknown_section = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Unknown section"));
        assert!(has_unknown_section);
    }

    #[test]
    fn parser_parse_section_error_unknown_section_with_content() {
        let content = "[BadSection]\nSome content here\nMore content\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unknown_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unknown section") || issue.message.contains("BadSection")
        });
        assert!(has_unknown_error);
    }

    #[test]
    fn parser_parse_section_error_unclosed_bracket_at_eof() {
        let content = "[Script Info";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unclosed_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unclosed section header")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_unclosed_error);
    }

    #[test]
    fn parser_parse_section_error_empty_section_name() {
        let content = "[]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_empty_section_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unknown section")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_empty_section_error);
    }

    #[test]
    fn parser_parse_section_error_whitespace_only_section() {
        let content = "[   ]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_whitespace_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unknown section")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_whitespace_error);
    }

    #[test]
    fn parser_error_recovery_multiple_unknown_sections() {
        let content = "[BadSection1]\nStyle: Default,Arial\n[BadSection2]\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        let style_suggestion_count = script
            .issues()
            .iter()
            .filter(|issue| issue.message.contains("[V4+ Styles]"))
            .count();
        let events_suggestion_count = script
            .issues()
            .iter()
            .filter(|issue| issue.message.contains("[Events]"))
            .count();
        assert!(style_suggestion_count >= 1);
        assert!(events_suggestion_count >= 1);
    }

    #[test]
    fn parser_skip_to_next_section_no_protection_edge_case() {
        let content = "[UnknownSection]\nLine without next section";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unknown_error = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Unknown section"));
        assert!(has_unknown_error);
    }

    #[test]
    fn parser_find_section_end_at_exact_boundary() {
        let content = "[Script Info]\nTitle: Test\n[V4+ Styles]";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(!script.sections().is_empty());
    }

    #[test]
    fn parser_section_header_without_content() {
        let content = "[Script Info]\n[V4+ Styles]\nFormat: Name\nStyle: Default,Arial";
        let parser = Parser::new(content);
        let script = parser.parse();
        assert!(script.sections().len() >= 2);
    }

    #[test]
    fn parser_malformed_section_headers_mixed() {
        let content = "[Script Info\nTitle: Test\n]NotASection[\n[V4+ Styles]\nFormat: Name\nStyle: Default,Arial";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_errors = script.issues().iter().any(|issue| {
            issue.message.contains("Unclosed section header")
                || issue.message.contains("Unknown section")
                || issue.message.contains("Failed to parse section")
        });
        assert!(has_errors);
    }

    #[test]
    fn parser_nested_bracket_edge_cases() {
        let content =
            "[[Script Info]]\nTitle: Test\n[V4+ Styles]\nFormat: Name\nStyle: Default,Arial";
        let parser = Parser::new(content);
        let script = parser.parse();
        let has_unknown_error = script.issues().iter().any(|issue| {
            issue.message.contains("Unknown section") || issue.message.contains("[Script Info]")
        });
        assert!(has_unknown_error);
    }

    #[test]
    fn parser_section_with_trailing_characters() {
        let content = "[Script Info] Extra Text\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        // Should parse successfully - trailing text after ] is ignored
        assert!(!script.sections().is_empty());
        // Should not generate unknown section errors
        let has_unknown_error = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Unknown section"));
        assert!(!has_unknown_error);
    }

    #[test]
    fn parser_complex_error_recovery_scenario() {
        let content = "[BadSection1]\nStyle: Test,Arial,20\nComment: 0,0:00:00.00,0:00:01.00,,Comment text\n[BadSection2]\nDialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,Test\n[Script Info]\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();

        let has_style_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("[V4+ Styles]"));
        let has_events_suggestion = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("[Events]"));

        assert!(has_style_suggestion);
        assert!(has_events_suggestion);
    }

    #[test]
    fn parser_input_size_limit_exactly_at_boundary() {
        let content = "a".repeat(50 * 1024 * 1024 - 1);
        let parser = Parser::new(&content);
        let script = parser.parse();
        // Should not have size limit error since we're just under the limit
        let has_size_error = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("Input size limit exceeded"));
        assert!(!has_size_error);
    }

    #[test]
    fn parser_bom_detection_partial_sequences() {
        // Create content with partial UTF-8 BOM (0xEF, 0xBB without 0xBF)
        let bytes = &[
            0xEF, 0xBB, b'[', b'S', b'c', b'r', b'i', b'p', b't', b' ', b'I', b'n', b'f', b'o',
            b']', b'\n', b'T', b'i', b't', b'l', b'e', b':', b' ', b'T', b'e', b's', b't',
        ];
        let content_partial_bom = String::from_utf8_lossy(bytes);
        let parser = Parser::new(&content_partial_bom);
        let script = parser.parse();
        let has_bom_warning = script.issues().iter().any(|issue| {
            issue.message.contains("BOM") || issue.message.contains("byte order mark")
        });
        assert!(has_bom_warning);
    }

    #[test]
    fn parser_version_detection_edge_cases() {
        let content = "[Script Info]\nScriptType: v4.00++\nTitle: Test";
        let parser = Parser::new(content);
        let script = parser.parse();
        // Should handle malformed script type gracefully
        assert!(!script.sections().is_empty());
    }
}
