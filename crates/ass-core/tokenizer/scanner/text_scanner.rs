//! Text and field-value scanning routines for the token scanner.
//!
//! Implements general text scanning, field-value scanning, and hex-value
//! detection, including the optional SIMD-accelerated fast paths.

use super::token_scanner::TokenScanner;
use crate::tokenizer::{state::TokenContext, tokens::TokenType};
use crate::Result;

#[cfg(feature = "simd")]
use crate::tokenizer::simd;

impl TokenScanner<'_> {
    /// Scan general text content
    ///
    /// # Errors
    ///
    /// Returns an error if character navigation fails.
    pub fn scan_text(&mut self, context: TokenContext) -> Result<TokenType> {
        let start = self.navigator.position();

        // Use SIMD delimiter scanning when available and context doesn't affect delimiters
        #[cfg(feature = "simd")]
        {
            // Only use SIMD when context doesn't change delimiter behavior
            let use_simd = !matches!(context, TokenContext::FieldValue);

            if use_simd {
                if let Some(delimiter_pos) = self.scan_delimiters_simd(start) {
                    self.navigator.position = delimiter_pos;
                } else {
                    self.navigator.position = self.source.len();
                }
                self.navigator.chars = self.source[self.navigator.position..].chars();
                self.navigator.peek_char = None;
            }
        }

        // Fallback to scalar scanning (or when SIMD can't be used due to context)
        #[cfg(not(feature = "simd"))]
        let use_scalar = true;
        #[cfg(feature = "simd")]
        let use_scalar = matches!(context, TokenContext::FieldValue);

        if use_scalar {
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
        } else if !span.is_empty() && span.chars().all(char::is_whitespace) {
            Ok(TokenType::Whitespace)
        } else {
            Ok(TokenType::Text)
        }
    }

    /// Check if a span represents a hex value
    pub(super) fn is_hex_value(span: &str) -> bool {
        // Check &H format first (standard ASS hex format)
        if let Some(after_prefix) = span.strip_prefix("&H") {
            let hex_part = after_prefix
                .strip_suffix('&')
                .map_or(after_prefix, |stripped| stripped);

            if !hex_part.is_empty()
                && hex_part.len() % 2 == 0
                && hex_part.len() <= 8
                && hex_part.chars().all(|c| c.is_ascii_hexdigit())
            {
                #[cfg(feature = "simd")]
                {
                    return TokenScanner::parse_hex_simd(hex_part).is_some();
                }
                #[cfg(not(feature = "simd"))]
                {
                    return true;
                }
            }
        }

        // Raw hex without &H prefix is very rare in ASS files - don't detect it
        // to avoid conflicts with numbers and text

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
            if ch == ',' || ch == '\n' || ch == '\r' || ch == '{' || ch == '[' {
                break;
            }

            self.navigator.advance_char()?;
        }

        let span = &self.source[start..self.navigator.position()];

        if !span.is_empty()
            && span
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == ':')
        {
            Ok(TokenType::Number)
        } else if !span.is_empty() && span.chars().all(char::is_whitespace) {
            Ok(TokenType::Whitespace)
        } else {
            Ok(TokenType::Text)
        }
    }
}
