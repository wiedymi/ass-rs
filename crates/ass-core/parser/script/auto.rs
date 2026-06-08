//! Section-context line parsing with automatic type detection.
//!
//! Provides [`Script::parse_line_auto`], which infers a line's section from its
//! prefix, and [`Script::parse_line_in_section`], which parses a line against a
//! known section type using the script's stored formats.

use alloc::boxed::Box;

use crate::parser::ast::SectionType;
use crate::parser::errors::ParseError;
use crate::Result;

use super::types::LineContent;
use super::Script;

impl<'a> Script<'a> {
    /// Parse a line based on its section context
    ///
    /// Automatically determines the section type from the line content and parses accordingly.
    ///
    /// # Arguments
    ///
    /// * `line` - The line to parse
    /// * `line_number` - The line number for error reporting
    ///
    /// # Returns
    ///
    /// A tuple of (`section_type`, `parsed_content`) or error
    ///
    /// # Errors
    ///
    /// Returns error if the line format is invalid or section type cannot be determined
    pub fn parse_line_auto(
        &self,
        line: &'a str,
        line_number: u32,
    ) -> core::result::Result<(SectionType, LineContent<'a>), ParseError> {
        let trimmed = line.trim();

        // Try to detect line type
        if trimmed.starts_with("Style:") {
            if let Some(style_data) = trimmed.strip_prefix("Style:") {
                let style = self.parse_style_line_with_context(style_data.trim(), line_number)?;
                return Ok((SectionType::Styles, LineContent::Style(Box::new(style))));
            }
        } else if trimmed.starts_with("Dialogue:")
            || trimmed.starts_with("Comment:")
            || trimmed.starts_with("Picture:")
            || trimmed.starts_with("Sound:")
            || trimmed.starts_with("Movie:")
            || trimmed.starts_with("Command:")
        {
            let event = self.parse_event_line_with_context(trimmed, line_number)?;
            return Ok((SectionType::Events, LineContent::Event(Box::new(event))));
        } else if trimmed.contains(':') && !trimmed.starts_with("Format:") {
            // Likely a Script Info field
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();
                return Ok((SectionType::ScriptInfo, LineContent::Field(key, value)));
            }
        }

        Err(ParseError::InvalidFieldFormat {
            line: line_number as usize,
        })
    }

    /// Parse line in section context
    ///
    /// Parses a single line knowing its section context, using stored format information.
    ///
    /// # Arguments
    ///
    /// * `section_type` - The type of section containing this line
    /// * `line` - The line text to parse
    /// * `line_number` - Line number for error reporting
    ///
    /// # Returns
    ///
    /// Parsed line content or error
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::MissingFormat`] if format information is missing
    /// Returns other parse errors from line-specific parsers
    pub fn parse_line_in_section(
        &self,
        section_type: SectionType,
        line: &'a str,
        line_number: u32,
    ) -> Result<LineContent<'a>> {
        match section_type {
            SectionType::Events => {
                let format = self
                    .events_format()
                    .ok_or(crate::utils::errors::CoreError::Parse(
                        ParseError::MissingFormat,
                    ))?;
                crate::parser::sections::EventsParser::parse_event_line(line, format, line_number)
                    .map(|event| LineContent::Event(Box::new(event)))
                    .map_err(crate::utils::errors::CoreError::Parse)
            }
            SectionType::Styles => {
                let format = self
                    .styles_format()
                    .ok_or(crate::utils::errors::CoreError::Parse(
                        ParseError::MissingFormat,
                    ))?;
                crate::parser::sections::StylesParser::parse_style_line(line, format, line_number)
                    .map(|style| LineContent::Style(Box::new(style)))
                    .map_err(crate::utils::errors::CoreError::Parse)
            }
            SectionType::ScriptInfo => {
                // Parse as key-value field
                if let Some((key, value)) = line.split_once(':') {
                    Ok(LineContent::Field(key.trim(), value.trim()))
                } else {
                    Err(crate::utils::errors::CoreError::Parse(
                        ParseError::InvalidFieldFormat {
                            line: line_number as usize,
                        },
                    ))
                }
            }
            _ => Err(crate::utils::errors::CoreError::Parse(
                ParseError::UnsupportedSection(section_type),
            )),
        }
    }
}
