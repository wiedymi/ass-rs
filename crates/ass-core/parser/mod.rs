//! ASS script parser module
//!
//! Provides zero-copy parsing of ASS subtitle scripts with lifetime-generic AST nodes.
//! Supports full ASS v4+, SSA v4 compatibility, and libass 0.17.4+ extensions.
//!
//! # Performance
//!
//! - Target: <5ms parsing for typical 1KB scripts
//! - Memory: ~1.1x input size via zero-copy spans
//! - Incremental updates: <2ms for single-event changes
//!
//! # Example
//!
//! ```rust
//! use ass_core::parser::Script;
//!
//! let script_text = r#"
//! [Script Info]
//! Title: Example
//! ScriptType: v4.00+
//!
//! [Events]
//! Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
//! Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
//! "#;
//!
//! let script = Script::parse(script_text)?;
//! assert_eq!(script.sections().len(), 2);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{utils::CoreError, Result, ScriptVersion};
use alloc::{format, string::ToString, vec::Vec};
#[cfg(feature = "stream")]
use core::ops::Range;

pub mod ast;
pub mod errors;

#[cfg(feature = "stream")]
pub mod streaming;

pub use ast::{Event, ScriptInfo, Section, Style};
pub use errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue};

/// Main ASS script container with zero-copy lifetime-generic design
///
/// Uses `&'a str` spans throughout the AST to avoid allocations during parsing.
/// Thread-safe via immutable design after construction.
#[derive(Debug, Clone, PartialEq)]
pub struct Script<'a> {
    /// Input source text for span validation
    source: &'a str,

    /// Script version detected from headers
    version: ScriptVersion,

    /// Parsed sections in document order
    sections: Vec<Section<'a>>,

    /// Parse warnings and recoverable errors
    issues: Vec<ParseIssue>,
}

impl<'a> Script<'a> {
    /// Parse ASS script from source text with zero-copy design
    ///
    /// Performs full validation and partial error recovery. Returns script
    /// even with errors - check `issues()` for problems.
    ///
    /// # Performance
    ///
    /// Target <5ms for 1KB typical scripts. Uses minimal allocations via
    /// zero-copy spans referencing input text.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::parser::Script;
    /// let script = Script::parse("[Script Info]\nTitle: Test")?;
    /// assert_eq!(script.version(), ass_core::ScriptVersion::AssV4);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(source: &'a str) -> Result<Self> {
        let parser = Parser::new(source);
        parser.parse()
    }

    /// Parse incrementally with range-based updates for editors
    ///
    /// Updates only the specified range, keeping other sections unchanged.
    /// Enables <2ms edit responsiveness for interactive editing.
    ///
    /// # Arguments
    ///
    /// * `range` - Byte range in source to re-parse
    /// * `new_text` - Replacement text for the range
    ///
    /// # Returns
    ///
    /// Delta containing changes that can be applied to existing script.
    #[cfg(feature = "stream")]
    pub fn parse_partial(&self, range: Range<usize>, new_text: &'a str) -> Result<ScriptDelta<'a>> {
        streaming::parse_incremental(self, range, new_text)
    }

    /// Get script version detected during parsing
    pub fn version(&self) -> ScriptVersion {
        self.version
    }

    /// Get all parsed sections in document order
    pub fn sections(&self) -> &[Section<'a>] {
        &self.sections
    }

    /// Get parse issues (warnings, recoverable errors)
    pub fn issues(&self) -> &[ParseIssue] {
        &self.issues
    }

    /// Get source text that spans reference
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// Find section by type
    pub fn find_section(&self, section_type: SectionType) -> Option<&Section<'a>> {
        self.sections
            .iter()
            .find(|s| s.section_type() == section_type)
    }

    /// Validate all spans reference source text correctly
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    #[cfg(debug_assertions)]
    pub fn validate_spans(&self) -> bool {
        let source_ptr = self.source.as_ptr();
        let source_range = source_ptr as usize..source_ptr as usize + self.source.len();

        self.sections
            .iter()
            .all(|section| section.validate_spans(&source_range))
    }
}

/// Section type discriminant for efficient lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionType {
    ScriptInfo,
    Styles,
    Events,
    Fonts,
    Graphics,
}

/// Internal parser state machine
struct Parser<'a> {
    source: &'a str,
    position: usize,
    line: usize,
    version: ScriptVersion,
    sections: Vec<Section<'a>>,
    issues: Vec<ParseIssue>,
}

