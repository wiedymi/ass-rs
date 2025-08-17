//! Line break handling with type preservation
//!
//! This module provides enhanced line break processing that preserves
//! the distinction between hard (\N) and soft (\n) line breaks.

use alloc::{string::String, vec::Vec};

/// Type of line break in ASS text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineBreakType {
    /// Hard line break (\N) - forces a new line
    Hard,
    /// Soft line break (\n) - allows wrapping
    Soft,
}

/// Line break information in processed text
#[derive(Debug, Clone)]
pub struct LineBreakInfo {
    /// Position in the plain text where the break occurs
    pub position: usize,
    /// Type of line break
    pub break_type: LineBreakType,
}

/// Enhanced text with line break preservation
#[derive(Debug, Clone)]
pub struct TextWithLineBreaks {
    /// Plain text with line breaks converted to newlines
    pub text: String,
    /// Information about each line break's type and position
    pub line_breaks: Vec<LineBreakInfo>,
    /// Non-breaking space positions
    pub nbsp_positions: Vec<usize>,
}

impl TextWithLineBreaks {
    /// Process text preserving line break types
    #[must_use]
    pub fn from_text(text: &str, drawing_mode: bool) -> Self {
        let mut plain_text = String::new();
        let mut line_breaks = Vec::new();
        let mut nbsp_positions = Vec::new();

        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'N' => {
                            chars.next(); // consume 'N'
                            if !drawing_mode {
                                line_breaks.push(LineBreakInfo {
                                    position: plain_text.len(),
                                    break_type: LineBreakType::Hard,
                                });
                                plain_text.push('\n');
                            }
                        }
                        'n' => {
                            chars.next(); // consume 'n'
                            if !drawing_mode {
                                line_breaks.push(LineBreakInfo {
                                    position: plain_text.len(),
                                    break_type: LineBreakType::Soft,
                                });
                                plain_text.push('\n');
                            }
                        }
                        'h' => {
                            chars.next(); // consume 'h'
                            if !drawing_mode {
                                nbsp_positions.push(plain_text.len());
                                plain_text.push('\u{00A0}'); // Non-breaking space
                            }
                        }
                        _ => {
                            // Not a special sequence, keep both characters
                            plain_text.push(ch);
                            plain_text.push(next_ch);
                            chars.next();
                        }
                    }
                } else {
                    // Backslash at end of string
                    plain_text.push(ch);
                }
            } else {
                plain_text.push(ch);
            }
        }

        Self {
            text: plain_text,
            line_breaks,
            nbsp_positions,
        }
    }

    /// Get the type of line break at a given position
    #[must_use]
    pub fn get_break_type_at(&self, position: usize) -> Option<LineBreakType> {
        self.line_breaks
            .iter()
            .find(|lb| lb.position == position)
            .map(|lb| lb.break_type)
    }

    /// Check if a position has a non-breaking space
    #[must_use]
    pub fn is_nbsp_at(&self, position: usize) -> bool {
        self.nbsp_positions.contains(&position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_line_break() {
        let text = r"Line 1\NLine 2";
        let processed = TextWithLineBreaks::from_text(text, false);

        assert_eq!(processed.text, "Line 1\nLine 2");
        assert_eq!(processed.line_breaks.len(), 1);
        assert_eq!(processed.line_breaks[0].break_type, LineBreakType::Hard);
        assert_eq!(processed.line_breaks[0].position, 6);
    }

    #[test]
    fn test_soft_line_break() {
        let text = r"Line 1\nLine 2";
        let processed = TextWithLineBreaks::from_text(text, false);

        assert_eq!(processed.text, "Line 1\nLine 2");
        assert_eq!(processed.line_breaks.len(), 1);
        assert_eq!(processed.line_breaks[0].break_type, LineBreakType::Soft);
        assert_eq!(processed.line_breaks[0].position, 6);
    }

    #[test]
    fn test_mixed_line_breaks() {
        let text = r"Line 1\NLine 2\nLine 3";
        let processed = TextWithLineBreaks::from_text(text, false);

        assert_eq!(processed.text, "Line 1\nLine 2\nLine 3");
        assert_eq!(processed.line_breaks.len(), 2);
        assert_eq!(processed.line_breaks[0].break_type, LineBreakType::Hard);
        assert_eq!(processed.line_breaks[1].break_type, LineBreakType::Soft);
    }

    #[test]
    fn test_non_breaking_space() {
        let text = r"Word1\hWord2";
        let processed = TextWithLineBreaks::from_text(text, false);

        assert_eq!(processed.text, "Word1\u{00A0}Word2");
        assert_eq!(processed.nbsp_positions.len(), 1);
        assert_eq!(processed.nbsp_positions[0], 5);
    }

    #[test]
    fn test_drawing_mode_ignores_special() {
        let text = r"Draw\NCommands\nHere\hIgnored";
        let processed = TextWithLineBreaks::from_text(text, true);

        assert_eq!(processed.text, "DrawCommandsHereIgnored");
        assert_eq!(processed.line_breaks.len(), 0);
        assert_eq!(processed.nbsp_positions.len(), 0);
    }
}
