//! Generic UU-encoded binary data parser for `[Fonts]` and `[Graphics]`.
//!
//! Defines [`BinaryDataParser`], the position-tracked engine shared by the
//! font and graphic section parsers.

use alloc::vec::Vec;

use crate::parser::{ast::Span, position_tracker::PositionTracker};

/// Generic parser for binary data sections (`[Fonts\]` and `[Graphics]`)
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
