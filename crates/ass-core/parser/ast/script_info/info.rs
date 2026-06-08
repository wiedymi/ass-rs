//! `ScriptInfo` struct definition and metadata accessors
//!
//! Defines the zero-copy [`ScriptInfo`] AST node for the [Script Info]
//! section along with its field accessors and ASS serialization helper.

use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

use super::Span;
#[cfg(debug_assertions)]
use core::ops::Range;

/// Script Info section containing metadata and headers
///
/// Represents the [Script Info] section of an ASS file as key-value pairs
/// with zero-copy string references. Provides convenient accessor methods
/// for standard ASS metadata fields.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::{ScriptInfo, Span};
///
/// let fields = vec![("Title", "Test Script"), ("ScriptType", "v4.00+")];
/// let info = ScriptInfo { fields, span: Span::new(0, 0, 0, 0) };
///
/// assert_eq!(info.title(), "Test Script");
/// assert_eq!(info.script_type(), Some("v4.00+"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ScriptInfo<'a> {
    /// Key-value pairs as zero-copy spans
    pub fields: Vec<(&'a str, &'a str)>,
    /// Span in source text where this script info section is defined
    pub span: Span,
}

impl<'a> ScriptInfo<'a> {
    /// Get field value by key (case-sensitive)
    ///
    /// Searches for the specified key in the script info fields and
    /// returns the associated value if found.
    ///
    /// # Arguments
    ///
    /// * `key` - Field name to search for
    ///
    /// # Returns
    ///
    /// The field value if found, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{ScriptInfo, Span};
    /// let fields = vec![("Title", "Test"), ("Author", "User")];
    /// let info = ScriptInfo { fields, span: Span::new(0, 0, 0, 0) };
    ///
    /// assert_eq!(info.get_field("Title"), Some("Test"));
    /// assert_eq!(info.get_field("Unknown"), None);
    /// ```
    #[must_use]
    pub fn get_field(&self, key: &str) -> Option<&'a str> {
        self.fields.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    /// Get script title, defaulting to `<untitled>`
    ///
    /// Returns the "Title" field value or a default if not specified.
    /// This is a convenience method for the most commonly accessed field.
    #[must_use]
    pub fn title(&self) -> &str {
        self.get_field("Title").unwrap_or("<untitled>")
    }

    /// Get script type version
    ///
    /// Returns the "`ScriptType`" field which indicates the ASS version
    /// and feature compatibility (e.g., "v4.00+", "v4.00").
    #[must_use]
    pub fn script_type(&self) -> Option<&'a str> {
        self.get_field("ScriptType")
    }

    /// Get play resolution as (width, height)
    ///
    /// Parses `PlayResX` and `PlayResY` fields to determine the intended
    /// video resolution for subtitle rendering.
    ///
    /// # Returns
    ///
    /// Tuple of (width, height) if both fields are present and valid,
    /// `None` if either field is missing or invalid.
    #[must_use]
    pub fn play_resolution(&self) -> Option<(u32, u32)> {
        let width = self.get_field("PlayResX")?.parse().ok()?;
        let height = self.get_field("PlayResY")?.parse().ok()?;
        Some((width, height))
    }

    /// Get layout resolution as (width, height)
    ///
    /// Layout resolution defines the coordinate system for positioning and scaling
    /// subtitles relative to the video resolution. Used by style analysis for
    /// proper layout calculations.
    ///
    /// # Returns
    ///
    /// Tuple of (width, height) if both fields are present and valid,
    /// `None` if either field is missing or invalid.
    #[must_use]
    pub fn layout_resolution(&self) -> Option<(u32, u32)> {
        let width = self.get_field("LayoutResX")?.parse().ok()?;
        let height = self.get_field("LayoutResY")?.parse().ok()?;
        Some((width, height))
    }

    /// Get wrap style setting
    ///
    /// Returns the `WrapStyle` field which controls how long lines are wrapped.
    /// Defaults to 0 (smart wrapping) if not specified.
    ///
    /// # Wrap Styles
    ///
    /// - 0: Smart wrapping (default)
    /// - 1: End-of-line wrapping
    /// - 2: No wrapping
    /// - 3: Smart wrapping with lower line longer
    #[must_use]
    pub fn wrap_style(&self) -> u8 {
        self.get_field("WrapStyle")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Convert script info to ASS string representation
    ///
    /// Generates the [Script Info] section with all fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{ScriptInfo, Span};
    /// let fields = vec![("Title", "Test Script"), ("ScriptType", "v4.00+")];
    /// let info = ScriptInfo { fields, span: Span::new(0, 0, 0, 0) };
    /// let ass_string = info.to_ass_string();
    /// assert!(ass_string.contains("[Script Info]"));
    /// assert!(ass_string.contains("Title: Test Script"));
    /// assert!(ass_string.contains("ScriptType: v4.00+"));
    /// ```
    #[must_use]
    pub fn to_ass_string(&self) -> alloc::string::String {
        use core::fmt::Write;
        let mut result = alloc::string::String::from("[Script Info]\n");
        for (key, value) in &self.fields {
            let _ = writeln!(result, "{key}: {value}");
        }
        result
    }

    /// Validate all spans in this `ScriptInfo` reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references point to memory within
    /// the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        self.fields.iter().all(|(key, value)| {
            let key_ptr = key.as_ptr() as usize;
            let value_ptr = value.as_ptr() as usize;
            source_range.contains(&key_ptr) && source_range.contains(&value_ptr)
        })
    }
}
