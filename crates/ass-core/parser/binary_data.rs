//! Binary data parsing for [Fonts] and [Graphics] sections
//!
//! Handles UU-encoded font and graphic data embedded in ASS scripts.
//! Both sections use similar structure: filename declaration followed by
//! base64/UU-encoded data lines.

use crate::Result;
use alloc::vec::Vec;

use super::ast::{Font, Graphic, Section};

/// Parser for [Fonts] section with embedded font data
pub(super) struct FontsParser<'a> {
    source: &'a str,
    position: usize,
    line: usize,
}

impl<'a> FontsParser<'a> {
    /// Create new fonts parser
    pub fn new(source: &'a str, position: usize, line: usize) -> Self {
        Self {
            source,
            position,
            line,
        }
    }

    /// Parse complete [Fonts] section
    pub fn parse(mut self) -> Result<Section<'a>> {
        let mut fonts = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() || self.at_next_section() {
                break;
            }

            if let Some(font) = self.parse_font_entry()? {
                fonts.push(font);
            }
        }

        Ok(Section::Fonts(fonts))
    }

    /// Parse single font entry (fontname: + data lines)
    fn parse_font_entry(&mut self) -> Result<Option<Font<'a>>> {
        let line_start = self.position;
        let line_end = self.find_line_end();
        let line = &self.source[line_start..line_end];

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key == "fontname" {
                let filename = line[colon_pos + 1..].trim();
                self.skip_line();

                let data_lines = self.collect_data_lines();
                return Ok(Some(Font {
                    filename,
                    data_lines,
                }));
            }
        }

        self.skip_line();
        Ok(None)
    }

    /// Collect UU-encoded data lines until next section or empty line
    fn collect_data_lines(&mut self) -> Vec<&'a str> {
        let mut data_lines = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            let data_line_start = self.position;
            let data_line_end = self.find_line_end();
            let data_line = &self.source[data_line_start..data_line_end];
            let trimmed = data_line.trim();

            if trimmed.is_empty() || trimmed.starts_with('[') {
                break;
            }

            data_lines.push(data_line);
            self.skip_line();
        }

        data_lines
    }

    /// Check if at start of next section
    fn at_next_section(&self) -> bool {
        self.source[self.position..].trim_start().starts_with('[')
    }

    /// Find end of current line
    fn find_line_end(&self) -> usize {
        self.source[self.position..]
            .find('\n')
            .map(|pos| self.position + pos)
            .unwrap_or(self.source.len())
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
}

/// Parser for [Graphics] section with embedded graphic data
pub(super) struct GraphicsParser<'a> {
    source: &'a str,
    position: usize,
    line: usize,
}

impl<'a> GraphicsParser<'a> {
    /// Create new graphics parser
    pub fn new(source: &'a str, position: usize, line: usize) -> Self {
        Self {
            source,
            position,
            line,
        }
    }

    /// Parse complete [Graphics] section
    pub fn parse(mut self) -> Result<Section<'a>> {
        let mut graphics = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() || self.at_next_section() {
                break;
            }

            if let Some(graphic) = self.parse_graphic_entry()? {
                graphics.push(graphic);
            }
        }

        Ok(Section::Graphics(graphics))
    }

    /// Parse single graphic entry (filename: + data lines)
    fn parse_graphic_entry(&mut self) -> Result<Option<Graphic<'a>>> {
        let line_start = self.position;
        let line_end = self.find_line_end();
        let line = &self.source[line_start..line_end];

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key == "filename" {
                let filename = line[colon_pos + 1..].trim();
                self.skip_line();

                let data_lines = self.collect_data_lines();
                return Ok(Some(Graphic {
                    filename,
                    data_lines,
                }));
            }
        }

        self.skip_line();
        Ok(None)
    }

    /// Collect UU-encoded data lines until next section or empty line
    fn collect_data_lines(&mut self) -> Vec<&'a str> {
        let mut data_lines = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            let data_line_start = self.position;
            let data_line_end = self.find_line_end();
            let data_line = &self.source[data_line_start..data_line_end];
            let trimmed = data_line.trim();

            if trimmed.is_empty() || trimmed.starts_with('[') {
                break;
            }

            data_lines.push(data_line);
            self.skip_line();
        }

        data_lines
    }

    /// Check if at start of next section
    fn at_next_section(&self) -> bool {
        self.source[self.position..].trim_start().starts_with('[')
    }

    /// Find end of current line
    fn find_line_end(&self) -> usize {
        self.source[self.position..]
            .find('\n')
            .map(|pos| self.position + pos)
            .unwrap_or(self.source.len())
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
}
