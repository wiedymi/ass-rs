//! Binary data parsing for [Fonts] and [Graphics] sections
//!
//! Handles UU-encoded font and graphic data embedded in ASS scripts.
//! Both sections use similar structure: filename declaration followed by
//! base64/UU-encoded data lines.

use alloc::vec::Vec;

use super::ast::{Font, Graphic, Section};

/// Parser for [Fonts] section with embedded font data
pub(super) struct FontsParser<'a> {
    /// Source text being parsed
    source: &'a str,
    /// Current byte position in source
    position: usize,
    /// Current line number for error reporting
    line: usize,
}

impl<'a> FontsParser<'a> {
    /// Create new fonts parser
    pub const fn new(source: &'a str, position: usize, line: usize) -> Self {
        Self {
            source,
            position,
            line,
        }
    }

    /// Parse complete [Fonts] section
    pub fn parse(mut self) -> Section<'a> {
        let mut fonts = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() || self.at_next_section() {
                break;
            }

            if let Some(font) = self.parse_font_entry() {
                fonts.push(font);
            }
        }

        Section::Fonts(fonts)
    }

    /// Parse single font entry (fontname: + data lines)
    fn parse_font_entry(&mut self) -> Option<Font<'a>> {
        let line_start = self.position;
        let line_end = self.find_line_end();
        let line = &self.source[line_start..line_end];

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key == "fontname" {
                let filename = line[colon_pos + 1..].trim();
                self.skip_line();

                let data_lines = self.collect_data_lines();
                return Some(Font {
                    filename,
                    data_lines,
                });
            }
        }

        self.skip_line();
        None
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

            // Skip comment lines
            if trimmed.starts_with(';') || trimmed.starts_with('!') {
                self.skip_line();
                continue;
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
            .map_or(self.source.len(), |pos| self.position + pos)
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
    /// Source text being parsed
    source: &'a str,
    /// Current byte position in source
    position: usize,
    /// Current line number for error reporting
    line: usize,
}

impl<'a> GraphicsParser<'a> {
    /// Create new graphics parser
    pub const fn new(source: &'a str, position: usize, line: usize) -> Self {
        Self {
            source,
            position,
            line,
        }
    }

    /// Parse complete [Graphics] section
    pub fn parse(mut self) -> Section<'a> {
        let mut graphics = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() || self.at_next_section() {
                break;
            }

