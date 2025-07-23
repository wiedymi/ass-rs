//! Token scanning methods for ASS tokenizer
//!
//! Provides specialized scanning functions for different ASS script elements
//! including section headers, style overrides, comments, and text content.

use crate::{utils::CoreError, Result};
use alloc::format;
use core::str::Chars;

#[cfg(feature = "simd")]
use super::simd;
use super::{state::TokenContext, tokens::TokenType};

/// Character navigation helper for tokenizer
///
/// Provides character-level navigation with position tracking and
/// lookahead capabilities for efficient tokenization.
#[derive(Debug, Clone)]
pub struct CharNavigator<'a> {
    /// Source text being scanned
    source: &'a str,
    /// Current byte position in source
    position: usize,
    /// Current line number (1-based)
    line: usize,
    /// Current column number (1-based)
    column: usize,
    /// Character iterator for the source
    chars: Chars<'a>,
    /// Lookahead character for peeking
    peek_char: Option<char>,
    /// Last character processed (for \r\n handling)
    last_char: Option<char>,
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

/// Scanner for different token types
#[derive(Debug, Clone)]
pub struct TokenScanner<'a> {
    /// Character navigator for position tracking
    navigator: CharNavigator<'a>,
    /// Source text reference
    source: &'a str,
}

impl<'a> TokenScanner<'a> {
    /// Create new token scanner
    #[must_use]
    pub fn new(source: &'a str, position: usize, line: usize, column: usize) -> Self {
        Self {
            navigator: CharNavigator::new(source, position, line, column),
            source,
        }
    }

    /// Get current navigator state (mutable)
    pub fn navigator_mut(&mut self) -> &mut CharNavigator<'a> {
        &mut self.navigator
    }

    /// Get current navigator state (immutable)
    #[must_use]
    pub const fn navigator(&self) -> &CharNavigator<'a> {
        &self.navigator
    }

    /// Scan section header like [Script Info]
    ///
    /// # Errors
    ///
    /// Returns an error if character navigation fails.
    pub fn scan_section_header(&mut self) -> Result<TokenType> {
        self.navigator.advance_char()?; // Skip '['

        while !self.navigator.is_at_end() {
            let ch = self.navigator.peek_char()?;
            if ch == ']' {
                break;
            }
            self.navigator.advance_char()?;
        }

        Ok(TokenType::SectionHeader)
    }

    /// Scan style override block like {\b1\i1}
    ///
    /// # Errors
    ///
    /// Returns an error if character navigation fails.
    pub fn scan_style_override(&mut self) -> Result<TokenType> {
        self.navigator.advance_char()?; // Skip '{'

        let mut brace_depth = 1;
        while !self.navigator.is_at_end() && brace_depth > 0 {
            let ch = self.navigator.peek_char()?;
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }

            if brace_depth > 0 {
                self.navigator.advance_char()?;
            }
        }

        Ok(TokenType::OverrideBlock)
    }

    /// Scan comment line starting with ; or !:
    ///
    /// # Errors
    ///
    /// Returns an error if character navigation fails.
    pub fn scan_comment(&mut self) -> Result<TokenType> {
        while !self.navigator.is_at_end() {
            let ch = self.navigator.peek_char()?;
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.navigator.advance_char()?;
        }

        Ok(TokenType::Comment)
    }

    /// Scan general text content
    ///
    /// # Errors
    ///
    /// Returns an error if character navigation fails.
    pub fn scan_text(&mut self, context: TokenContext) -> Result<TokenType> {
        let start = self.navigator.position();

        // Use SIMD delimiter scanning when available
        #[cfg(feature = "simd")]
        {
            if let Some(delimiter_pos) = self.scan_delimiters_simd(start) {
                self.navigator.position = delimiter_pos;
            } else {
                self.navigator.position = self.source.len();
            }
            self.navigator.chars = self.source[self.navigator.position..].chars();
            self.navigator.peek_char = None;
        }

        // Fallback to scalar scanning
        #[cfg(not(feature = "simd"))]
        {
            while !self.navigator.is_at_end() {
                let ch = self.navigator.peek_char()?;

                // Check for delimiters based on context
                let is_delimiter = match context {
                    TokenContext::FieldValue => {
                        // In field values, don't treat colon as delimiter (for time formats)
                        matches!(ch, ',' | '{' | '}' | '[' | ']' | '\n' | '\r')
                    }
                    _ => {
                        // In other contexts, treat colon as delimiter
                        matches!(ch, ',' | ':' | '{' | '}' | '[' | ']' | '\n' | '\r')
                            || (ch == ';' && context == TokenContext::Document)
                    }
                };

                if is_delimiter {
                    break;
                }

                self.navigator.advance_char()?;
            }
        }

        let span = &self.source[start..self.navigator.position()];

        if context == TokenContext::SectionHeader {
            Ok(TokenType::SectionName)
        } else if Self::is_hex_value(span) {
            Ok(TokenType::HexValue)
        } else if !span.is_empty()
            && span
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        {
            Ok(TokenType::Number)
        } else {
            Ok(TokenType::Text)
        }
    }

    /// Check if a span represents a hex value
    fn is_hex_value(span: &str) -> bool {
        #[cfg(feature = "simd")]
        {
            if span.chars().all(|c| c.is_ascii_hexdigit())
                && span.len() % 2 == 0
                && !span.is_empty()
            {
                return TokenScanner::parse_hex_simd(span).is_some();
            }

            if let Some(after_prefix) = span.strip_prefix("&H") {
                let hex_part = after_prefix
                    .strip_suffix('&')
                    .map_or(after_prefix, |stripped| stripped);

                return !hex_part.is_empty()
                    && hex_part.len() % 2 == 0
                    && TokenScanner::parse_hex_simd(hex_part).is_some();
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            if span.chars().all(|c| c.is_ascii_hexdigit())
                && span.len() % 2 == 0
                && !span.is_empty()
            {
                return true;
            }

            if let Some(after_prefix) = span.strip_prefix("&H") {
                let hex_part = if let Some(stripped) = after_prefix.strip_suffix('&') {
                    stripped
                } else {
                    after_prefix
                };

                return !hex_part.is_empty()
                    && hex_part.chars().all(|c| c.is_ascii_hexdigit())
                    && hex_part.len() % 2 == 0;
            }
        }

        false
    }

    /// Fast delimiter scanning using SIMD when available
    #[cfg(feature = "simd")]
    fn scan_delimiters_simd(&self, start: usize) -> Option<usize> {
        simd::scan_delimiters(&self.source[start..]).map(|offset| start + offset)
    }

    /// Fast hex parsing using SIMD when available
    #[cfg(feature = "simd")]
    fn parse_hex_simd(hex_str: &str) -> Option<u32> {
        simd::parse_hex_u32(hex_str)
    }

    /// Scan field value content in field value context
    ///
    /// In field value context, colons are not delimiters (for time formats)
    /// and we consume until comma, newline, or end of input.
    ///
    /// # Errors
    ///
    /// Returns an error if character navigation fails.
    pub fn scan_field_value(&mut self) -> Result<TokenType> {
        let start = self.navigator.position();

        while !self.navigator.is_at_end() {
            let ch = self.navigator.peek_char()?;

            // Stop at delimiters that end field values
            if ch == ',' || ch == '\n' || ch == '\r' {
                break;
            }

            self.navigator.advance_char()?;
        }

        let span = &self.source[start..self.navigator.position()];

        if span
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == ':')
        {
            Ok(TokenType::Number)
        } else {
            Ok(TokenType::Text)
        }
    }
}

#[cfg(test)]
mod tests;
