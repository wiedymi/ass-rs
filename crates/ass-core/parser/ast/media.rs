//! Media AST nodes for embedded fonts and graphics
//!
//! Contains Font and Graphic structs representing embedded media from the
//! [Fonts] and [Graphics] sections with zero-copy design and UU-decoding.

#[cfg(not(feature = "std"))]
extern crate alloc;

use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::format;

use super::Span;
#[cfg(debug_assertions)]
use core::ops::Range;

/// Embedded font from `[Fonts\]` section
///
/// Represents a font file embedded in the ASS script using UU-encoding.
/// Provides lazy decoding to avoid processing overhead unless the font
/// data is actually needed.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::{Font, Span};
///
/// let font = Font {
///     filename: "custom.ttf",
///     data_lines: vec!["begin 644 custom.ttf", "M'XL..."],
///     span: Span::new(0, 0, 0, 0),
/// };
///
/// // Decode when needed
/// let decoded = font.decode_data()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Font<'a> {
    /// Font filename as it appears in the `[Fonts\]` section
    pub filename: &'a str,

    /// UU-encoded font data lines as zero-copy spans
    pub data_lines: Vec<&'a str>,

    /// Span in source text where this font is defined
    pub span: Span,
}

impl Font<'_> {
    /// Decode UU-encoded font data with lazy evaluation
    ///
    /// Converts the UU-encoded data lines to raw binary font data.
    /// This is expensive so it's only done when explicitly requested.
    ///
    /// # Returns
    ///
    /// Decoded binary font data on success, error if UU-decoding fails
    ///
    /// # Errors
    ///
    /// Returns an error if the UU-encoded data is malformed or cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{Font, Span};
    /// # let font = Font { filename: "test.ttf", data_lines: vec![], span: Span::new(0, 0, 0, 0) };
    /// match font.decode_data() {
    ///     Ok(data) => println!("Font size: {} bytes", data.len()),
    ///     Err(e) => eprintln!("Decode error: {}", e),
    /// }
    /// ```
    pub fn decode_data(&self) -> Result<Vec<u8>, crate::utils::CoreError> {
        crate::utils::decode_uu_data(self.data_lines.iter().copied())
    }

    /// Convert font to ASS string representation
    ///
    /// Generates the font entry as it appears in the `[Fonts\]` section.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{Font, Span};
    /// let font = Font {
    ///     filename: "custom.ttf",
    ///     data_lines: vec!["begin 644 custom.ttf", "M'XL...", "end"],
    ///     span: Span::new(0, 0, 0, 0),
    /// };
    /// let ass_string = font.to_ass_string();
    /// assert!(ass_string.starts_with("fontname: custom.ttf\n"));
    /// assert!(ass_string.contains("M'XL..."));
    /// ```
    #[must_use]
    pub fn to_ass_string(&self) -> alloc::string::String {

        let mut result = format!("fontname: {}\n", self.filename);
        for line in &self.data_lines {
            result.push_str(line);
            result.push('\n');
        }
        result
    }

    /// Validate all spans in this Font reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that filename and all data line references point to
    /// memory within the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let filename_ptr = self.filename.as_ptr() as usize;
        let filename_valid = source_range.contains(&filename_ptr);

        let data_valid = self.data_lines.iter().all(|line| {
            let ptr = line.as_ptr() as usize;
            source_range.contains(&ptr)
        });

        filename_valid && data_valid
    }
}

/// Embedded graphic from `[Graphics\]` section
///
/// Represents an image file embedded in the ASS script using UU-encoding.
/// Commonly used for logos, textures, and other graphical elements.
/// Provides lazy decoding for performance.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::{Graphic, Span};
///
/// let graphic = Graphic {
///     filename: "logo.png",
///     data_lines: vec!["begin 644 logo.png", "M89PNG..."],
///     span: Span::new(0, 0, 0, 0),
/// };
///
/// let decoded = graphic.decode_data()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graphic<'a> {
    /// Graphic filename as it appears in the `[Graphics\]` section
    pub filename: &'a str,

    /// UU-encoded graphic data lines as zero-copy spans
    pub data_lines: Vec<&'a str>,

    /// Span in source text where this graphic is defined
    pub span: Span,
}

