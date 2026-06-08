//! Embedded font AST node for the `[Fonts]` section.
//!
//! Defines the [`Font`] struct with lazy UU-decoding helpers and zero-copy
//! spans over the original ASS source text.

#[cfg(not(feature = "std"))]
use alloc::format;
use alloc::vec::Vec;

use super::super::Span;
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
