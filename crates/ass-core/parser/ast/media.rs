//! Media AST nodes for embedded fonts and graphics
//!
//! Contains Font and Graphic structs representing embedded media from the
//! [Fonts] and [Graphics] sections with zero-copy design and UU-decoding.

use alloc::vec::Vec;
#[cfg(debug_assertions)]
use core::ops::Range;

/// Embedded font from [Fonts] section
///
/// Represents a font file embedded in the ASS script using UU-encoding.
/// Provides lazy decoding to avoid processing overhead unless the font
/// data is actually needed.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::Font;
///
/// let font = Font {
///     filename: "custom.ttf",
///     data_lines: vec!["begin 644 custom.ttf", "M'XL..."],
/// };
///
/// // Decode when needed
/// let decoded = font.decode_data()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Font<'a> {
    /// Font filename as it appears in the [Fonts] section
    pub filename: &'a str,

    /// UU-encoded font data lines as zero-copy spans
    pub data_lines: Vec<&'a str>,
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
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::Font;
    /// # let font = Font { filename: "test.ttf", data_lines: vec![] };
    /// match font.decode_data() {
    ///     Ok(data) => println!("Font size: {} bytes", data.len()),
    ///     Err(e) => eprintln!("Decode error: {}", e),
    /// }
    /// ```
    pub fn decode_data(&self) -> Result<Vec<u8>, crate::utils::CoreError> {
        crate::utils::decode_uu_data(self.data_lines.iter().copied())
    }

    /// Validate all spans in this Font reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that filename and all data line references point to
    /// memory within the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
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

/// Embedded graphic from [Graphics] section
///
/// Represents an image file embedded in the ASS script using UU-encoding.
/// Commonly used for logos, textures, and other graphical elements.
/// Provides lazy decoding for performance.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::Graphic;
///
/// let graphic = Graphic {
///     filename: "logo.png",
///     data_lines: vec!["begin 644 logo.png", "M89PNG..."],
/// };
///
/// let decoded = graphic.decode_data()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Graphic<'a> {
    /// Graphic filename as it appears in the [Graphics] section
    pub filename: &'a str,

    /// UU-encoded graphic data lines as zero-copy spans
    pub data_lines: Vec<&'a str>,
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
    pub fn decode_data(&self) -> Result<Vec<u8>, crate::utils::CoreError> {
        crate::utils::decode_uu_data(self.data_lines.iter().copied())
    }

    /// Validate all spans in this Graphic reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that filename and all data line references point to
    /// memory within the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
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

    #[test]
    fn font_creation() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["line1", "line2"],
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
        };

        let cloned = font.clone();
        assert_eq!(font, cloned);
    }

    #[test]
    fn graphic_clone_eq() {
        let graphic = Graphic {
            filename: "test.png",
            data_lines: vec!["data"],
        };

        let cloned = graphic.clone();
        assert_eq!(graphic, cloned);
    }

    #[test]
    fn font_debug() {
        let font = Font {
            filename: "debug.ttf",
            data_lines: vec!["test"],
        };

        let debug_str = format!("{:?}", font);
        assert!(debug_str.contains("Font"));
        assert!(debug_str.contains("debug.ttf"));
    }

    #[test]
    fn graphic_debug() {
        let graphic = Graphic {
            filename: "debug.png",
            data_lines: vec!["test"],
        };

        let debug_str = format!("{:?}", graphic);
        assert!(debug_str.contains("Graphic"));
        assert!(debug_str.contains("debug.png"));
    }

    #[test]
    fn empty_data_lines() {
        let font = Font {
            filename: "empty.ttf",
            data_lines: Vec::new(),
        };

        let graphic = Graphic {
            filename: "empty.png",
            data_lines: Vec::new(),
        };

        assert!(font.data_lines.is_empty());
        assert!(graphic.data_lines.is_empty());
    }

    #[test]
    fn media_inequality() {
        let font1 = Font {
            filename: "font1.ttf",
            data_lines: vec!["data"],
        };

        let font2 = Font {
            filename: "font2.ttf",
            data_lines: vec!["data"],
        };

        assert_ne!(font1, font2);
    }
}
