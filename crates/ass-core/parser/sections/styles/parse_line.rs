//! Standalone single-line style parsing for incremental updates.
//!
//! Provides [`StylesParser::parse_style_line`], a stateless entry point used by
//! incremental parsing to materialize a single [`Style`] from a line and format.

use super::StylesParser;
use crate::parser::{
    ast::{Span, Style},
    errors::ParseError,
};
use alloc::vec::Vec;

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
}
