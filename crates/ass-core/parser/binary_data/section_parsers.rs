//! Font and graphic section parsers built on [`BinaryDataParser`].
//!
//! [`FontsParser`] and [`GraphicsParser`] are thin wrappers that configure the
//! generic binary data parser for the `[Fonts]` and `[Graphics]` sections.

use super::binary_parser::BinaryDataParser;
use crate::parser::ast::{Font, Graphic, Section};

/// Parser for `[Fonts\]` section - wrapper around `BinaryDataParser`
pub(in crate::parser) struct FontsParser;

impl FontsParser {
    /// Parse `[Fonts\]` section
    ///
    /// Returns tuple of (Section, `final_position`, `final_line`)
    pub fn parse(source: &str, position: usize, line: usize) -> (Section<'_>, usize, usize) {
        let parser = BinaryDataParser::new(
            source,
            position,
            line,
            "fontname",
            |filename, data_lines, span| Font {
                filename,
                data_lines,
                span,
            },
        );
        let (fonts, final_position, final_line) = parser.parse();
        (Section::Fonts(fonts), final_position, final_line)
    }
}

/// Parser for `[Graphics\]` section - wrapper around `BinaryDataParser`
pub(in crate::parser) struct GraphicsParser;

impl GraphicsParser {
    /// Parse `[Graphics\]` section
    ///
    /// Returns tuple of (Section, `final_position`, `final_line`)
    pub fn parse(source: &str, position: usize, line: usize) -> (Section<'_>, usize, usize) {
        let parser = BinaryDataParser::new(
            source,
            position,
            line,
            "filename",
            |filename, data_lines, span| Graphic {
                filename,
                data_lines,
                span,
            },
        );
        let (graphics, final_position, final_line) = parser.parse();
        (Section::Graphics(graphics), final_position, final_line)
    }
}
