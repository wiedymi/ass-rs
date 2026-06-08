//! Token scanner type and its primary scanning routines.
//!
//! Defines the [`TokenScanner`] type and the scanning methods for section
//! headers, style override blocks, and comment lines.

use super::navigator::CharNavigator;
use crate::tokenizer::tokens::TokenType;
use crate::Result;

/// Scanner for different token types
#[derive(Debug, Clone)]
pub struct TokenScanner<'a> {
    /// Character navigator for position tracking
    pub(super) navigator: CharNavigator<'a>,
    /// Source text reference
    pub(super) source: &'a str,
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
}