impl<'a> Parser<'a> {
    /// Create new parser for source text
    fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            version: ScriptVersion::AssV4, // Default, updated when ScriptType found
            sections: Vec::new(),
            issues: Vec::new(),
        }
    }

    /// Parse complete script
    fn parse(mut self) -> Result<Script<'a>> {
        // Skip BOM if present
        if self.source.starts_with('\u{FEFF}') {
            self.position = 3;
        }

        // Parse all sections
        while self.position < self.source.len() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() {
                break;
            }

            match self.parse_section() {
                Ok(section) => self.sections.push(section),
                Err(e) => {
                    // Handle unknown sections as warnings, other errors as errors
                    let (severity, message) = if e.to_string().contains("Unknown section") {
                        (IssueSeverity::Warning, e.to_string())
                    } else {
                        (
                            IssueSeverity::Error,
                            format!("Failed to parse section: {}", e),
                        )
                    };

                    self.issues.push(ParseIssue::new(
                        severity,
                        IssueCategory::Structure,
                        message,
                        self.line,
                    ));

                    // Skip to next section for error recovery
                    self.skip_to_next_section();
                }
            }
        }

        Ok(Script {
            source: self.source,
            version: self.version,
            sections: self.sections,
            issues: self.issues,
        })
    }

    /// Parse a single section (e.g., [Script Info])
    fn parse_section(&mut self) -> Result<Section<'a>> {
        let _start_pos = self.position;

        // Expect section header like [Script Info]
        if !self.source[self.position..].starts_with('[') {
            return Err(CoreError::from(ParseError::ExpectedSectionHeader {
                line: self.line,
            }));
        }

        // Find closing bracket
        let header_end = self.source[self.position..]
            .find(']')
            .ok_or(CoreError::from(ParseError::UnclosedSectionHeader {
                line: self.line,
            }))?
            + self.position;

        let section_name = &self.source[self.position + 1..header_end];
        self.position = header_end + 1;
        self.skip_line();

        // Parse section content based on name
        match section_name.trim() {
            "Script Info" => self.parse_script_info(),
            "V4+ Styles" | "V4 Styles" => self.parse_styles(),
            "Events" => self.parse_events(),
            "Fonts" => self.parse_fonts(),
            "Graphics" => self.parse_graphics(),
            _ => {
                // Unknown section - consume until next section for recovery
                self.skip_to_next_section();
                Err(CoreError::from(ParseError::UnknownSection {
                    section: section_name.to_string(),
                    line: self.line,
                }))
            }
        }
    }

    /// Parse [Script Info] section
    fn parse_script_info(&mut self) -> Result<Section<'a>> {
        let mut fields = Vec::new();

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

            // Parse key: value pairs
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();

                // Update version if ScriptType found
                if key == "ScriptType" {
                    if let Some(version) = ScriptVersion::from_header(value) {
                        self.version = version;
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

        Ok(Section::ScriptInfo(ScriptInfo { fields }))
    }

    /// Parse [V4+ Styles] section
    fn parse_styles(&mut self) -> Result<Section<'a>> {
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
                // Skip format line for now - we assume standard V4+ format
                self.skip_line();
                continue;
            } else if line.starts_with("Style:") {
                // Parse style definition
                if let Some(style_data) = line.strip_prefix("Style:") {
                    if let Some(style) = self.parse_style_line(style_data.trim())? {
                        styles.push(style);
                    }
                }
                self.skip_line();
            } else {
                // Skip unrecognized lines
                self.skip_line();
            }
        }

        Ok(Section::Styles(styles))
    }

    /// Parse a single style line
    fn parse_style_line(&mut self, line: &'a str) -> Result<Option<Style<'a>>> {
        let parts: Vec<&str> = line.split(',').collect();

        // V4+ styles should have 23 fields minimum
        if parts.len() < 23 {
            self.issues.push(ParseIssue::new(
                IssueSeverity::Warning,
                IssueCategory::Format,
                format!("Style line has {} fields, expected 23", parts.len()),
                self.line,
            ));
            return Ok(None);
        }

        let style = Style {
            name: parts[0].trim(),
            fontname: parts[1].trim(),
            fontsize: parts[2].trim(),
            primary_colour: parts[3].trim(),
            secondary_colour: parts[4].trim(),
            outline_colour: parts[5].trim(),
            back_colour: parts[6].trim(),
            bold: parts[7].trim(),
            italic: parts[8].trim(),
            underline: parts[9].trim(),
            strikeout: parts[10].trim(),
            scale_x: parts[11].trim(),
            scale_y: parts[12].trim(),
            spacing: parts[13].trim(),
            angle: parts[14].trim(),
            border_style: parts[15].trim(),
            outline: parts[16].trim(),
            shadow: parts[17].trim(),
            alignment: parts[18].trim(),
            margin_l: parts[19].trim(),
            margin_r: parts[20].trim(),
            margin_v: parts[21].trim(),
            encoding: parts[22].trim(),
        };

        Ok(Some(style))
    }

    /// Parse [Events] section
    fn parse_events(&mut self) -> Result<Section<'a>> {
        let mut events = Vec::new();

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
                // Skip format line for now - we assume standard format
                self.skip_line();
                continue;
            } else if let Some(event) = self.parse_event_line(line)? {
                events.push(event);
                self.skip_line();
            } else {
                // Skip unrecognized lines
                self.skip_line();
            }
        }

        Ok(Section::Events(events))
    }

    /// Parse a single event line
    fn parse_event_line(&mut self, line: &'a str) -> Result<Option<ast::Event<'a>>> {
        // Find the event type prefix
        let colon_pos = line
            .find(':')
            .ok_or_else(|| CoreError::from(ParseError::InvalidFieldFormat { line: self.line }))?;

        let event_type_str = &line[..colon_pos];
        let event_data = &line[colon_pos + 1..];

        // Parse event type
        let event_type = match event_type_str.trim() {
            "Dialogue" => ast::EventType::Dialogue,
            "Comment" => ast::EventType::Comment,
            "Picture" => ast::EventType::Picture,
            "Sound" => ast::EventType::Sound,
            "Movie" => ast::EventType::Movie,
            "Command" => ast::EventType::Command,
            _ => {
                self.issues.push(ParseIssue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Format,
                    format!("Unknown event type: {}", event_type_str),
                    self.line,
                ));
                return Ok(None);
            }
        };

        // Parse comma-separated fields using splitn for robust text handling
        let mut parts = event_data.splitn(10, ',');
        let layer = parts.next().unwrap_or("").trim();
        let start = parts.next().unwrap_or("").trim();
        let end = parts.next().unwrap_or("").trim();
        let style = parts.next().unwrap_or("").trim();
        let name = parts.next().unwrap_or("").trim();
        let margin_l = parts.next().unwrap_or("").trim();
        let margin_r = parts.next().unwrap_or("").trim();
        let margin_v = parts.next().unwrap_or("").trim();
        let effect = parts.next().unwrap_or("").trim();
        let text = parts.next().unwrap_or("").trim(); // The rest of the string

        let event = ast::Event {
            event_type,
            layer,
            start,
            end,
            style,
            name,
            margin_l,
            margin_r,
            margin_v,
            effect,
            text,
        };

        Ok(Some(event))
    }

    /// Parse [Fonts] section
    fn parse_fonts(&mut self) -> Result<Section<'a>> {
        // TODO: Implement fonts parsing
        self.skip_to_next_section();
        Ok(Section::Fonts(Vec::new()))
    }

    /// Parse [Graphics] section
    fn parse_graphics(&mut self) -> Result<Section<'a>> {
        // TODO: Implement graphics parsing
        self.skip_to_next_section();
        Ok(Section::Graphics(Vec::new()))
    }

    /// Check if at start of next section
    fn at_next_section(&self) -> bool {
        self.source[self.position..].trim_start().starts_with('[')
    }

    /// Get current line as string slice
    fn current_line(&self) -> &'a str {
        let line_start = self.source[..self.position]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        let line_end = self.source[self.position..]
            .find('\n')
            .map(|pos| self.position + pos)
            .unwrap_or(self.source.len());

        &self.source[line_start..line_end]
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
                // Comment line - skip to next line
                self.skip_line();
            } else if trimmed != remaining {
                // Had leading whitespace - advance position
                self.position += remaining.len() - trimmed.len();
            } else {
                break;
            }
        }
    }

    /// Skip to next section for error recovery
    fn skip_to_next_section(&mut self) {
        while self.position < self.source.len() {
            if self.at_next_section() {
                break;
            }
            self.skip_line();
        }
    }
}

