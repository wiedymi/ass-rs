//! Zero-copy span utilities for AST references.
//!
//! Provides [`Spans`], a helper for validating and locating string slices that
//! reference the original source text while maintaining zero-copy semantics.

use core::ops::Range;

/// Zero-copy span utilities for AST node validation and manipulation
///
/// Provides safe methods to work with string slices that reference
/// the original source text, maintaining zero-copy semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spans<'a> {
    /// Reference to the original source text
    source: &'a str,
}

impl<'a> Spans<'a> {
    /// Create new span utilities for source text
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self { source }
    }

    /// Validate that a span references this source text
    ///
    /// Returns `true` if the span is a valid substring of the source.
    /// Used for debug assertions to ensure zero-copy invariants.
    #[must_use]
    pub fn validate_span(&self, span: &str) -> bool {
        let source_start = self.source.as_ptr() as usize;
        let source_end = source_start + self.source.len();

        let span_start = span.as_ptr() as usize;
        let span_end = span_start + span.len();

        span_start >= source_start && span_end <= source_end
    }

    /// Get byte offset of span within source
    #[must_use]
    pub fn span_offset(&self, span: &str) -> Option<usize> {
        let source_start = self.source.as_ptr() as usize;
        let span_start = span.as_ptr() as usize;

        if self.validate_span(span) {
            Some(span_start - source_start)
        } else {
            None
        }
    }

    /// Get line number (1-based) for a span
    #[must_use]
    pub fn span_line(&self, span: &str) -> Option<usize> {
        let offset = self.span_offset(span)?;
        Some(self.source[..offset].chars().filter(|&c| c == '\n').count() + 1)
    }

    /// Get column number (1-based) for a span
    #[must_use]
    pub fn span_column(&self, span: &str) -> Option<usize> {
        let offset = self.span_offset(span)?;
        let line_start = self.source[..offset].rfind('\n').map_or(0, |pos| pos + 1);

        Some(self.source[line_start..offset].chars().count() + 1)
    }

    /// Extract substring by byte range
    #[must_use]
    pub fn substring(&self, range: Range<usize>) -> Option<&'a str> {
        self.source.get(range)
    }
}
