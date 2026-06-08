//! Core per-token stepping logic for [`AssTokenizer`].
//!
//! Implements `next_token`, the context-sensitive scanner step that drives
//! incremental tokenization while preserving zero-copy `&'a str` spans.

use super::{AssTokenizer, Token, TokenContext, TokenType};
use crate::Result;

impl<'a> AssTokenizer<'a> {
    /// Get next token from input stream
    ///
    /// Returns `None` when end of input reached. Maintains zero-copy
    /// semantics by returning spans into source text.
    ///
    /// # Errors
    ///
    /// Returns an error if tokenization fails due to invalid input or scanner errors.
    pub fn next_token(&mut self) -> Result<Option<Token<'a>>> {
        if self.context.allows_whitespace_skipping() {
            self.scanner.navigator_mut().skip_whitespace();
        }

        if self.scanner.navigator().is_at_end() {
            return Ok(None);
        }

        let start_pos = self.scanner.navigator().position();
        let start_line = self.scanner.navigator().line();
        let start_column = self.scanner.navigator().column();

        let current_char = self.scanner.navigator_mut().peek_char()?;

        let token_type = match (current_char, self.context) {
            ('[', _) => {
                self.context = TokenContext::SectionHeader;
                self.scanner.scan_section_header()
            }
            (']', TokenContext::SectionHeader) => {
                self.context = TokenContext::Document;
                self.scanner.navigator_mut().advance_char()?;
                Ok(TokenType::SectionClose)
            }
            (':', TokenContext::Document) => {
                self.context = self.context.enter_field_value();
                self.scanner.navigator_mut().advance_char()?;
                Ok(TokenType::Colon)
            }

            ('{', _) => {
                self.context = TokenContext::StyleOverride;
                self.scanner.scan_style_override()
            }
            ('}', TokenContext::StyleOverride) => {
                self.context = TokenContext::Document;
                self.scanner.navigator_mut().advance_char()?;
                Ok(TokenType::OverrideClose)
            }
            (',', _) => {
                self.scanner.navigator_mut().advance_char()?;
                Ok(TokenType::Comma)
            }
            ('\n' | '\r', _) => {
                self.context = self.context.reset_to_document();
                self.scanner.navigator_mut().advance_char()?;
                if current_char == '\r' && self.scanner.navigator_mut().peek_char()? == '\n' {
                    self.scanner.navigator_mut().advance_char()?;
                }
                Ok(TokenType::Newline)
            }
            (';', TokenContext::Document) => self.scanner.scan_comment(),
            ('!', TokenContext::Document) => {
                // Check if next character is ':' for comment marker "!:"
                if self.scanner.navigator().peek_next() == Ok(':') {
                    self.scanner.scan_comment()
                } else {
                    self.scanner.scan_text(self.context)
                }
            }
            // Handle delimiter characters in wrong context as literal text
            ('}', _) => {
                // '}' outside StyleOverride context is literal text
                self.scanner.navigator_mut().advance_char()?;
                Ok(TokenType::Text)
            }
            (']', _) => {
                // ']' outside SectionHeader context is literal text
                self.scanner.navigator_mut().advance_char()?;
                Ok(TokenType::Text)
            }
            _ => {
                // In FieldValue context, consume everything until delimiter
                if self.context == TokenContext::FieldValue {
                    self.scanner.scan_field_value()
                } else {
                    self.scanner.scan_text(self.context)
                }
            }
        }?;

        let end_pos = self.scanner.navigator().position();
        let span = &self.source[start_pos..end_pos];

        // Check for infinite loop - position must advance unless at end
        if start_pos == end_pos && !self.scanner.navigator().is_at_end() {
            return Err(crate::utils::CoreError::internal(
                "Tokenizer position not advancing",
            ));
        }

        Ok(Some(Token {
            token_type,
            span,
            line: start_line,
            column: start_column,
        }))
    }
}
