//! Binary data parsing for [Fonts] and [Graphics] sections
//!
//! Handles UU-encoded font and graphic data embedded in ASS scripts.
//! Both sections use similar structure: filename declaration followed by
//! base64/UU-encoded data lines.

use alloc::vec::Vec;

use super::{
    ast::{Font, Graphic, Section, Span},
    position_tracker::PositionTracker,
};

/// Generic parser for binary data sections ([Fonts] and [Graphics])
pub(super) struct BinaryDataParser<'a, T> {
    /// Position tracker for accurate span generation
    tracker: PositionTracker<'a>,
    /// Expected key for entries (e.g., "fontname" or "filename")
    entry_key: &'static str,
    /// Function to construct AST node from filename, data lines, and span
    constructor: fn(&'a str, Vec<&'a str>, Span) -> T,
}

impl<'a, T> BinaryDataParser<'a, T> {
    /// Create new binary data parser
    pub fn new(
        source: &'a str,
        position: usize,
        line: usize,
        entry_key: &'static str,
        constructor: fn(&'a str, Vec<&'a str>, Span) -> T,
    ) -> Self {
        Self {
            tracker: PositionTracker::new_at(
                source,
                position,
                u32::try_from(line).unwrap_or(u32::MAX),
                1,
            ),
            entry_key,
            constructor,
        }
    }

    /// Parse complete binary data section
    ///
    /// Returns (entries, `final_position`, `final_line`)
    pub fn parse(mut self) -> (Vec<T>, usize, usize) {
        let mut entries = Vec::new();

        while !self.tracker.is_at_end() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.tracker.is_at_end() || self.at_next_section() {
                break;
            }

            if let Some(entry) = self.parse_entry() {
                entries.push(entry);
            }
        }

