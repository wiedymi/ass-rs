//! Tokenizers for Script Info, Format, and Style lines.

use super::{HighlightToken, SyntaxHighlightExtension, TokenType};
use crate::core::{Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

impl SyntaxHighlightExtension {
    /// Tokenize a Script Info line
    pub(super) fn tokenize_info_line(
        &self,
        tokens: &mut Vec<HighlightToken>,
        line: &str,
        line_start: usize,
    ) -> Result<()> {
        if let Some(colon_pos) = line.find(':') {
            // Field name
            tokens.push(HighlightToken {
                range: Range::new(
                    Position::new(line_start),
                    Position::new(line_start + colon_pos + 1),
                ),
                token_type: TokenType::FieldName,
                semantic_info: Some(line[..colon_pos].to_string()),
            });

            // Field value
            let value_start = line_start + colon_pos + 1;
            let value = line[colon_pos + 1..].trim_start();
            if !value.is_empty() {
                tokens.push(HighlightToken {
                    range: Range::new(
                        Position::new(value_start),
                        Position::new(line_start + line.len()),
                    ),
                    token_type: TokenType::FieldValue,
                    semantic_info: None,
                });
            }
        }
        Ok(())
    }

    /// Tokenize a Format line
    pub(super) fn tokenize_format_line(
        &self,
        tokens: &mut Vec<HighlightToken>,
        line: &str,
        line_start: usize,
    ) -> Result<()> {
        // "Format:" part
        tokens.push(HighlightToken {
            range: Range::new(Position::new(line_start), Position::new(line_start + 7)),
            token_type: TokenType::FieldName,
            semantic_info: Some("Format".to_string()),
        });

        // Rest is field value
        tokens.push(HighlightToken {
            range: Range::new(
                Position::new(line_start + 7),
                Position::new(line_start + line.len()),
            ),
            token_type: TokenType::FieldValue,
            semantic_info: None,
        });

        Ok(())
    }

    /// Tokenize a Style line
    pub(super) fn tokenize_style_line(
        &self,
        tokens: &mut Vec<HighlightToken>,
        line: &str,
        line_start: usize,
    ) -> Result<()> {
        // "Style:" part
        tokens.push(HighlightToken {
            range: Range::new(Position::new(line_start), Position::new(line_start + 6)),
            token_type: TokenType::FieldName,
            semantic_info: Some("Style".to_string()),
        });

        // Parse style fields
        let fields = line[6..].trim_start().split(',');
        let mut field_start = line_start + 6;

        for (i, field) in fields.enumerate() {
            let field_len = field.len();

            // First field is style name
            if i == 0 {
                tokens.push(HighlightToken {
                    range: Range::new(
                        Position::new(field_start),
                        Position::new(field_start + field_len),
                    ),
                    token_type: TokenType::StyleName,
                    semantic_info: Some(field.trim().to_string()),
                });
            } else {
                tokens.push(HighlightToken {
                    range: Range::new(
                        Position::new(field_start),
                        Position::new(field_start + field_len),
                    ),
                    token_type: TokenType::FieldValue,
                    semantic_info: None,
                });
            }

            field_start += field_len + 1; // +1 for comma
        }

        Ok(())
    }
}
