//! Top-level document tokenization driving the per-line tokenizers.

use super::{HighlightToken, SyntaxHighlightExtension, TokenType};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

impl SyntaxHighlightExtension {
    /// Tokenize a document
    pub fn tokenize_document(&mut self, document: &EditorDocument) -> Result<Vec<HighlightToken>> {
        let content = document.text();
        let doc_id = document.id();

        // Check cache
        if let Some(cached_tokens) = self.token_cache.get(doc_id) {
            return Ok(cached_tokens.clone());
        }

        let mut tokens = Vec::new();
        let mut current_section = None;
        let mut line_start = 0;

        for line in content.lines() {
            let line_range = Range::new(
                Position::new(line_start),
                Position::new(line_start + line.len()),
            );

            // Handle section headers
            if line.starts_with('[') && line.ends_with(']') {
                tokens.push(HighlightToken {
                    range: line_range,
                    token_type: TokenType::SectionHeader,
                    semantic_info: Some(line[1..line.len() - 1].to_string()),
                });
                current_section = Some(line[1..line.len() - 1].to_string());
            }
            // Handle comments (only lines starting with semicolon)
            else if line.starts_with(';') {
                tokens.push(HighlightToken {
                    range: line_range,
                    token_type: TokenType::Comment,
                    semantic_info: None,
                });
            }
            // Handle fields based on current section
            else if let Some(ref section) = current_section {
                match section.as_str() {
                    "Script Info" | "Aegisub Project Garbage" => {
                        self.tokenize_info_line(&mut tokens, line, line_start)?;
                    }
                    "V4+ Styles" | "V4 Styles" => {
                        if line.starts_with("Format:") {
                            self.tokenize_format_line(&mut tokens, line, line_start)?;
                        } else if line.starts_with("Style:") {
                            self.tokenize_style_line(&mut tokens, line, line_start)?;
                        }
                    }
                    "Events" => {
                        if line.starts_with("Format:") {
                            self.tokenize_format_line(&mut tokens, line, line_start)?;
                        } else if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                            self.tokenize_event_line(&mut tokens, line, line_start)?;
                        }
                    }
                    _ => {
                        // Unknown section - highlight as text
                        tokens.push(HighlightToken {
                            range: line_range,
                            token_type: TokenType::Text,
                            semantic_info: None,
                        });
                    }
                }
            }

            line_start += line.len() + 1; // +1 for newline

            // Check token limit
            if self.config.max_tokens > 0 && tokens.len() >= self.config.max_tokens {
                break;
            }
        }

        // Cache the tokens
        self.token_cache.insert(doc_id.to_string(), tokens.clone());

        Ok(tokens)
    }
}