        let final_position = self.tracker.offset();
        let final_line = self.tracker.line() as usize;
        (entries, final_position, final_line)
    }

    /// Parse single entry (key: + data lines)
    fn parse_entry(&mut self) -> Option<T> {
        let entry_start = self.tracker.checkpoint();
        let line = self.current_line();

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key == self.entry_key {
                let filename = line[colon_pos + 1..].trim();
                self.tracker.skip_line();

                let data_lines = self.collect_data_lines();

                // Calculate span for this entry (from filename line to end of data)
                let entry_end = self.tracker.checkpoint();
                let span = entry_end.span_from(&entry_start);

                return Some((self.constructor)(filename, data_lines, span));
            }
        }

        self.tracker.skip_line();
        None
    }

    /// Collect UU-encoded data lines until next section or empty line
    fn collect_data_lines(&mut self) -> Vec<&'a str> {
        let mut data_lines = Vec::new();

        while !self.tracker.is_at_end() && !self.at_next_section() {
            let data_line = self.current_line();
            let trimmed = data_line.trim();

            if trimmed.is_empty() || trimmed.starts_with('[') {
                break;
            }

            // Skip comment lines
            if trimmed.starts_with(';') || trimmed.starts_with('!') {
                self.tracker.skip_line();
                continue;
            }

            // Stop at hash comments (# followed by space or at end of line)
            // But not UU-encoded data (# followed immediately by encoded chars)
            if trimmed.starts_with("# ") || trimmed == "#" {
                break;
            }

            data_lines.push(data_line);
            self.tracker.skip_line();
        }

        data_lines
    }

    /// Check if at start of next section
    fn at_next_section(&self) -> bool {
        self.tracker.remaining().trim_start().starts_with('[')
    }

    /// Get current line from source
    fn current_line(&self) -> &'a str {
        let remaining = self.tracker.remaining();
        let end = remaining.find('\n').unwrap_or(remaining.len());
        &remaining[..end]
    }

    /// Skip whitespace and comment lines
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.tracker.skip_whitespace();

            let remaining = self.tracker.remaining();
            if remaining.is_empty() {
                break;
            }

            if remaining.starts_with(';') || remaining.starts_with("!:") {
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

/// Parser for [Fonts] section - wrapper around `BinaryDataParser`
pub(super) struct FontsParser;

impl FontsParser {
    /// Parse [Fonts] section
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

/// Parser for [Graphics] section - wrapper around `BinaryDataParser`
pub(super) struct GraphicsParser;

impl GraphicsParser {
    /// Parse [Graphics] section
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_parser_empty_section() {
        let source = "";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert!(fonts.is_empty());
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_single_font() {
        let source = "fontname: arial.ttf\ndata1\ndata2\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert!(graphics.is_empty());
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_single_graphic() {
        let source = "filename: logo.png\nimage_data1\nimage_data2\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

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
        let (section, _, _) = FontsParser::parse(source, 0, 1);

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
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "D:\\Images\\logo.png");
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn fonts_parser_malformed_entry_no_colon() {
        let source = "invalid_font_entry\ndata1\ndata2\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            // Should skip malformed entries without colon
            assert!(fonts.is_empty());
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_empty_filename() {
        let source = "fontname: \ndata1\ndata2\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "");
            assert_eq!(fonts[0].data_lines.len(), 2);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_whitespace_only_filename() {
        let source = "fontname:   \ndata1\ndata2\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "");
            assert_eq!(fonts[0].data_lines.len(), 2);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_comments_between_data_lines() {
        let source =
            "fontname: arial.ttf\ndata1\n; Comment line\ndata2\n! Another comment\ndata3\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "arial.ttf");
            // Comments should be skipped, only data lines included
            assert_eq!(fonts[0].data_lines.len(), 3);
            assert_eq!(fonts[0].data_lines[0], "data1");
            assert_eq!(fonts[0].data_lines[1], "data2");
            assert_eq!(fonts[0].data_lines[2], "data3");
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_empty_lines_between_data() {
        let source = "fontname: arial.ttf\ndata1\n\n\ndata2\n   \ndata3\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "arial.ttf");
            // Parser stops at first empty line
            assert_eq!(fonts[0].data_lines.len(), 1);
            assert_eq!(fonts[0].data_lines[0], "data1");
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_entry_at_end_of_file() {
        let source = "fontname: arial.ttf\ndata1\ndata2";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            assert_eq!(fonts[0].filename, "arial.ttf");
            assert_eq!(fonts[0].data_lines.len(), 2);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn fonts_parser_mixed_comment_styles() {
        let source = "fontname: arial.ttf\ndata1\n; Semicolon comment\ndata2\n! Exclamation comment\ndata3\n# Hash comment\ndata4\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 1);
            // Hash comments are not skipped, so parsing stops at # Hash comment
            assert_eq!(fonts[0].data_lines.len(), 3);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn graphics_parser_malformed_entry_no_colon() {
        let source = "invalid_graphic_entry\nimg_data1\nimg_data2\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            // Should skip malformed entries without colon
            assert!(graphics.is_empty());
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_empty_filename() {
        let source = "filename: \nimg_data1\nimg_data2\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "");
            assert_eq!(graphics[0].data_lines.len(), 2);
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_whitespace_only_filename() {
        let source = "filename:   \nimg_data1\nimg_data2\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "");
            assert_eq!(graphics[0].data_lines.len(), 2);
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_comments_between_data_lines() {
        let source = "filename: logo.png\nimg_data1\n; Comment line\nimg_data2\n! Another comment\nimg_data3\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "logo.png");
            // Comments should be skipped, only data lines included
            assert_eq!(graphics[0].data_lines.len(), 3);
            assert_eq!(graphics[0].data_lines[0], "img_data1");
            assert_eq!(graphics[0].data_lines[1], "img_data2");
            assert_eq!(graphics[0].data_lines[2], "img_data3");
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_empty_lines_between_data() {
        let source = "filename: logo.png\nimg_data1\n\n\nimg_data2\n   \nimg_data3\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "logo.png");
            // Parser stops at first empty line
            assert_eq!(graphics[0].data_lines.len(), 1);
            assert_eq!(graphics[0].data_lines[0], "img_data1");
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_entry_at_end_of_file() {
        let source = "filename: logo.png\nimg_data1\nimg_data2";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            assert_eq!(graphics[0].filename, "logo.png");
            assert_eq!(graphics[0].data_lines.len(), 2);
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn graphics_parser_mixed_comment_styles() {
        let source = "filename: logo.png\nimg_data1\n; Semicolon comment\nimg_data2\n! Exclamation comment\nimg_data3\n# Hash comment\nimg_data4\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 1);
            // Hash comments are not skipped, so parsing stops at # Hash comment
            assert_eq!(graphics[0].data_lines.len(), 3);
        } else {
            panic!("Expected Graphics section");
        }
    }

    #[test]
    fn fonts_parser_multiple_entries_with_edge_cases() {
        let source = "fontname: font1.ttf\ndata1_1\ndata1_2\n\ninvalid_entry_no_colon\n\nfontname: font2.ttf\n; Comment\ndata2_1\n\nfontname: \ndata3_1\n";
        let (section, _, _) = FontsParser::parse(source, 0, 1);

        if let Section::Fonts(fonts) = section {
            assert_eq!(fonts.len(), 3); // All valid font entries should be parsed

            assert_eq!(fonts[0].filename, "font1.ttf");
            assert_eq!(fonts[0].data_lines.len(), 2);

            assert_eq!(fonts[1].filename, "font2.ttf");
            assert_eq!(fonts[1].data_lines.len(), 1);

            assert_eq!(fonts[2].filename, "");
            assert_eq!(fonts[2].data_lines.len(), 1);
        } else {
            panic!("Expected Fonts section");
        }
    }

    #[test]
    fn graphics_parser_multiple_entries_with_edge_cases() {
        let source = "filename: image1.png\nimg1_1\nimg1_2\n\ninvalid_entry_no_colon\n\nfilename: image2.png\n; Comment\nimg2_1\n\nfilename: \nimg3_1\n";
        let (section, _, _) = GraphicsParser::parse(source, 0, 1);

        if let Section::Graphics(graphics) = section {
            assert_eq!(graphics.len(), 3); // All valid graphic entries should be parsed

            assert_eq!(graphics[0].filename, "image1.png");
            assert_eq!(graphics[0].data_lines.len(), 2);

            assert_eq!(graphics[1].filename, "image2.png");
            assert_eq!(graphics[1].data_lines.len(), 1);

            assert_eq!(graphics[2].filename, "");
            assert_eq!(graphics[2].data_lines.len(), 1);
        } else {
            panic!("Expected Graphics section");
        }
    }
}
