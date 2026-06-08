//! Source position tracking for streaming tokenization.
//!
//! Defines the [`TokenPosition`] cursor that tracks byte offset, line, and
//! column while advancing through source text one character at a time.

/// Token stream position for streaming tokenization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPosition {
    /// Byte offset in source
    pub offset: usize,

    /// Line number (1-based)
    pub line: usize,

    /// Column number (1-based)
    pub column: usize,
}

impl TokenPosition {
    /// Create new position
    #[must_use]
    pub const fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    /// Create position at start of input
    #[must_use]
    pub const fn start() -> Self {
        Self::new(0, 1, 1)
    }

    /// Advance position by one character
    #[must_use]
    pub const fn advance(mut self, ch: char) -> Self {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self
    }

    /// Advance position by string length
    #[must_use]
    pub fn advance_by_str(mut self, s: &str) -> Self {
        for ch in s.chars() {
            self = self.advance(ch);
        }
        self
    }
}

impl Default for TokenPosition {
    fn default() -> Self {
        Self::start()
    }
}
