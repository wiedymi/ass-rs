//! Embedded graphic AST node for the `[Graphics]` section.
//!
//! Defines the [`Graphic`] struct with lazy UU-decoding helpers and zero-copy
//! spans over the original ASS source text.

#[cfg(not(feature = "std"))]
use alloc::format;
use alloc::vec::Vec;

use super::super::Span;
#[cfg(debug_assertions)]
use core::ops::Range;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
