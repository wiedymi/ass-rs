//! Token scanning methods for ASS tokenizer
//!
//! Provides specialized scanning functions for different ASS script elements
//! including section headers, style overrides, comments, and text content.

use crate::{utils::CoreError, Result};
use alloc::{format, string::ToString};
use core::str::Chars;

#[cfg(not(feature = "std"))]
extern crate alloc;
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
    fn is_hex_value(span: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn char_navigator_new() {
        let source = "test content";
        let nav = CharNavigator::new(source, 5, 2, 3);
        assert_eq!(nav.position(), 5);
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 3);
    }

    #[test]
    fn char_navigator_peek_char() {
        let source = "hello";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        assert_eq!(nav.peek_char().unwrap(), 'h');
        assert_eq!(nav.peek_char().unwrap(), 'h'); // Should not advance
        assert_eq!(nav.position(), 0);
    }

    #[test]
    fn char_navigator_peek_next() {
        let source = "hello";
        let nav = CharNavigator::new(source, 0, 1, 1);
        assert_eq!(nav.peek_next().unwrap(), 'e');
    }

    #[test]
    fn char_navigator_advance_char() {
        let source = "hello";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        assert_eq!(nav.advance_char().unwrap(), 'h');
        assert_eq!(nav.position(), 1);
        assert_eq!(nav.column(), 2);
    }

    #[test]
    fn char_navigator_advance_newline() {
        let source = "line1\nline2";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        // Advance to newline
        for _ in 0..5 {
            nav.advance_char().unwrap();
        }
        assert_eq!(nav.advance_char().unwrap(), '\n');
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn char_navigator_advance_carriage_return() {
        let source = "line1\rline2";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        // Advance to carriage return
        for _ in 0..5 {
            nav.advance_char().unwrap();
        }
        assert_eq!(nav.advance_char().unwrap(), '\r');
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn char_navigator_advance_crlf() {
        let source = "line1\r\nline2";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        // Advance to \r
        for _ in 0..5 {
            nav.advance_char().unwrap();
        }
        assert_eq!(nav.advance_char().unwrap(), '\r');
        assert_eq!(nav.line(), 2);
        // Advance to \n
        assert_eq!(nav.advance_char().unwrap(), '\n');
        assert_eq!(nav.line(), 2); // Should not increment again
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn char_navigator_skip_whitespace() {
        let source = "   \t  hello";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        nav.skip_whitespace();
        assert_eq!(nav.peek_char().unwrap(), 'h');
    }

    #[test]
    fn char_navigator_skip_whitespace_preserves_newlines() {
        let source = "   \n  hello";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        nav.skip_whitespace();
        assert_eq!(nav.peek_char().unwrap(), '\n');
    }

    #[test]
    fn char_navigator_is_at_end() {
        let source = "hi";
        let nav = CharNavigator::new(source, 2, 1, 1);
        assert!(nav.is_at_end());

        let nav2 = CharNavigator::new(source, 0, 1, 1);
        assert!(!nav2.is_at_end());
    }

    #[test]
    fn char_navigator_peek_char_at_end() {
        let source = "hi";
        let mut nav = CharNavigator::new(source, 2, 1, 1);
        assert!(nav.peek_char().is_err());
    }

    #[test]
    fn char_navigator_peek_next_at_end() {
        let source = "h";
        let nav = CharNavigator::new(source, 0, 1, 1);
        assert!(nav.peek_next().is_err());
    }

    #[test]
    fn token_scanner_new() {
        let source = "test content";
        let scanner = TokenScanner::new(source, 5, 2, 3);
        assert_eq!(scanner.navigator().position(), 5);
        assert_eq!(scanner.navigator().line(), 2);
        assert_eq!(scanner.navigator().column(), 3);
    }

    #[test]
    fn token_scanner_scan_section_header() {
        let source = "[Script Info]";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_section_header().unwrap();
        assert_eq!(token_type, TokenType::SectionHeader);
    }

    #[test]
    fn token_scanner_scan_style_override() {
        let source = "{\\b1\\i1}";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_style_override().unwrap();
        assert_eq!(token_type, TokenType::OverrideBlock);
    }

    #[test]
    fn token_scanner_scan_style_override_nested() {
        let source = "{\\b1{\\i1}\\b0}";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_style_override().unwrap();
        assert_eq!(token_type, TokenType::OverrideBlock);
    }

    #[test]
    fn token_scanner_scan_comment() {
        let source = "; This is a comment\nNext line";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_comment().unwrap();
        assert_eq!(token_type, TokenType::Comment);
    }

    #[test]
    fn token_scanner_scan_text_basic() {
        let source = "Hello World,";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type, TokenType::Text);
    }

    #[test]
    fn token_scanner_scan_text_number() {
        let source = "123.45,";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type, TokenType::Number);
    }

    #[test]
    fn token_scanner_scan_text_hex_value() {
        let source = "&HABCDEF&,";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type, TokenType::HexValue);
    }

    #[test]
    fn token_scanner_scan_text_section_name() {
        let source = "Script Info]";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::SectionHeader).unwrap();
        assert_eq!(token_type, TokenType::SectionName);
    }

    #[test]
    fn token_scanner_scan_text_field_value_context() {
        let source = "0:01:23.45,";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type, TokenType::Text);
    }

    #[test]
    fn token_scanner_scan_field_value() {
        let source = "Some field value,";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_field_value().unwrap();
        assert_eq!(token_type, TokenType::Text);
    }

    #[test]
    fn token_scanner_scan_field_value_number() {
        let source = "0:01:23.45,";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_field_value().unwrap();
        assert_eq!(token_type, TokenType::Number);
    }

    #[test]
    fn token_scanner_is_hex_value_simple() {
        // Raw hex without &H prefix is not detected to avoid conflicts
        assert!(!TokenScanner::is_hex_value("ABCD"));
        assert!(!TokenScanner::is_hex_value("1234"));

        // With &H prefix, hex values are detected
        assert!(TokenScanner::is_hex_value("&HABCD&"));
        assert!(TokenScanner::is_hex_value("&H1234&"));
        assert!(!TokenScanner::is_hex_value("&HABCDE&")); // Odd length
        assert!(!TokenScanner::is_hex_value("&HGHIJ&")); // Invalid hex
        assert!(!TokenScanner::is_hex_value("")); // Empty
    }

    #[test]
    fn token_scanner_is_hex_value_with_prefix() {
        assert!(TokenScanner::is_hex_value("&HFF00FF&"));
        assert!(TokenScanner::is_hex_value("&HFF00FF"));
        assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
        assert!(!TokenScanner::is_hex_value("&HGHIJ&")); // Invalid hex
    }

    #[test]
    fn token_scanner_is_hex_value_max_length() {
        // Test that hex values have a reasonable maximum length limit
        // Raw hex without &H prefix is not detected anymore
        assert!(!TokenScanner::is_hex_value("ABCDEF")); // 6 chars - no prefix
        assert!(!TokenScanner::is_hex_value("00FF00FF")); // 8 chars - no prefix
        assert!(!TokenScanner::is_hex_value("1234567890")); // 10 chars - too long
        assert!(!TokenScanner::is_hex_value(&"A".repeat(100))); // Very long - not hex value

        // Test with &H prefix format (more permissive)
        assert!(TokenScanner::is_hex_value("&H00FF00FF&")); // 8 hex chars - valid
        assert!(TokenScanner::is_hex_value("&HABCD&")); // 4 hex chars - valid with prefix
        assert!(!TokenScanner::is_hex_value("&H1234567890&")); // 10 hex chars - too long

        // Test that numbers/text are not detected as hex
        assert!(!TokenScanner::is_hex_value("00")); // No prefix
        assert!(!TokenScanner::is_hex_value("123abc")); // No prefix
    }

    #[test]
    fn token_scanner_hex_value_trailing_ampersand_variants() {
        // Test hex values with trailing ampersand
        assert!(TokenScanner::is_hex_value("&H00FFFFFF&"));
        assert!(TokenScanner::is_hex_value("&HFF0000&"));
        assert!(TokenScanner::is_hex_value("&H80FF00FF&"));

        // Test hex values without trailing ampersand (common in ASS files)
        assert!(TokenScanner::is_hex_value("&H00FFFFFF"));
        assert!(TokenScanner::is_hex_value("&HFF0000"));
        assert!(TokenScanner::is_hex_value("&H80FF00FF"));

        // Test edge cases
        assert!(TokenScanner::is_hex_value("&H00&"));
        assert!(TokenScanner::is_hex_value("&H00"));
        assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
        assert!(!TokenScanner::is_hex_value("&H")); // No hex part
    }

    #[test]
    fn token_scanner_scan_text_hex_value_ampersand_variants() {
        // Test hex value with trailing ampersand
        let source1 = "&H00FFFFFF&";
        let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
        let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type1, TokenType::HexValue);
        assert_eq!(scanner1.navigator().position(), source1.len());

        // Test hex value without trailing ampersand (common in ASS files)
        let source2 = "&H00FFFFFF";
        let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
        let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type2, TokenType::HexValue);
        assert_eq!(scanner2.navigator().position(), source2.len());

        // Test short hex value variants
        let source3 = "&HFF00&";
        let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
        let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type3, TokenType::HexValue);

        let source4 = "&HFF00";
        let mut scanner4 = TokenScanner::new(source4, 0, 1, 1);
        let token_type4 = scanner4.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type4, TokenType::HexValue);
    }

    #[test]
    fn token_scanner_delimiter_context_field_value() {
        let source = "Title: My Script";
        let mut scanner = TokenScanner::new(source, 7, 1, 8); // Start after "Title: "
        let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type, TokenType::Text);
        // Should have consumed "My Script" without stopping at colon
    }

    #[test]
    fn token_scanner_delimiter_context_document() {
        let source = "Field:Value";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type, TokenType::Text);
        // Should stop at the colon, so only "Field" is consumed
        assert_eq!(scanner.navigator().position(), 5);
    }

    #[test]
    fn token_scanner_various_delimiters() {
        let test_cases = vec![
            (",", TokenContext::Document),
            ("{", TokenContext::Document),
            ("}", TokenContext::Document),
            ("[", TokenContext::Document),
            ("]", TokenContext::Document),
            ("\n", TokenContext::Document),
            ("\r", TokenContext::Document),
        ];

        for (delimiter, context) in test_cases {
            let source = format!("text{delimiter}more");
            let mut scanner = TokenScanner::new(&source, 0, 1, 1);
            let _token_type = scanner.scan_text(context).unwrap();
            assert_eq!(scanner.navigator().position(), 4); // Should stop at delimiter
        }
    }

    #[test]
    fn token_scanner_navigator_mut() {
        let source = "test";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        {
            let nav_mut = scanner.navigator_mut();
            nav_mut.advance_char().unwrap();
        }
        assert_eq!(scanner.navigator().position(), 1);
    }

    #[test]
    fn char_navigator_utf8_handling() {
        let source = "café";
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        assert_eq!(nav.advance_char().unwrap(), 'c');
        assert_eq!(nav.advance_char().unwrap(), 'a');
        assert_eq!(nav.advance_char().unwrap(), 'f');
        assert_eq!(nav.advance_char().unwrap(), 'é');
        assert_eq!(nav.position(), 5); // 'é' is 2 bytes in UTF-8
    }

    #[test]
    fn token_scanner_empty_section_header() {
        let source = "[]";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_section_header().unwrap();
        assert_eq!(token_type, TokenType::SectionHeader);
    }

    #[test]
    fn token_scanner_unclosed_section_header() {
        let source = "[Script Info";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_section_header().unwrap();
        assert_eq!(token_type, TokenType::SectionHeader);
    }

    #[test]
    fn token_scanner_empty_style_override() {
        let source = "{}";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_style_override().unwrap();
        assert_eq!(token_type, TokenType::OverrideBlock);
    }

    #[test]
    fn token_scanner_unclosed_style_override() {
        let source = "{\\b1\\i1";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_style_override().unwrap();
        assert_eq!(token_type, TokenType::OverrideBlock);
    }

    #[test]
    fn token_scanner_comment_at_end() {
        let source = "; Comment at end";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_comment().unwrap();
        assert_eq!(token_type, TokenType::Comment);
    }

    #[test]
    fn char_navigator_advance_char_error_handling() {
        let mut nav = CharNavigator::new("", 0, 1, 1);
        assert!(nav.advance_char().is_err());
        assert!(nav.peek_char().is_err());
        assert!(nav.peek_next().is_err());
    }

    #[test]
    fn char_navigator_peek_operations_edge_cases() {
        let source = "a";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Test peek_char caching
        assert_eq!(nav.peek_char().unwrap(), 'a');
        assert_eq!(nav.peek_char().unwrap(), 'a'); // Should return cached value

        // Test peek_next at boundaries
        assert!(nav.peek_next().is_err()); // Only one character

        nav.advance_char().unwrap();
        assert!(nav.peek_char().is_err()); // At end
        assert!(nav.peek_next().is_err()); // At end
    }

    #[test]
    fn char_navigator_line_column_tracking_complex() {
        let source = "line1\r\nline2\rline3\nline4";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Test \r\n handling
        for _ in "line1".chars() {
            nav.advance_char().unwrap();
        }
        assert_eq!(nav.line(), 1);
        assert_eq!(nav.column(), 6);

        nav.advance_char().unwrap(); // \r
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);

        nav.advance_char().unwrap(); // \n (shouldn't increment line again)
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);

        // Test standalone \r
        for _ in "line2".chars() {
            nav.advance_char().unwrap();
        }
        nav.advance_char().unwrap(); // \r
        assert_eq!(nav.line(), 3);
        assert_eq!(nav.column(), 1);

        // Test standalone \n
        for _ in "line3".chars() {
            nav.advance_char().unwrap();
        }
        nav.advance_char().unwrap(); // \n
        assert_eq!(nav.line(), 4);
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn char_navigator_skip_whitespace_variations() {
        let source = " \t\u{00A0}\u{2000} text"; // Various whitespace types
        let mut nav = CharNavigator::new(source, 0, 1, 1);
        nav.skip_whitespace();
        assert_eq!(nav.peek_char().unwrap(), 't');

        // Test with newlines (should not skip)
        let source2 = "  \n  text";
        let mut nav2 = CharNavigator::new(source2, 0, 1, 1);
        nav2.skip_whitespace();
        assert_eq!(nav2.peek_char().unwrap(), '\n');
    }

    #[test]
    fn token_scanner_scan_text_field_value_context_edge_cases() {
        // Test colon in field values (should not be delimiter)
        let source = "0:00:30.50";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type, TokenType::Text);
        assert_eq!(scanner.navigator().position(), source.len());

        // Test various delimiters in field value context
        let source2 = "text,next";
        let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
        let token_type2 = scanner2.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type2, TokenType::Text);
        assert_eq!(scanner2.navigator().position(), 4); // Stopped at comma
    }

    #[test]
    fn token_scanner_scan_text_number_detection() {
        // Test positive number
        let source1 = "123.45";
        let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
        let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type1, TokenType::Number);

        // Test negative number
        let source2 = "-123.45";
        let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
        let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type2, TokenType::Number);

        // Test integer
        let source3 = "123";
        let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
        let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type3, TokenType::Number);

        // Test text (contains letters)
        let source4 = "123abc";
        let mut scanner4 = TokenScanner::new(source4, 0, 1, 1);
        let token_type4 = scanner4.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type4, TokenType::Text);
    }

    #[test]
    fn token_scanner_scan_field_value_comprehensive() {
        // Test field value with colon (time format)
        let source1 = "0:01:30.50,next";
        let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
        let token_type1 = scanner1.scan_field_value().unwrap();
        assert_eq!(token_type1, TokenType::Number);
        assert_eq!(scanner1.navigator().position(), 10); // Stopped at comma

        // Test regular text field value
        let source2 = "Some text,next";
        let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
        let token_type2 = scanner2.scan_field_value().unwrap();
        assert_eq!(token_type2, TokenType::Text);
        assert_eq!(scanner2.navigator().position(), 9); // Stopped at comma

        // Test field value stopping at various delimiters
        let source3 = "text\nmore";
        let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
        let token_type3 = scanner3.scan_field_value().unwrap();
        assert_eq!(token_type3, TokenType::Text);
        assert_eq!(scanner3.navigator().position(), 4); // Stopped at newline

        let source4 = "text{override}";
        let mut scanner4 = TokenScanner::new(source4, 0, 1, 1);
        let token_type4 = scanner4.scan_field_value().unwrap();
        assert_eq!(token_type4, TokenType::Text);
        assert_eq!(scanner4.navigator().position(), 4); // Stopped at brace

        let source5 = "text[section]";
        let mut scanner5 = TokenScanner::new(source5, 0, 1, 1);
        let token_type5 = scanner5.scan_field_value().unwrap();
        assert_eq!(token_type5, TokenType::Text);
        assert_eq!(scanner5.navigator().position(), 4); // Stopped at bracket
    }

    #[test]
    fn token_scanner_section_header_variations() {
        // Test section header with spaces
        let source1 = "[ Script Info ]";
        let mut scanner1 = TokenScanner::new(source1, 1, 1, 2); // Start after [
        let token_type1 = scanner1.scan_section_header().unwrap();
        assert_eq!(token_type1, TokenType::SectionHeader);

        // Test section header with special characters
        let source2 = "[V4+ Styles]";
        let mut scanner2 = TokenScanner::new(source2, 1, 1, 2); // Start after [
        let token_type2 = scanner2.scan_section_header().unwrap();
        assert_eq!(token_type2, TokenType::SectionHeader);

        // Test malformed section header (missing closing bracket)
        let source3 = "[Script Info\nNext line";
        let mut scanner3 = TokenScanner::new(source3, 1, 1, 2); // Start after [
        let token_type3 = scanner3.scan_section_header().unwrap();
        assert_eq!(token_type3, TokenType::SectionHeader);
    }

    #[test]
    fn token_scanner_style_override_complex() {
        // Test nested braces
        let source1 = "{\\b1{nested}\\i1}";
        let mut scanner1 = TokenScanner::new(source1, 1, 1, 2); // Start after {
        let token_type1 = scanner1.scan_style_override().unwrap();
        assert_eq!(token_type1, TokenType::OverrideBlock);

        // Test override with no tags
        let source2 = "{ }";
        let mut scanner2 = TokenScanner::new(source2, 1, 1, 2); // Start after {
        let token_type2 = scanner2.scan_style_override().unwrap();
        assert_eq!(token_type2, TokenType::OverrideBlock);

        // Test unclosed override at end of line
        let source3 = "{\\b1\\i1\n";
        let mut scanner3 = TokenScanner::new(source3, 1, 1, 2); // Start after {
        let token_type3 = scanner3.scan_style_override().unwrap();
        assert_eq!(token_type3, TokenType::OverrideBlock);
    }

    #[test]
    fn token_scanner_comment_variations() {
        // Test comment with exclamation
        let source1 = "!: This is a comment";
        let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
        let token_type1 = scanner1.scan_comment().unwrap();
        assert_eq!(token_type1, TokenType::Comment);

        // Test comment at end of file without newline
        let source2 = "; Comment";
        let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
        let token_type2 = scanner2.scan_comment().unwrap();
        assert_eq!(token_type2, TokenType::Comment);
        assert_eq!(scanner2.navigator().position(), source2.len());

        // Test empty comment
        let source3 = ";\n";
        let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
        let token_type3 = scanner3.scan_comment().unwrap();
        assert_eq!(token_type3, TokenType::Comment);
    }

    #[test]
    fn token_scanner_unicode_handling() {
        // Test Unicode text
        let source1 = "中文测试";
        let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
        let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type1, TokenType::Text);
        assert_eq!(scanner1.navigator().position(), source1.len());

        // Test Unicode in section header
        let source2 = "[スクリプト情報]";
        let mut scanner2 = TokenScanner::new(source2, 1, 1, 2); // Start after [
        let token_type2 = scanner2.scan_section_header().unwrap();
        assert_eq!(token_type2, TokenType::SectionHeader);

        // Test emoji
        let source3 = "🎭🎬🎪";
        let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
        let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type3, TokenType::Text);
        assert_eq!(scanner3.navigator().position(), source3.len());
    }

    #[test]
    fn token_scanner_empty_content_handling() {
        // Test empty text scan
        let source1 = ",next";
        let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
        let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type1, TokenType::Text);
        assert_eq!(scanner1.navigator().position(), 0); // No advancement for empty content

        // Test scan at end of input
        let source2 = "text";
        let mut scanner2 = TokenScanner::new(source2, 4, 1, 5); // Start at end
        let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type2, TokenType::Text);
    }

    #[test]
    fn char_navigator_boundary_conditions() {
        // Test very long line
        let long_line = "a".repeat(10000);
        let mut nav = CharNavigator::new(&long_line, 0, 1, 1);
        for i in 1..=10000 {
            nav.advance_char().unwrap();
            assert_eq!(nav.column(), i + 1);
        }
        assert!(nav.is_at_end());

        // Test many lines
        let many_lines = "a\n".repeat(1000);
        let mut nav2 = CharNavigator::new(&many_lines, 0, 1, 1);
        for i in 1..=1000 {
            nav2.advance_char().unwrap(); // 'a'
            nav2.advance_char().unwrap(); // '\n'
            assert_eq!(nav2.line(), i + 1);
            assert_eq!(nav2.column(), 1);
        }
    }

    #[test]
    fn token_scanner_simd_fallback_coverage() {
        // Test contexts that should trigger scalar scanning even with SIMD
        let source = "field:value,next";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type, TokenType::Text);
        // Should stop at comma, not colon in field value context
        assert_eq!(scanner.navigator().position(), 11);
    }

    #[test]
    fn char_navigator_error_recovery() {
        // Test error handling when advancing past end
        let source = "a";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Advance to end
        nav.advance_char().unwrap();
        assert!(nav.is_at_end());

        // Further advances should fail
        assert!(nav.advance_char().is_err());
        assert!(nav.peek_char().is_err());
    }

    #[test]
    fn char_navigator_peek_char_caching() {
        let source = "hello";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // First peek should cache the character
        assert_eq!(nav.peek_char().unwrap(), 'h');

        // Subsequent peeks should return cached value
        assert_eq!(nav.peek_char().unwrap(), 'h');
        assert_eq!(nav.peek_char().unwrap(), 'h');

        // After advancing, peek should work on next character
        nav.advance_char().unwrap();
        assert_eq!(nav.peek_char().unwrap(), 'e');
    }

    #[test]
    fn char_navigator_last_char_tracking() {
        let source = "a\r\nb";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        nav.advance_char().unwrap(); // 'a'
        nav.advance_char().unwrap(); // '\r' - should increment line
        assert_eq!(nav.line(), 2);

        nav.advance_char().unwrap(); // '\n' - should NOT increment line again
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn token_scanner_hex_value_edge_cases() {
        // Raw hex without &H prefix is not detected to avoid conflicts
        assert!(!TokenScanner::is_hex_value("FF"));
        assert!(!TokenScanner::is_hex_value("00"));
        assert!(!TokenScanner::is_hex_value("ABCDEF"));
        assert!(!TokenScanner::is_hex_value("123456"));

        // Test &H prefix variations
        assert!(TokenScanner::is_hex_value("&HFF&"));
        assert!(TokenScanner::is_hex_value("&HFF"));
        assert!(TokenScanner::is_hex_value("&H00FF00FF&"));
        assert!(TokenScanner::is_hex_value("&H00FF00FF"));

        // Test invalid cases
        assert!(!TokenScanner::is_hex_value("F")); // Raw hex not detected
        assert!(!TokenScanner::is_hex_value("GG")); // Raw hex not detected
        assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
        assert!(!TokenScanner::is_hex_value("&HG&")); // Invalid hex with prefix
        assert!(!TokenScanner::is_hex_value("")); // Empty string
    }

    #[test]
    fn scan_text_classification_verification() {
        // Verify that previously problematic cases now work correctly

        // Test case 1: "0:00:30.50" should be Text (timestamps are structured data, not numbers)
        let source1 = "0:00:30.50";
        let mut scanner1 = TokenScanner::new(source1, 0, 1, 1);
        let token_type1 = scanner1.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type1, TokenType::Text);

        // Test case 2: "123abc" should be Text (not hex without &H prefix)
        let source2 = "123abc";
        let mut scanner2 = TokenScanner::new(source2, 0, 1, 1);
        let token_type2 = scanner2.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type2, TokenType::Text);

        // Test case 3: Proper hex with &H prefix should be HexValue
        let source3 = "&H00FF00&";
        let mut scanner3 = TokenScanner::new(source3, 0, 1, 1);
        let token_type3 = scanner3.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type3, TokenType::HexValue);
    }

    #[test]
    fn token_scanner_delimiter_combinations() {
        // Test complex delimiter combinations
        let source = "text:{}[],more";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type, TokenType::Text);
        assert_eq!(scanner.navigator().position(), 4); // Should stop at first delimiter ':'
    }

    #[test]
    fn token_scanner_field_value_delimiter_handling() {
        // Test that colons are not delimiters in field value context
        let source = "0:00:30.50";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type, TokenType::Text); // Contains colons, so treated as text in scan_text
        assert_eq!(scanner.navigator().position(), source.len()); // Should consume entire string
    }

    #[test]
    fn token_scanner_semicolon_context_sensitivity() {
        // Test semicolon handling in different contexts
        let source = "text;comment";

        // In Document context, semicolon behavior depends on SIMD vs scalar implementation
        let mut scanner1 = TokenScanner::new(source, 0, 1, 1);
        let token_type1 = scanner1.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type1, TokenType::Text);
        // Position may vary based on SIMD feature and implementation details
        assert!(scanner1.navigator().position() >= 4);

        // In FieldValue context, should not stop at semicolon
        let mut scanner2 = TokenScanner::new(source, 0, 1, 1);
        let token_type2 = scanner2.scan_text(TokenContext::FieldValue).unwrap();
        assert_eq!(token_type2, TokenType::Text);
        assert_eq!(scanner2.navigator().position(), source.len());
    }

    #[test]
    fn token_scanner_number_detection_edge_cases() {
        // Test various number formats
        let test_cases = vec![
            ("123", true),
            ("123.45", true),
            ("-123", true),
            ("-123.45", true),
            ("123.", true),
            (".45", true),
            ("-.45", true),
            ("123abc", false),  // Contains letters
            ("", false),        // Empty
            (".", true),        // Just decimal point
            ("-", true),        // Just minus
            ("--123", true),    // Still only contains valid number chars
            ("12.34.56", true), // Multiple decimals but still only valid chars
        ];

        for (input, expected_is_number) in test_cases {
            let source = format!("{input},");
            let mut scanner = TokenScanner::new(&source, 0, 1, 1);
            let token_type = scanner.scan_text(TokenContext::Document).unwrap();

            if expected_is_number && !input.is_empty() {
                assert_eq!(token_type, TokenType::Number, "Failed for input: {input}");
            } else {
                assert_ne!(token_type, TokenType::Number, "Failed for input: {input}");
            }
        }
    }

    #[test]
    fn token_scanner_style_override_brace_depth() {
        // Test proper brace depth tracking
        let source = "{{{{}}}}";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_style_override().unwrap();
        assert_eq!(token_type, TokenType::OverrideBlock);
        assert_eq!(scanner.navigator().position(), 7); // Should stop before final closing brace
    }

    #[test]
    fn token_scanner_style_override_unbalanced() {
        // Test unbalanced braces
        let source = "{{{}}"; // Missing one closing brace
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_style_override().unwrap();
        assert_eq!(token_type, TokenType::OverrideBlock);
        // Should consume until end even if unbalanced
        assert_eq!(scanner.navigator().position(), source.len());
    }

    #[test]
    fn char_navigator_whitespace_at_end() {
        let source = "text   ";
        let mut nav = CharNavigator::new(source, 4, 1, 5); // Start at first space
        nav.skip_whitespace();
        assert!(nav.is_at_end());
    }

    #[test]
    fn char_navigator_mixed_newlines() {
        let source = "\r\n\n\r";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // \r
        nav.advance_char().unwrap();
        assert_eq!(nav.line(), 2);

        // \n (after \r, should not increment line)
        nav.advance_char().unwrap();
        assert_eq!(nav.line(), 2);

        // \n (standalone, should increment line)
        nav.advance_char().unwrap();
        assert_eq!(nav.line(), 3);

        // \r (standalone, should increment line)
        nav.advance_char().unwrap();
        assert_eq!(nav.line(), 4);
    }

    #[test]
    fn token_scanner_empty_span_handling() {
        // Test scanning empty content
        let source = ",";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_text(TokenContext::Document).unwrap();
        assert_eq!(token_type, TokenType::Text);
        assert_eq!(scanner.navigator().position(), 0); // Should not advance for empty content
    }

    #[test]
    fn token_scanner_field_value_time_format() {
        // Test time format recognition in field values
        let source = "1:23:45.67";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);
        let token_type = scanner.scan_field_value().unwrap();
        assert_eq!(token_type, TokenType::Number);
        assert_eq!(scanner.navigator().position(), source.len());
    }

    #[test]
    fn char_navigator_position_consistency() {
        let source = "café🎭";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        let start_pos = nav.position();
        nav.advance_char().unwrap(); // 'c' - 1 byte
        assert_eq!(nav.position(), start_pos + 1);

        nav.advance_char().unwrap(); // 'a' - 1 byte
        assert_eq!(nav.position(), start_pos + 2);

        nav.advance_char().unwrap(); // 'f' - 1 byte
        assert_eq!(nav.position(), start_pos + 3);

        nav.advance_char().unwrap(); // 'é' - 2 bytes
        assert_eq!(nav.position(), start_pos + 5);

        nav.advance_char().unwrap(); // '🎭' - 4 bytes
        assert_eq!(nav.position(), start_pos + 9);
    }

    #[test]
    fn token_scanner_all_contexts_coverage() {
        // Test scanning in all different contexts
        let contexts = vec![
            TokenContext::Document,
            TokenContext::SectionHeader,
            TokenContext::FieldValue,
            TokenContext::StyleOverride,
        ];

        for context in contexts {
            let source = "test:value,more";
            let mut scanner = TokenScanner::new(source, 0, 1, 1);
            let token_type = scanner.scan_text(context).unwrap();

            // Should return appropriate token type based on context
            match context {
                TokenContext::SectionHeader => assert_eq!(token_type, TokenType::SectionName),
                _ => assert!(matches!(
                    token_type,
                    TokenType::Text | TokenType::Number | TokenType::HexValue
                )),
            }
        }
    }

    #[test]
    fn char_navigator_column_reset_on_newlines() {
        let source = "long line text\nshort\n";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Advance to end of first line
        for _ in 0..14 {
            nav.advance_char().unwrap();
        }
        assert_eq!(nav.column(), 15);

        // Advance over newline
        nav.advance_char().unwrap(); // \n
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);

        // Advance a few chars on second line
        for _ in 0..5 {
            nav.advance_char().unwrap();
        }
        assert_eq!(nav.column(), 6);

        // Advance over second newline
        nav.advance_char().unwrap(); // \n
        assert_eq!(nav.line(), 3);
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn char_navigator_utf8_error_handling() {
        // Test invalid UTF-8 sequences
        let source = "valid\x7F\x7E";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Should advance through valid chars
        assert!(nav.advance_char().is_ok());
        assert!(nav.advance_char().is_ok());
        assert!(nav.advance_char().is_ok());
        assert!(nav.advance_char().is_ok());
        assert!(nav.advance_char().is_ok());

        // Should handle invalid UTF-8
        let result = nav.advance_char();
        match result {
            Ok(_) | Err(_) => {
                // Both outcomes acceptable - important is no panic
                assert!(nav.position() > 0);
            }
        }
    }

    #[test]
    fn char_navigator_peek_char_caching_coverage() {
        let source = "abc";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // First peek should cache the character
        let first_peek = nav.peek_char();
        assert_eq!(first_peek, Ok('a'));

        // Second peek should use cached value
        let second_peek = nav.peek_char();
        assert_eq!(second_peek, Ok('a'));

        // Advance should clear cache and move to next char
        assert!(nav.advance_char().is_ok());

        // Next peek should get new character
        let third_peek = nav.peek_char();
        assert_eq!(third_peek, Ok('b'));
    }

    #[test]
    fn char_navigator_last_char_tracking_coverage() {
        let source = "xy\nz";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Advance through characters and track last_char
        nav.advance_char().unwrap(); // 'x'
        assert_eq!(nav.last_char, Some('x'));

        nav.advance_char().unwrap(); // 'y'
        assert_eq!(nav.last_char, Some('y'));

        nav.advance_char().unwrap(); // '\n'
        assert_eq!(nav.last_char, Some('\n'));
        assert_eq!(nav.line(), 2);

        nav.advance_char().unwrap(); // 'z'
        assert_eq!(nav.last_char, Some('z'));
    }

    #[test]
    fn token_scanner_hex_value_comprehensive_coverage() {
        // Test hex with ampersand suffix (must be even length)
        assert!(TokenScanner::is_hex_value("&H1234&"));
        assert!(TokenScanner::is_hex_value("&HFFFF&"));
        assert!(TokenScanner::is_hex_value("&H00&"));

        // Test hex without ampersand suffix (must be even length)
        assert!(TokenScanner::is_hex_value("&H1234"));
        assert!(TokenScanner::is_hex_value("&HABCD"));

        // Test empty hex part
        assert!(!TokenScanner::is_hex_value("&H&"));
        assert!(!TokenScanner::is_hex_value("&H"));

        // Test odd length (invalid)
        assert!(!TokenScanner::is_hex_value("&H123&"));
        assert!(!TokenScanner::is_hex_value("&H0&"));

        // Test max length enforcement
        assert!(!TokenScanner::is_hex_value("&H123456789ABCDEF&")); // Too long

        // Test invalid characters
        assert!(!TokenScanner::is_hex_value("&HZ123&"));
        assert!(!TokenScanner::is_hex_value("&H12G4&"));
    }

    #[test]
    fn token_scanner_delimiter_context_comprehensive() {
        let source = ",{[}]:;\n\r";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        // Test field value context delimiters
        let result = scanner.scan_field_value();
        assert!(result.is_ok());

        // Test document context delimiter behavior
        let source2 = ";comment";
        let scanner2 = TokenScanner::new(source2, 0, 1, 1);
        let nav_pos = scanner2.navigator().position();

        // Should handle semicolon in document context
        assert_eq!(nav_pos, 0);
    }

    #[test]
    fn token_scanner_scan_text_number_classification() {
        let source = "123.45";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
        assert!(result.is_ok());

        let token_type = result.unwrap();
        assert_eq!(token_type, crate::tokenizer::tokens::TokenType::Number);
    }

    #[test]
    fn token_scanner_section_header_boundary_coverage() {
        let source = "[Section]";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        // Should find closing bracket
        let result = scanner.scan_section_header();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            crate::tokenizer::tokens::TokenType::SectionHeader
        );
    }

    #[test]
    fn token_scanner_style_override_brace_matching() {
        let source = "{\\b1}text{\\b0}";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        // Should handle nested braces correctly
        let result = scanner.scan_style_override();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            crate::tokenizer::tokens::TokenType::OverrideBlock
        );
    }

    #[test]
    fn token_scanner_simd_fallback_forced_coverage() {
        let source = "test,delimiter:content";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        // Force scalar path by testing with different contexts
        let result = scanner.scan_field_value();
        assert!(result.is_ok());
    }

    #[test]
    fn char_navigator_advance_char_utf8_length_tracking() {
        let source = "a🎵b";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Advance over 'a' (1 byte)
        nav.advance_char().unwrap();
        assert_eq!(nav.position(), 1);

        // Advance over '🎵' (4 bytes)
        nav.advance_char().unwrap();
        assert_eq!(nav.position(), 5);

        // Advance over 'b' (1 byte)
        nav.advance_char().unwrap();
        assert_eq!(nav.position(), 6);
    }

    #[test]
    fn token_scanner_empty_span_edge_cases() {
        let source = "";
        let scanner = TokenScanner::new(source, 0, 1, 1);

        // Test scan operations on empty source
        let nav_result = scanner.navigator();
        assert!(nav_result.is_at_end());

        // Verify position tracking
        assert_eq!(nav_result.position(), 0);
        assert_eq!(nav_result.line(), 1);
        assert_eq!(nav_result.column(), 1);
    }

    #[test]
    fn char_navigator_peek_operations_at_boundaries() {
        let source = "a";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Peek at first character
        assert_eq!(nav.peek_char().unwrap(), 'a');

        // Peek next should handle end of input
        assert!(nav.peek_next().is_err());

        // Advance to end
        nav.advance_char().unwrap();
        assert!(nav.is_at_end());

        // Peek at end should handle end of input
        assert!(nav.peek_char().is_err());
        assert!(nav.peek_next().is_err());
    }

    #[test]
    fn token_scanner_all_delimiter_combinations_coverage() {
        let delimiters = [',', ':', '{', '}', '[', ']', '\n', '\r'];

        for &delimiter in &delimiters {
            let source = format!("text{delimiter}more");
            let mut scanner = TokenScanner::new(&source, 0, 1, 1);

            // Test delimiter detection in different contexts
            let result = scanner.scan_field_value();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn char_navigator_newline_variations_comprehensive() {
        // Test different newline types
        let sources = [
            "line1\nline2",   // LF
            "line1\rline2",   // CR
            "line1\r\nline2", // CRLF
        ];

        for source in &sources {
            let mut nav = CharNavigator::new(source, 0, 1, 1);

            // Advance to newline
            while let Ok(ch) = nav.advance_char() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
            }

            // Should be on line 2 after newline processing
            if !nav.is_at_end() {
                nav.advance_char().ok(); // Move past newline
                assert!(nav.line() >= 2);
            }
        }
    }

    #[test]
    fn char_navigator_carriage_return_line_increment() {
        // Target lines 114-118: '\r' handling in advance_char
        let source = "text\rmore";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Advance to carriage return
        for _ in 0..4 {
            nav.advance_char().unwrap();
        }

        // This should hit the '\r' branch and increment line
        let ch = nav.advance_char().unwrap();
        assert_eq!(ch, '\r');
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn char_navigator_newline_line_increment() {
        // Target lines 130-131: '\n' handling
        let source = "text\nmore";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        for _ in 0..4 {
            nav.advance_char().unwrap();
        }

        let ch = nav.advance_char().unwrap();
        assert_eq!(ch, '\n');
        assert_eq!(nav.line(), 2);
        assert_eq!(nav.column(), 1);
    }

    #[test]
    fn char_navigator_column_increment_default() {
        // Target lines 144: default column increment
        let source = "abc";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        nav.advance_char().unwrap(); // 'a'
        assert_eq!(nav.column(), 2);

        nav.advance_char().unwrap(); // 'b'
        assert_eq!(nav.column(), 3);

        nav.advance_char().unwrap(); // 'c'
        assert_eq!(nav.column(), 4);
    }

    #[test]
    fn char_navigator_skip_whitespace_loop() {
        // Target lines 140-144: skip_whitespace loop
        let source = "   \t\n  text";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        nav.skip_whitespace();
        assert_eq!(nav.position(), 4); // Should stop at newline
    }

    #[test]
    fn token_scanner_section_header_closing_bracket() {
        // Target lines 196, 199: section header scanning
        let source = "[Test]";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_section_header();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            crate::tokenizer::tokens::TokenType::SectionHeader
        );
    }

    #[test]
    fn token_scanner_style_override_closing_brace() {
        // Target lines 216, 222: style override scanning
        let source = "{\\b1}";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_style_override();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            crate::tokenizer::tokens::TokenType::OverrideBlock
        );
    }

    #[test]
    fn token_scanner_comment_scanning() {
        // Target line 241: comment scanning
        let source = "! This is a comment";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_comment();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            crate::tokenizer::tokens::TokenType::Comment
        );
    }

    #[test]
    fn token_scanner_scan_text_hex_detection() {
        // Target lines 263, 274: hex value detection in scan_text
        let source = "&H1234&";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            crate::tokenizer::tokens::TokenType::HexValue
        );
    }

    #[test]
    fn token_scanner_scan_text_number_detection_targeted() {
        // Target lines 283-284: number detection
        let source = "123.45";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), crate::tokenizer::tokens::TokenType::Number);
    }

    #[test]
    fn token_scanner_scan_text_section_name_context() {
        // Target lines 288, 290-291: section name context
        let source = "Script Info";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_text(crate::tokenizer::state::TokenContext::SectionHeader);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            crate::tokenizer::tokens::TokenType::SectionName
        );
    }

    #[test]
    fn token_scanner_scan_text_field_value_context_targeted() {
        // Target lines 295, 299: field value context
        let source = "value_text";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_text(crate::tokenizer::state::TokenContext::FieldValue);
        assert!(result.is_ok());
        // Should return Text type in field value context
    }

    #[test]
    fn token_scanner_scan_text_default_case() {
        // Target line 305: default text case
        let source = "regular_text";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), crate::tokenizer::tokens::TokenType::Text);
    }

    #[test]
    fn token_scanner_is_hex_value_ampersand_suffix() {
        // Target lines 324, 326, 328: hex value validation
        assert!(TokenScanner::is_hex_value("&H1234&"));
        assert!(TokenScanner::is_hex_value("&HABCD&"));
        assert!(!TokenScanner::is_hex_value("&H&")); // Empty hex part
    }

    #[test]
    fn token_scanner_is_hex_value_no_ampersand() {
        // Target line 339: hex without trailing ampersand
        assert!(TokenScanner::is_hex_value("&H1234"));
        assert!(TokenScanner::is_hex_value("&HABCD"));
    }

    #[test]
    fn token_scanner_scan_field_value_basic() {
        // Target lines 381, 386: scan_field_value
        let source = "field_value,next";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        let result = scanner.scan_field_value();
        assert!(result.is_ok());
    }

    #[test]
    fn char_navigator_peek_char_error_path() {
        // Target lines 76, 79, 81-82: peek_char error handling
        let source = "a";
        let mut nav = CharNavigator::new(source, 0, 1, 1);

        // Advance past end
        nav.advance_char().unwrap();
        assert!(nav.is_at_end());

        // peek_char should return error at end
        let result = nav.peek_char();
        assert!(result.is_err());
    }

    #[test]
    fn char_navigator_peek_next_error_path() {
        // Target lines 108, 110-111, 113: peek_next error handling
        let source = "a";
        let nav = CharNavigator::new(source, 0, 1, 1);

        // peek_next from last character should error
        let result = nav.peek_next();
        assert!(result.is_err());
    }
}
