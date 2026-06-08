//! Tokenizers for override-tag text and individual tag content.

use super::{HighlightToken, SyntaxHighlightExtension, TokenType};
use crate::core::{Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

impl SyntaxHighlightExtension {
    /// Tokenize text with override tags
    pub(super) fn tokenize_text_with_tags(
        &self,
        tokens: &mut Vec<HighlightToken>,
        text: &str,
        text_start: usize,
    ) -> Result<()> {
        let mut pos = 0;
        let bytes = text.as_bytes();

        while pos < bytes.len() {
            if bytes[pos] == b'{' {
                // Find matching }
                if let Some(end_pos) = text[pos..].find('}') {
                    let tag_content = &text[pos + 1..pos + end_pos];

                    // Opening brace
                    tokens.push(HighlightToken {
                        range: Range::new(
                            Position::new(text_start + pos),
                            Position::new(text_start + pos + 1),
                        ),
                        token_type: TokenType::OverrideTag,
                        semantic_info: None,
                    });

                    // Tag content
                    self.tokenize_tag_content(tokens, tag_content, text_start + pos + 1)?;

                    // Closing brace
                    tokens.push(HighlightToken {
                        range: Range::new(
                            Position::new(text_start + pos + end_pos),
                            Position::new(text_start + pos + end_pos + 1),
                        ),
                        token_type: TokenType::OverrideTag,
                        semantic_info: None,
                    });

                    pos += end_pos + 1;
                } else {
                    // Unclosed tag - mark as error
                    tokens.push(HighlightToken {
                        range: Range::new(
                            Position::new(text_start + pos),
                            Position::new(text_start + text.len()),
                        ),
                        token_type: TokenType::Error,
                        semantic_info: Some("Unclosed override tag".to_string()),
                    });
                    break;
                }
            } else {
                // Find next tag or end of text
                let next_tag = text[pos..].find('{').unwrap_or(text.len() - pos);

                if next_tag > 0 {
                    tokens.push(HighlightToken {
                        range: Range::new(
                            Position::new(text_start + pos),
                            Position::new(text_start + pos + next_tag),
                        ),
                        token_type: TokenType::Text,
                        semantic_info: None,
                    });
                }

                pos += next_tag;
            }
        }

        Ok(())
    }

    /// Tokenize tag content
    fn tokenize_tag_content(
        &self,
        tokens: &mut Vec<HighlightToken>,
        content: &str,
        content_start: usize,
    ) -> Result<()> {
        // Simple tag parsing - could be enhanced
        let parts = content.split('\\').filter(|s| !s.is_empty());
        let mut pos = 0;

        for part in parts {
            // Skip initial backslash positions
            while pos < content.len() && content.as_bytes()[pos] == b'\\' {
                pos += 1;
            }

            if pos >= content.len() {
                break;
            }

            // Find tag name and parameters
            let tag_end = part
                .find(|c: char| !c.is_alphanumeric())
                .unwrap_or(part.len());

            if tag_end > 0 {
                // Tag name
                tokens.push(HighlightToken {
                    range: Range::new(
                        Position::new(content_start + pos),
                        Position::new(content_start + pos + tag_end),
                    ),
                    token_type: TokenType::OverrideTag,
                    semantic_info: Some(part[..tag_end].to_string()),
                });

                // Parameters
                if tag_end < part.len() {
                    tokens.push(HighlightToken {
                        range: Range::new(
                            Position::new(content_start + pos + tag_end),
                            Position::new(content_start + pos + part.len()),
                        ),
                        token_type: TokenType::TagParameter,
                        semantic_info: None,
                    });
                }
            }

            pos += part.len() + 1; // +1 for backslash
        }

        Ok(())
    }
}
