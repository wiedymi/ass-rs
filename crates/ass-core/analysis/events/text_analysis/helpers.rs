//! Accessors and Unicode helpers for [`TextAnalysis`].
//!
//! Provides the read-only getters over the analysis result plus the internal
//! helpers used by the parser for drawing-mode tracking, line counting, and
//! Unicode complexity detection.

use super::TextAnalysis;
use crate::analysis::events::tags::{OverrideTag, TagDiagnostic};
use alloc::{string::String, vec::Vec};

impl<'a> TextAnalysis<'a> {
    /// Get plain text without override tags
    #[must_use]
    pub fn plain_text(&self) -> &str {
        &self.plain_text
    }

    /// Get Unicode character count
    #[must_use]
    pub const fn char_count(&self) -> usize {
        self.char_count
    }

    /// Get line count after processing linebreaks
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Check if text contains bidirectional content
    #[must_use]
    pub const fn has_bidi_text(&self) -> bool {
        self.has_bidi_text
    }

    /// Check if text contains complex Unicode beyond basic Latin
    #[must_use]
    pub const fn has_complex_unicode(&self) -> bool {
        self.has_complex_unicode
    }

    /// Get parsed override tags
    #[must_use]
    pub fn override_tags(&self) -> &[OverrideTag<'a>] {
        &self.override_tags
    }

    /// Get parse diagnostics collected during analysis
    #[must_use]
    pub fn diagnostics(&self) -> &[TagDiagnostic<'a>] {
        &self.parse_diagnostics
    }

    /// Update drawing mode state based on override tag content
    pub(super) fn update_drawing_mode(tag_content: &str, current_mode: bool) -> bool {
        let mut pos = 0;
        let chars: Vec<char> = tag_content.chars().collect();
        let mut drawing_mode = current_mode;

        while pos < chars.len() {
            if chars[pos] == '\\' && pos + 1 < chars.len() && chars[pos + 1] == 'p' {
                pos += 2;
                let mut number_str = String::new();

                while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '-') {
                    number_str.push(chars[pos]);
                    pos += 1;
                }

                if let Ok(p_value) = number_str.parse::<i32>() {
                    drawing_mode = p_value > 0;
                }
            } else {
                pos += 1;
            }
        }

        drawing_mode
    }

    /// Count lines correctly, handling empty lines and trailing newlines
    pub(super) fn count_lines(text: &str) -> usize {
        if text.is_empty() {
            return 1;
        }

        // For ASS subtitles, count newlines and add 1, but handle special cases
        let newline_count = text.chars().filter(|&ch| ch == '\n').count();

        if newline_count == 0 {
            // No newlines means 1 line
            1
        } else if text.trim_end_matches('\n').is_empty() {
            // Text is only newlines - each newline creates a line boundary
            newline_count + 1
        } else {
            // Text has content - trailing newlines don't create additional lines
            // Use lines() count which handles this correctly
            text.lines().count().max(1)
        }
    }

    /// Detect bidirectional text (RTL scripts)
    pub(super) fn detect_bidi_text(text: &str) -> bool {
        text.chars().any(|ch| matches!(ch as u32, 0x0590..=0x05FF | 0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF))
    }

    /// Detect complex Unicode beyond basic Latin
    pub(super) fn detect_complex_unicode(text: &str) -> bool {
        text.chars().any(|ch| {
            let code = ch as u32;
            code > 0x00FF || matches!(code, 0x0000..=0x001F | 0x007F..=0x009F | 0x200C..=0x200D | 0x2060..=0x206F)
        })
    }
}