impl Graphic<'_> {
    /// Decode UU-encoded graphic data with lazy evaluation
    ///
    /// Converts the UU-encoded data lines to raw binary image data.
    /// This is expensive so it's only done when explicitly requested.
    ///
    /// # Returns
    ///
    /// Decoded binary image data on success, error if UU-decoding fails
    ///
    /// # Errors
    ///
    /// Returns an error if the UU-encoded data is malformed or cannot be decoded.
    pub fn decode_data(&self) -> Result<Vec<u8>, crate::utils::CoreError> {
        crate::utils::decode_uu_data(self.data_lines.iter().copied())
    }

    /// Convert graphic to ASS string representation
    ///
    /// Generates the graphic entry as it appears in the `[Graphics\]` section.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{Graphic, Span};
    /// let graphic = Graphic {
    ///     filename: "logo.png",
    ///     data_lines: vec!["begin 644 logo.png", "M'XL...", "end"],
    ///     span: Span::new(0, 0, 0, 0),
    /// };
    /// let ass_string = graphic.to_ass_string();
    /// assert!(ass_string.starts_with("filename: logo.png\n"));
    /// assert!(ass_string.contains("M'XL..."));
    /// ```
    #[must_use]
    pub fn to_ass_string(&self) -> alloc::string::String {

        let mut result = format!("filename: {}\n", self.filename);
        for line in &self.data_lines {
            result.push_str(line);
            result.push('\n');
        }
        result
    }

    /// Validate all spans in this Graphic reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that filename and all data line references point to
    /// memory within the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let filename_ptr = self.filename.as_ptr() as usize;
        let filename_valid = source_range.contains(&filename_ptr);

        let data_valid = self.data_lines.iter().all(|line| {
            let ptr = line.as_ptr() as usize;
            source_range.contains(&ptr)
        });

        filename_valid && data_valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::{format, vec};

    #[test]
    fn font_creation() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["line1", "line2"],
            span: Span::new(0, 0, 0, 0),
        };

        assert_eq!(font.filename, "test.ttf");
        assert_eq!(font.data_lines.len(), 2);
        assert_eq!(font.data_lines[0], "line1");
        assert_eq!(font.data_lines[1], "line2");
    }

    #[test]
    fn graphic_creation() {
        let graphic = Graphic {
            filename: "logo.png",
            data_lines: vec!["data1", "data2", "data3"],
            span: Span::new(0, 0, 0, 0),
        };

        assert_eq!(graphic.filename, "logo.png");
        assert_eq!(graphic.data_lines.len(), 3);
        assert_eq!(graphic.data_lines[0], "data1");
    }

    #[test]
    fn font_clone_eq() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["data"],
            span: Span::new(0, 0, 0, 0),
        };

        let cloned = font.clone();
        assert_eq!(font, cloned);
    }

    #[test]
    fn graphic_clone_eq() {
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec!["data"],
            span: Span::new(0, 0, 0, 0),
        };

        let cloned = graphic.clone();
        assert_eq!(graphic, cloned);
    }

    #[test]
    fn font_debug() {
        let font = Font {
            filename: "debug.ttf",
            data_lines: vec!["test"],
            span: Span::new(0, 0, 0, 0),
        };

        let debug_str = format!("{font:?}");
        assert!(debug_str.contains("Font"));
        assert!(debug_str.contains("debug.ttf"));
    }

    #[test]
    fn graphic_debug() {
        let graphic = Graphic {
            filename: "debug.png",
            data_lines: vec!["test"],
            span: Span::new(0, 0, 0, 0),
        };

        let debug_str = format!("{graphic:?}");
        assert!(debug_str.contains("Graphic"));
        assert!(debug_str.contains("debug.png"));
    }

    #[test]
    fn empty_data_lines() {
        let font = Font {
            filename: "empty.ttf",
            data_lines: Vec::new(),
            span: Span::new(0, 0, 0, 0),
        };

        let graphic = Graphic {
            filename: "empty.png",
            data_lines: Vec::new(),
            span: Span::new(0, 0, 0, 0),
        };

        assert!(font.data_lines.is_empty());
        assert!(graphic.data_lines.is_empty());
    }

    #[test]
    fn media_inequality() {
        let font1 = Font {
            filename: "font1.ttf",
            data_lines: vec!["data"],
            span: Span::new(0, 0, 0, 0),
        };

        let font2 = Font {
            filename: "font2.ttf",
            data_lines: vec!["data"],
            span: Span::new(0, 0, 0, 0),
        };

        assert_ne!(font1, font2);
    }

    #[test]
    fn font_decode_data_valid() {
        // Valid UU-encoded data for "Cat" (test with known encoding)
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["#0V%T", "`"],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = font.decode_data().unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn font_decode_data_empty_lines() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec![],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = font.decode_data().unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn font_decode_data_whitespace_lines() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["   ", "\t\n", ""],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = font.decode_data().unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn font_decode_data_with_end_marker() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["#0V%T", "end"],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = font.decode_data().unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn font_decode_data_zero_length_line() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["#0V%T", " "],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = font.decode_data().unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn font_decode_data_multiline() {
        // Multi-line UU-encoded data
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["$4F3\"", "$4F3\""],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = font.decode_data().unwrap();
        // Should decode both lines and concatenate results
        assert_eq!(decoded.len(), 6); // 3 bytes per line
    }

    #[test]
    fn graphic_decode_data_valid() {
        // Valid UU-encoded data for "PNG" (test with known encoding)
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec!["#4$Y'"],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = graphic.decode_data().unwrap();
        assert_eq!(decoded, b"PNG");
    }

    #[test]
    fn graphic_decode_data_empty_lines() {
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec![],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = graphic.decode_data().unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn graphic_decode_data_with_end_marker() {
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec!["#4$Y'", "end"],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = graphic.decode_data().unwrap();
        assert_eq!(decoded, b"PNG");
    }

    #[test]
    fn graphic_decode_data_whitespace_handling() {
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec!["#4$Y'  ", "\t\n", ""],
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = graphic.decode_data().unwrap();
        assert_eq!(decoded, b"PNG");
    }

    #[test]
    fn font_decode_data_handles_malformed_gracefully() {
        // UU decoding should not panic on malformed data but may return unexpected results
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["invalid-characters-here"],
            span: Span::new(0, 0, 0, 0),
        };
        // Should not panic, result depends on UU decoder implementation
        let _result = font.decode_data();
    }

    #[test]
    fn graphic_decode_data_handles_malformed_gracefully() {
        // UU decoding should not panic on malformed data but may return unexpected results
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec!["!@#$%^&*()"],
            span: Span::new(0, 0, 0, 0),
        };
        // Should not panic, result depends on UU decoder implementation
        let _result = graphic.decode_data();
    }

    #[test]
    fn font_decode_data_length_validation() {
        // Test that length encoding in first character is respected
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["!    "], // '!' encodes length 1, but provides more data
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = font.decode_data().unwrap();
        assert_eq!(decoded.len(), 1); // Should be truncated to declared length
    }

    #[test]
    fn graphic_decode_data_length_validation() {
        // Test that length encoding in first character is respected
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec!["\"````"], // '"' encodes length 2, provides padding
            span: Span::new(0, 0, 0, 0),
        };
        let decoded = graphic.decode_data().unwrap();
        assert_eq!(decoded.len(), 2); // Should be truncated to declared length
    }

    #[cfg(debug_assertions)]
    #[test]
    fn font_validate_spans() {
        let source = "fontname: test.ttf\ndata1\ndata2";
        let font = Font {
            filename: &source[10..18],                          // "test.ttf"
            data_lines: vec![&source[19..24], &source[25..30]], // "data1", "data2"
            span: Span::new(0, 0, 0, 0),
        };

        let source_range = (source.as_ptr() as usize)..(source.as_ptr() as usize + source.len());
        assert!(font.validate_spans(&source_range));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn graphic_validate_spans() {
        let source = "filename: logo.png\nimage1\nimage2";
        let graphic = Graphic {
            filename: &source[10..18],                          // "logo.png"
            data_lines: vec![&source[19..25], &source[26..32]], // "image1", "image2"
            span: Span::new(0, 0, 0, 0),
        };

        let source_range = (source.as_ptr() as usize)..(source.as_ptr() as usize + source.len());
        assert!(graphic.validate_spans(&source_range));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn font_validate_spans_invalid() {
        let source1 = "fontname: test.ttf";
        let source2 = "different source";

        let font = Font {
            filename: &source1[10..18],       // "test.ttf" from source1
            data_lines: vec![&source2[0..9]], // "different" from source2
            span: Span::new(0, 0, 0, 0),
        };

        let source1_range =
            (source1.as_ptr() as usize)..(source1.as_ptr() as usize + source1.len());
        assert!(!font.validate_spans(&source1_range)); // Should fail because data_lines reference different source
    }
}
