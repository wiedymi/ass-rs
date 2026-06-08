//! Tokenizer for Dialogue/Comment event lines.

use super::{HighlightToken, SyntaxHighlightExtension, TokenType};
use crate::core::{Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

impl SyntaxHighlightExtension {
    /// Tokenize an Event line
    pub(super) fn tokenize_event_line(
        &self,
        tokens: &mut Vec<HighlightToken>,
        line: &str,
        line_start: usize,
    ) -> Result<()> {
        let event_type = if line.starts_with("Dialogue:") {
            "Dialogue"
        } else {
            "Comment"
        };

        // Event type
        let type_len = event_type.len() + 1; // +1 for colon
        tokens.push(HighlightToken {
            range: Range::new(
                Position::new(line_start),
                Position::new(line_start + type_len),
            ),
            token_type: TokenType::EventType,
            semantic_info: Some(event_type.to_string()),
        });

        // Parse event fields
        let fields_start = line_start + type_len;
        let fields_text = &line[type_len..];

        // Find the text field (last field after 9 commas)
        let mut comma_count = 0;
        let mut text_start = None;

        for (i, ch) in fields_text.char_indices() {
            if ch == ',' {
                comma_count += 1;
                if comma_count == 9 {
                    text_start = Some(i + 1);
                    break;
                }
            }
        }

        // Tokenize fields before text
        if let Some(text_offset) = text_start {
            let pre_text = &fields_text[..text_offset];
            let mut field_start = fields_start;

            for (i, field) in pre_text.split(',').enumerate() {
                let field_len = field.len();

                match i {
                    1 | 2 => {
                        // Start and End times
                        tokens.push(HighlightToken {
                            range: Range::new(
                                Position::new(field_start),
                                Position::new(field_start + field_len),
                            ),
                            token_type: TokenType::TimeCode,
                            semantic_info: None,
                        });
                    }
                    3 => {
                        // Style name
                        tokens.push(HighlightToken {
                            range: Range::new(
                                Position::new(field_start),
                                Position::new(field_start + field_len),
                            ),
                            token_type: TokenType::StyleName,
                            semantic_info: Some(field.trim().to_string()),
                        });
                    }
                    _ => {
                        // Other fields
                        tokens.push(HighlightToken {
                            range: Range::new(
                                Position::new(field_start),
                                Position::new(field_start + field_len),
                            ),
                            token_type: TokenType::FieldValue,
                            semantic_info: None,
                        });
                    }
                }

                field_start += field_len + 1; // +1 for comma
            }

            // Tokenize text field with override tags
            if self.config.highlight_tags {
                let text_field = &fields_text[text_offset..];
                self.tokenize_text_with_tags(tokens, text_field, fields_start + text_offset)?;
            } else {
                // Just mark as text
                tokens.push(HighlightToken {
                    range: Range::new(
                        Position::new(fields_start + text_offset),
                        Position::new(line_start + line.len()),
                    ),
                    token_type: TokenType::Text,
                    semantic_info: None,
                });
            }
        }

        Ok(())
    }
}
