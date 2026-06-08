//! Parsing entry points and context-aware single-line parsing.
//!
//! Provides [`Script::parse`], the [`Script::builder`] constructor, and the
//! format-aware helpers used to parse individual style and event lines against
//! the script's stored format definitions.

use crate::parser::ast::{Event, Style};
use crate::parser::errors::ParseError;
use crate::parser::main::Parser;
use crate::Result;

use super::builder::ScriptBuilder;
use super::Script;

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
    ///
    /// # Errors
    ///
    /// Returns an error if the source contains malformed section headers or
    /// other unrecoverable syntax errors.
    pub fn parse(source: &'a str) -> Result<Self> {
        let parser = Parser::new(source);
        Ok(parser.parse())
    }

    /// Create a new script builder for parsing with optional extensions
    ///
    /// The builder pattern allows configuration of parsing options including
    /// extension registry for custom tag handlers and section processors.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::parser::Script;
    /// # #[cfg(feature = "plugins")]
    /// # use ass_core::plugin::ExtensionRegistry;
    /// # #[cfg(feature = "plugins")]
    /// let registry = ExtensionRegistry::new();
    /// # #[cfg(feature = "plugins")]
    /// let script = Script::builder()
    ///     .with_registry(&registry)
    ///     .parse("[Script Info]\nTitle: Test")?;
    /// # #[cfg(not(feature = "plugins"))]
    /// let script = Script::builder()
    ///     .parse("[Script Info]\nTitle: Test")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn builder() -> ScriptBuilder<'a> {
        ScriptBuilder::new()
    }

    /// Parse a style line with context from the script
    ///
    /// Uses the script's stored format for [V4+ Styles] section if available,
    /// otherwise falls back to default format.
    ///
    /// # Arguments
    ///
    /// * `line` - The style line to parse (without "Style:" prefix)
    /// * `line_number` - The line number for error reporting
    ///
    /// # Returns
    ///
    /// Parsed Style or error if the line is malformed
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::InsufficientFields`] if the line has fewer fields than expected
    pub fn parse_style_line_with_context(
        &self,
        line: &'a str,
        line_number: u32,
    ) -> core::result::Result<Style<'a>, ParseError> {
        use crate::parser::sections::StylesParser;

        let format = self.styles_format.as_deref().unwrap_or(&[
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

        StylesParser::parse_style_line(line, format, line_number)
    }

    /// Parse an event line with context from the script
    ///
    /// Uses the script's stored format for `[Events\]` section if available,
    /// otherwise falls back to default format.
    ///
    /// # Arguments
    ///
    /// * `line` - The event line to parse (e.g., "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text")
    /// * `line_number` - The line number for error reporting
    ///
    /// # Returns
    ///
    /// Parsed Event or error if the line is malformed
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::InvalidEventType`] if the line doesn't start with a valid event type
    /// Returns [`ParseError::InsufficientFields`] if the line has fewer fields than expected
    pub fn parse_event_line_with_context(
        &self,
        line: &'a str,
        line_number: u32,
    ) -> core::result::Result<Event<'a>, ParseError> {
        use crate::parser::sections::EventsParser;

        let format = self.events_format.as_deref().unwrap_or(&[
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ]);

        EventsParser::parse_event_line(line, format, line_number)
    }
}
