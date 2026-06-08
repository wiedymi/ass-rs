//! String conversion and span-validation methods for the `Style` node.
//!
//! Provides ASS string serialization helpers (`to_ass_string`,
//! `to_ass_string_with_format`) and the debug-only `validate_spans` invariant
//! check for the zero-copy `Style` struct.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

use super::Style;
#[cfg(debug_assertions)]
use core::ops::Range;

impl Style<'_> {
    /// Convert style to ASS string representation
    ///
    /// Generates the standard ASS style line format for V4+ styles.
    /// Uses `margin_v` by default, but will use `margin_t/margin_b` if provided (V4++ format).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::Style;
    /// let style = Style {
    ///     name: "TestStyle",
    ///     fontname: "Arial",
    ///     fontsize: "20",
    ///     ..Style::default()
    /// };
    /// let ass_string = style.to_ass_string();
    /// assert!(ass_string.starts_with("Style: TestStyle,Arial,20,"));
    /// ```
    #[must_use]
    pub fn to_ass_string(&self) -> alloc::string::String {
        // Use standard V4+ format by default
        // TODO: Support custom format lines
        format!(
            "Style: {},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.name,
            self.fontname,
            self.fontsize,
            self.primary_colour,
            self.secondary_colour,
            self.outline_colour,
            self.back_colour,
            self.bold,
            self.italic,
            self.underline,
            self.strikeout,
            self.scale_x,
            self.scale_y,
            self.spacing,
            self.angle,
            self.border_style,
            self.outline,
            self.shadow,
            self.alignment,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.encoding
        )
    }

    /// Convert style to ASS string with specific format
    ///
    /// Generates an ASS style line according to the provided format specification.
    /// This allows handling both V4+ and V4++ formats, as well as custom formats.
    ///
    /// # Arguments
    ///
    /// * `format` - Field names in order (e.g., ["Name", "Fontname", "Fontsize", ...])
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::Style;
    /// let style = Style {
    ///     name: "Simple",
    ///     fontname: "Arial",
    ///     fontsize: "16",
    ///     ..Style::default()
    /// };
    /// let format = vec!["Name", "Fontname", "Fontsize"];
    /// assert_eq!(
    ///     style.to_ass_string_with_format(&format),
    ///     "Style: Simple,Arial,16"
    /// );
    /// ```
    #[must_use]
    pub fn to_ass_string_with_format(&self, format: &[&str]) -> alloc::string::String {
        let mut field_values = Vec::with_capacity(format.len());

        for field in format {
            let value = match *field {
                "Name" => self.name,
                "Fontname" => self.fontname,
                "Fontsize" => self.fontsize,
                "PrimaryColour" => self.primary_colour,
                "SecondaryColour" => self.secondary_colour,
                "OutlineColour" | "TertiaryColour" => self.outline_colour,
                "BackColour" => self.back_colour,
                "Bold" => self.bold,
                "Italic" => self.italic,
                "Underline" => self.underline,
                "Strikeout" | "StrikeOut" => self.strikeout,
                "ScaleX" => self.scale_x,
                "ScaleY" => self.scale_y,
                "Spacing" => self.spacing,
                "Angle" => self.angle,
                "BorderStyle" => self.border_style,
                "Outline" => self.outline,
                "Shadow" => self.shadow,
                "Alignment" => self.alignment,
                "MarginL" => self.margin_l,
                "MarginR" => self.margin_r,
                "MarginV" => self.margin_v,
                "MarginT" => self.margin_t.unwrap_or("0"),
                "MarginB" => self.margin_b.unwrap_or("0"),
                "Encoding" => self.encoding,
                "RelativeTo" => self.relative_to.unwrap_or("0"),
                _ => "", // Unknown fields default to empty
            };
            field_values.push(value);
        }

        let joined = field_values.join(",");
        format!("Style: {joined}")
    }

    /// Validate all spans in this Style reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references point to memory within
    /// the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let spans = [
            self.name,
            self.fontname,
            self.fontsize,
            self.primary_colour,
            self.secondary_colour,
            self.outline_colour,
            self.back_colour,
            self.bold,
            self.italic,
            self.underline,
            self.strikeout,
            self.scale_x,
            self.scale_y,
            self.spacing,
            self.angle,
            self.border_style,
            self.outline,
            self.shadow,
            self.alignment,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.encoding,
        ];

        spans.iter().all(|span| {
            let ptr = span.as_ptr() as usize;
            source_range.contains(&ptr)
        })
    }
}