            if let Some(graphic) = self.parse_graphic_entry() {
                graphics.push(graphic);
            }
        }

        Section::Graphics(graphics)
    }

    /// Parse single graphic entry (filename: + data lines)
    fn parse_graphic_entry(&mut self) -> Option<Graphic<'a>> {
        let line_start = self.position;
        let line_end = self.find_line_end();
        let line = &self.source[line_start..line_end];

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key == "filename" {
                let filename = line[colon_pos + 1..].trim();
                self.skip_line();

                let data_lines = self.collect_data_lines();
                return Some(Graphic {
                    filename,
                    data_lines,
                });
            }
        }

        self.skip_line();
        None
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

            // Skip comment lines
            if trimmed.starts_with(';') || trimmed.starts_with('!') {
                self.skip_line();
                continue;
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
            .map_or(self.source.len(), |pos| self.position + pos)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_parser_empty_section() {
        let source = "";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert!(fonts.is_empty());
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_single_font() {
        let source = "fontname: arial.ttf\ndata1\ndata2\n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "arial.ttf");
            assert_eq!(fonts[0].data_lines.len(), 2);
            assert_eq!(fonts[0].data_lines[0], "data1");
            assert_eq!(fonts[0].data_lines[1], "data2");
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_multiple_fonts() {
        let source = "fontname: font1.ttf\ndata1\ndata2\n\nfontname: font2.ttf\ndata3\ndata4\n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 2);

            assert_eq!(fonts[0].filename, "font1.ttf");
            assert_eq!(fonts[0].data_lines.len(), 2);
            assert_eq!(fonts[0].data_lines[0], "data1");
            assert_eq!(fonts[0].data_lines[1], "data2");

            assert_eq!(fonts[1].filename, "font2.ttf");
            assert_eq!(fonts[1].data_lines.len(), 2);
            assert_eq!(fonts[1].data_lines[0], "data3");
            assert_eq!(fonts[1].data_lines[1], "data4");
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_with_comments() {
        let source = "; This is a comment\nfontname: test.ttf\n!: Another comment\ndata1\ndata2\n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "test.ttf");
            assert_eq!(fonts[0].data_lines.len(), 2);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_with_whitespace() {
        let source = "  fontname:  arial.ttf  \n  data1  \n  data2  \n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "arial.ttf");
            assert_eq!(fonts[0].data_lines.len(), 2);
            assert_eq!(fonts[0].data_lines[0], "  data1  ");
            assert_eq!(fonts[0].data_lines[1], "  data2  ");
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_stops_at_next_section() {
        let source = "fontname: test.ttf\ndata1\ndata2\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "test.ttf");
            assert_eq!(fonts[0].data_lines.len(), 2);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_malformed_entry() {
        let source = "invalid_line\nfontname: valid.ttf\ndata1\n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "valid.ttf");
            assert_eq!(fonts[0].data_lines.len(), 1);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_no_data_lines() {
        let source = "fontname: empty.ttf\n[Events]\n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "empty.ttf");
            assert!(fonts[0].data_lines.is_empty());
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn graphics_parser_empty_section() {
        let source = "";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert!(graphics.is_empty());
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_single_graphic() {
        let source = "filename: logo.png\nimage_data1\nimage_data2\n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "logo.png");
            assert_eq!(graphics[0].data_lines.len(), 2);
            assert_eq!(graphics[0].data_lines[0], "image_data1");
            assert_eq!(graphics[0].data_lines[1], "image_data2");
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_multiple_graphics() {
        let source = "filename: img1.png\ndata1\ndata2\n\nfilename: img2.jpg\ndata3\ndata4\n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 2);

            assert_eq!(graphics[0].filename, "img1.png");
            assert_eq!(graphics[0].data_lines.len(), 2);
            assert_eq!(graphics[0].data_lines[0], "data1");
            assert_eq!(graphics[0].data_lines[1], "data2");

            assert_eq!(graphics[1].filename, "img2.jpg");
            assert_eq!(graphics[1].data_lines.len(), 2);
            assert_eq!(graphics[1].data_lines[0], "data3");
            assert_eq!(graphics[1].data_lines[1], "data4");
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_with_comments() {
        let source = "; Image section comment\nfilename: test.png\n!: Another comment\nimg_data1\nimg_data2\n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "test.png");
            assert_eq!(graphics[0].data_lines.len(), 2);
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_with_whitespace() {
        let source = "  filename:  logo.png  \n  img_data1  \n  img_data2  \n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "logo.png");
            assert_eq!(graphics[0].data_lines.len(), 2);
            assert_eq!(graphics[0].data_lines[0], "  img_data1  ");
            assert_eq!(graphics[0].data_lines[1], "  img_data2  ");
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_stops_at_next_section() {
        let source = "filename: test.png\nimg_data1\nimg_data2\n[Styles]\nFormat: Name, Fontname, Fontsize\n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "test.png");
            assert_eq!(graphics[0].data_lines.len(), 2);
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_malformed_entry() {
        let source = "invalid_line_without_colon\nfilename: valid.png\nimg_data1\n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "valid.png");
            assert_eq!(graphics[0].data_lines.len(), 1);
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_no_data_lines() {
        let source = "filename: empty.png\n[Fonts]\n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "empty.png");
            assert!(graphics[0].data_lines.is_empty());
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn fonts_parser_colon_in_filename() {
        let source = "fontname: C:\\Fonts\\arial.ttf\ndata1\n";
        let parser = FontsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "C:\\Fonts\\arial.ttf");
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn graphics_parser_colon_in_filename() {
        let source = "filename: D:\\Images\\logo.png\nimg_data1\n";
        let parser = GraphicsParser::new(source, 0, 1);
        let section = parser.parse();

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "D:\\Images\\logo.png");
        } else {
            panic!("Expected Graphics section");
        }
    }
}
