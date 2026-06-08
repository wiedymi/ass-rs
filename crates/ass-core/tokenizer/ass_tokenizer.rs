//! Incremental ASS tokenizer definition and lifecycle helpers.
//!
//! Houses the [`AssTokenizer`] struct together with construction, batch
//! tokenization, issue access, and position queries. The per-token stepping
//! logic lives in the sibling `next_token` module.

use super::{IssueCollector, Token, TokenContext, TokenIssue, TokenScanner};
use crate::Result;
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

/// Incremental tokenizer for ASS scripts with zero-copy design
///
/// Maintains lexical state for streaming tokenization. Uses `&'a str` spans
/// to avoid allocations, with optional SIMD acceleration for hot paths.
#[derive(Debug, Clone)]
pub struct AssTokenizer<'a> {
    /// Source text being tokenized
    pub(super) source: &'a str,
    /// Token scanner for character processing
    pub(super) scanner: TokenScanner<'a>,
    /// Current tokenization context
    pub(super) context: TokenContext,
    /// Issue collector for error reporting
    issues: IssueCollector<'a>,
}

impl<'a> AssTokenizer<'a> {
    /// Create new tokenizer for source text
    ///
    /// Handles BOM detection and UTF-8 validation upfront.
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        let initial_position = if source.starts_with('\u{FEFF}') {
            3 // BOM is 3 bytes
        } else {
            0
        };

        Self {
            source,
            scanner: TokenScanner::new(source, initial_position, 1, 1),
            context: TokenContext::Document,
            issues: IssueCollector::new(),
        }
    }

    /// Get all tokens as vector for batch processing
    ///
    /// # Errors
    ///
    /// Returns an error if tokenization fails for any token in the input.
    pub fn tokenize_all(&mut self) -> Result<Vec<Token<'a>>> {
        let mut tokens = Vec::new();
        let mut iteration_count = 0;
        while let Some(token) = self.next_token()? {
            tokens.push(token);
            iteration_count += 1;
            if iteration_count > 50 {
                return Err(crate::utils::CoreError::internal(
                    "Too many tokenizer iterations",
                ));
            }
        }

        Ok(tokens)
    }

    /// Get accumulated tokenization issues
    #[must_use]
    pub fn issues(&self) -> &[TokenIssue<'a>] {
        self.issues.issues()
    }

    /// Get current position in source
    #[must_use]
    pub const fn position(&self) -> usize {
        self.scanner.navigator().position()
    }

    /// Get current line number (1-based)
    #[must_use]
    pub const fn line(&self) -> usize {
        self.scanner.navigator().line()
    }

    /// Get current column number (1-based)
    #[must_use]
    pub const fn column(&self) -> usize {
        self.scanner.navigator().column()
    }

    /// Reset tokenizer to beginning of source
    pub fn reset(&mut self) {
        let initial_position = if self.source.starts_with('\u{FEFF}') {
            3
        } else {
            0
        };
        self.scanner = TokenScanner::new(self.source, initial_position, 1, 1);
        self.context = TokenContext::Document;
        self.issues.clear();
    }
}
