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
pub mod sections;

#[cfg(feature = "stream")]
pub mod streaming;

use sections::{EventsParser, ScriptInfoParser, StylesParser};

pub use ast::{Event, ScriptInfo, Section, Style};
pub use errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue, ParseResult};

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
    styles_format: Option<Vec<&'a str>>,
    events_format: Option<Vec<&'a str>>,
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
            styles_format: None,
            events_format: None,
        }
    }

    /// Parse complete script
    fn parse(mut self) -> Result<Script<'a>> {
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
                            format!("Failed to parse section: {}", e),
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

        Ok(Script {
            source: self.source,
            version: self.version,
            sections: self.sections,
            issues: self.issues,
        })
    }

    /// Parse a single section (e.g., [Script Info])
    fn parse_section(&mut self) -> Result<Section<'a>> {
        if !self.source[self.position..].starts_with('[') {
            return Err(CoreError::from(ParseError::ExpectedSectionHeader {
                line: self.line,
            }));
        }

        let header_end = self.source[self.position..]
            .find(']')
            .ok_or(CoreError::from(ParseError::UnclosedSectionHeader {
                line: self.line,
            }))?
            + self.position;

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
            "Fonts" => self.parse_fonts(),
            "Graphics" => self.parse_graphics(),
            _ => {
                self.skip_to_next_section();
                Err(CoreError::from(ParseError::UnknownSection {
                    section: section_name.to_string(),
                    line: self.line,
                }))
            }
        }
    }

    /// Parse [Fonts] section
    fn parse_fonts(&mut self) -> Result<Section<'a>> {
        self.skip_to_next_section();
        Ok(Section::Fonts(Vec::new()))
    }

    /// Parse [Graphics] section
    fn parse_graphics(&mut self) -> Result<Section<'a>> {
        self.skip_to_next_section();
        Ok(Section::Graphics(Vec::new()))
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

    #[test]
    fn parse_with_custom_format() {
        let script_text = r#"[Script Info]
Title: Format Test
ScriptType: v4.00+

[V4+ Styles]
Format: Fontsize, Name, Fontname, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: 20,Custom,Arial,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Start, Layer, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0:00:00.00,0,0:00:05.00,Custom,,0,0,0,,Custom format test
"#;

        let script = Script::parse(script_text).unwrap();
        assert_eq!(script.sections().len(), 3);

        if let Some(Section::Styles(styles)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Styles(_)))
        {
            assert_eq!(styles.len(), 1);
            let style = &styles[0];
            assert_eq!(style.name, "Custom");
            assert_eq!(style.fontname, "Arial");
            assert_eq!(style.fontsize, "20"); // This was first in our custom format
        } else {
            panic!("Should have found styles section");
        }

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert_eq!(event.start, "0:00:00.00");
            assert_eq!(event.layer, "0"); // Layer was second in our custom format
            assert_eq!(event.end, "0:00:05.00");
            assert_eq!(event.style, "Custom");
            assert_eq!(event.text, "Custom format test");
        } else {
            panic!("Should have found events section");
        }
    }
}
