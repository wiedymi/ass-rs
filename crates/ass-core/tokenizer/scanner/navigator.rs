//! Character navigation helper for the ASS tokenizer.
//!
//! Provides character-level navigation with position tracking and lookahead
//! capabilities for efficient tokenization of ASS subtitle scripts.

use crate::{utils::CoreError, Result};
use alloc::{format, string::ToString};
use core::str::Chars;

#[cfg(not(feature = "std"))]
extern crate alloc;

/// Character navigation helper for tokenizer
///
/// Provides character-level navigation with position tracking and
/// lookahead capabilities for efficient tokenization.
#[derive(Debug, Clone)]
pub struct CharNavigator<'a> {
    /// Source text being scanned
    source: &'a str,
    /// Current byte position in source
    pub(super) position: usize,
    /// Current line number (1-based)
    line: usize,
    /// Current column number (1-based)
    column: usize,
    /// Character iterator for the source
    pub(super) chars: Chars<'a>,
    /// Lookahead character for peeking
    pub(super) peek_char: Option<char>,
    /// Last character processed (for \r\n handling)
    pub(super) last_char: Option<char>,
}

impl<'a> CharNavigator<'a> {
    /// Create new character navigator
    #[must_use]
    pub fn new(source: &'a str, position: usize, line: usize, column: usize) -> Self {
        Self {
            source,
            position,
            line,
            column,
            chars: source[position..].chars(),
            peek_char: None,
            last_char: None,
        }
    }

    /// Get current position
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Get current line
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Get current column
    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Peek at current character without advancing
    ///
    /// # Errors
    ///
    /// Returns an error if the current position contains invalid UTF-8 or is at end of input.
    pub fn peek_char(&mut self) -> Result<char> {
        if let Some(ch) = self.peek_char {
            Ok(ch)
        } else if self.position < self.source.len() {
            let ch = self.source[self.position..].chars().next().ok_or_else(|| {
                CoreError::parse(format!("Invalid UTF-8 at position {}", self.position))
            })?;
            self.peek_char = Some(ch);
            Ok(ch)
        } else {
            Err(CoreError::parse("Unexpected end of input".to_string()))
        }
    }

    /// Peek at next character without advancing
    ///
    /// # Errors
    ///
    /// Returns an error if the next position is at end of input.
    pub fn peek_next(&self) -> Result<char> {
        let mut chars = self.source[self.position..].chars();
        chars.next(); // Skip current
        chars
            .next()
            .ok_or_else(|| CoreError::parse("Unexpected end of input".to_string()))
    }

    /// Advance by one character
    ///
    /// # Errors
    ///
    /// Returns an error if unable to peek at the current character.
    pub fn advance_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.peek_char = None;

        let _ = self.chars.next();
        self.position += ch.len_utf8();

        match ch {
            '\r' => {
                self.line += 1;
                self.column = 1;
            }
            '\n' => {
                // Only increment line if previous char wasn't \r (to handle \r\n properly)
                if self.last_char != Some('\r') {
                    self.line += 1;
                }
                self.column = 1;
            }
            _ => {
                self.column += 1;
            }
        }

        self.last_char = Some(ch);
        Ok(ch)
    }

    /// Skip whitespace (excluding newlines)
    pub fn skip_whitespace(&mut self) {
        while self.position < self.source.len() {
            if let Ok(ch) = self.peek_char() {
                if ch.is_whitespace() && ch != '\n' && ch != '\r' {
                    let _ = self.advance_char();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Check if at end of source
    #[must_use]
    pub const fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }
}
