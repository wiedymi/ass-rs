//! Cursor navigation and error-recovery helpers.
//!
//! Provides the low-level position-advancing routines used during parsing:
//! section boundary detection, line skipping, whitespace/comment skipping, and
//! error-recovery scanning that suggests likely section headers.

use super::Parser;
use alloc::string::{String, ToString};

impl Parser<'_> {
    /// Check if at start of next section
    pub(super) fn at_next_section(&self) -> bool {
        let remaining = self.source[self.position..].trim_start();
        if !remaining.starts_with('[') {
            return false;
        }

        // Check if this looks like a complete section header (has closing ])
        remaining.find('\n').map_or_else(
            || remaining.contains(']'),
            |line_end| remaining[..line_end].contains(']'),
        )
    }

    /// Skip to next line
    pub(super) fn skip_line(&mut self) {
        if let Some(newline_pos) = self.source[self.position..].find('\n') {
            self.position += newline_pos + 1;
            self.line += 1;
        } else {
            self.position = self.source.len();
        }
    }

    /// Skip whitespace and comment lines
    pub(super) fn skip_whitespace_and_comments(&mut self) {
        while self.position < self.source.len() {
            let remaining = &self.source[self.position..];
            let trimmed = remaining.trim_start();

            if trimmed.starts_with(';') || trimmed.starts_with("!:") {
                self.skip_line();
            } else if trimmed != remaining {
                self.position += remaining.len() - trimmed.len();
            } else {
                break;
            }
        }
    }

    /// Skip to next section for error recovery
    pub(super) fn skip_to_next_section(&mut self) -> Option<String> {
        let mut suggestion = None;
        let start_position = self.position;

        while self.position < self.source.len() {
            if self.at_next_section() {
                break;
            }

            // Look for patterns that suggest what section this might be
            let line_start = self.position;
            let line_end = self.source[self.position..]
                .find('\n')
                .map_or(self.source.len(), |i| self.position + i);

            if line_end > line_start {
                let line = &self.source[line_start..line_end];

                // Check for common section entry patterns
                if suggestion.is_none() {
                    if line.trim_start().starts_with("Style:") {
                        suggestion = Some("Did you mean '[V4+ Styles]'?".to_string());
                    } else if line.trim_start().starts_with("Dialogue:")
                        || line.trim_start().starts_with("Comment:")
                    {
                        suggestion = Some("Did you mean '[Events]'?".to_string());
                    } else if line.trim_start().starts_with("Title:")
                        || line.trim_start().starts_with("ScriptType:")
                    {
                        suggestion = Some("Did you mean '[Script Info]'?".to_string());
                    } else if line.trim_start().starts_with("Format:") {
                        // Format lines could be in styles or events
                        let remaining = &self.source[self.position..];
                        if remaining.contains("Dialogue:") {
                            suggestion = Some("Did you mean '[Events]'?".to_string());
                        } else if remaining.contains("Style:") {
                            suggestion = Some("Did you mean '[V4+ Styles]'?".to_string());
                        }
                    }
                }
            }

            self.skip_line();

            // Prevent infinite loop: if we haven't advanced, force advance by one character
            if self.position == start_position {
                self.position = (self.position + 1).min(self.source.len());
                break;
            }
        }

        suggestion
    }
}
