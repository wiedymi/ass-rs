//! Script Info AST node for ASS script metadata
//!
//! Contains the `ScriptInfo` struct representing the [Script Info] section
//! of ASS files with zero-copy design and convenient accessor methods
//! for common metadata fields.

use alloc::vec::Vec;
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
/// use ass_core::parser::ast::ScriptInfo;
///
/// let fields = vec![("Title", "Test Script"), ("ScriptType", "v4.00+")];
/// let info = ScriptInfo { fields };
///
/// assert_eq!(info.title(), "Test Script");
/// assert_eq!(info.script_type(), Some("v4.00+"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptInfo<'a> {
    /// Key-value pairs as zero-copy spans
    pub fields: Vec<(&'a str, &'a str)>,
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
    /// # use ass_core::parser::ast::ScriptInfo;
    /// let fields = vec![("Title", "Test"), ("Author", "User")];
    /// let info = ScriptInfo { fields };
    ///
    /// assert_eq!(info.get_field("Title"), Some("Test"));
    /// assert_eq!(info.get_field("Unknown"), None);
    /// ```
    #[must_use]
    pub fn get_field(&self, key: &str) -> Option<&'a str> {
        self.fields.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    /// Get script title, defaulting to "<untitled>"
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

    /// Validate all spans in this ScriptInfo reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references point to memory within
    /// the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        self.fields.iter().all(|(key, value)| {
            let key_ptr = key.as_ptr() as usize;
            let value_ptr = value.as_ptr() as usize;
            source_range.contains(&key_ptr) && source_range.contains(&value_ptr)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_info_field_access() {
        let fields = vec![("Title", "Test Script"), ("ScriptType", "v4.00+")];
        let info = ScriptInfo { fields };

        assert_eq!(info.title(), "Test Script");
        assert_eq!(info.script_type(), Some("v4.00+"));
        assert_eq!(info.get_field("Unknown"), None);
    }

    #[test]
    fn script_info_defaults() {
        let info = ScriptInfo { fields: Vec::new() };
        assert_eq!(info.title(), "<untitled>");
        assert_eq!(info.wrap_style(), 0);
        assert_eq!(info.layout_resolution(), None);
        assert_eq!(info.play_resolution(), None);
    }

    #[test]
    fn script_info_play_resolution() {
        let fields = vec![("PlayResX", "1920"), ("PlayResY", "1080")];
        let info = ScriptInfo { fields };
        assert_eq!(info.play_resolution(), Some((1920, 1080)));
    }

    #[test]
    fn script_info_partial_play_resolution() {
        let fields = vec![("PlayResX", "1920")];
        let info = ScriptInfo { fields };
        assert_eq!(info.play_resolution(), None);
    }

    #[test]
    fn script_info_layout_resolution() {
        let fields = vec![("LayoutResX", "1920"), ("LayoutResY", "1080")];
        let info = ScriptInfo { fields };
        assert_eq!(info.layout_resolution(), Some((1920, 1080)));
    }

    #[test]
    fn script_info_partial_layout_resolution() {
        let fields = vec![("LayoutResX", "1920")];
        let info = ScriptInfo { fields };
        assert_eq!(info.layout_resolution(), None);
    }

    #[test]
    fn script_info_wrap_style() {
        let fields = vec![("WrapStyle", "2")];
        let info = ScriptInfo { fields };
        assert_eq!(info.wrap_style(), 2);
    }

    #[test]
    fn script_info_invalid_wrap_style() {
        let fields = vec![("WrapStyle", "invalid")];
        let info = ScriptInfo { fields };
        assert_eq!(info.wrap_style(), 0); // Default fallback
    }

    #[test]
    fn script_info_invalid_resolution() {
        let fields = vec![("PlayResX", "invalid"), ("PlayResY", "1080")];
        let info = ScriptInfo { fields };
        assert_eq!(info.play_resolution(), None);
    }

    #[test]
    fn script_info_case_sensitive_keys() {
        let fields = vec![("title", "Test"), ("Title", "Correct")];
        let info = ScriptInfo { fields };
        assert_eq!(info.get_field("Title"), Some("Correct"));
        assert_eq!(info.get_field("title"), Some("Test"));
    }
}