/// Incremental parsing delta for efficient editor updates
#[cfg(feature = "stream")]
#[derive(Debug, Clone)]
pub struct ScriptDelta<'a> {
    /// Sections that were added
    pub added: Vec<Section<'a>>,

    /// Sections that were modified (old index -> new section)
    pub modified: Vec<(usize, Section<'a>)>,

    /// Section indices that were removed
    pub removed: Vec<usize>,

    /// New parse issues
    pub new_issues: Vec<ParseIssue>,
}

#[cfg(feature = "stream")]
impl ScriptDelta<'_> {
    /// Check if the delta contains no changes
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.new_issues.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_script() {
        let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
        assert_eq!(script.sections().len(), 1);
        assert_eq!(script.version(), ScriptVersion::AssV4);
    }

    #[test]
    fn parse_with_script_type() {
        let script = Script::parse("[Script Info]\nScriptType: v4.00+\nTitle: Test").unwrap();
        assert_eq!(script.version(), ScriptVersion::AssV4);
    }

    #[test]
    fn parse_with_bom() {
        let script = Script::parse("\u{FEFF}[Script Info]\nTitle: Test").unwrap();
        assert_eq!(script.sections().len(), 1);
    }

    #[test]
    fn parse_empty_input() {
        let script = Script::parse("").unwrap();
        assert_eq!(script.sections().len(), 0);
    }

    #[test]
    fn parse_unknown_section() {
        let script =
            Script::parse("[Script Info]\nTitle: Test\n[Unknown Section]\nSomething: here")
                .unwrap();
        assert_eq!(script.sections().len(), 1);
        assert_eq!(script.issues().len(), 1);
        assert_eq!(script.issues()[0].severity, IssueSeverity::Warning);
    }
}
