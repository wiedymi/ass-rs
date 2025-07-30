//! Built-in syntax highlighting extension for ASS/SSA files
//!
//! Provides syntax highlighting for ASS subtitle format, including:
//! - Section headers (\\[Script Info\\], \\[Styles\\], \\[Events\\])
//! - Field names and values
//! - Override tags and their parameters
//! - Comments and special formatting

use crate::core::{EditorDocument, Position, Range, Result};
use crate::extensions::{
    EditorExtension, ExtensionCapability, ExtensionCommand, ExtensionContext, ExtensionInfo,
    ExtensionResult, ExtensionState, MessageLevel,
};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::collections::HashMap;

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Section headers like [Script Info]
    SectionHeader,
    /// Field names like Title:, PlayResX:
    FieldName,
    /// Field values
    FieldValue,
    /// Event type (Dialogue, Comment)
    EventType,
    /// Style name references
    StyleName,
    /// Time codes
    TimeCode,
    /// Override tags like {\pos(100,200)}
    OverrideTag,
    /// Tag parameters
    TagParameter,
    /// Comments
    Comment,
    /// Plain text
    Text,
    /// Errors or invalid syntax
    Error,
}

impl TokenType {
    /// Get CSS class name for web-based highlighting
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::SectionHeader => "ass-section-header",
            Self::FieldName => "ass-field-name",
            Self::FieldValue => "ass-field-value",
            Self::EventType => "ass-event-type",
            Self::StyleName => "ass-style-name",
            Self::TimeCode => "ass-timecode",
            Self::OverrideTag => "ass-override-tag",
            Self::TagParameter => "ass-tag-param",
            Self::Comment => "ass-comment",
            Self::Text => "ass-text",
            Self::Error => "ass-error",
        }
    }

    /// Get ANSI color code for terminal highlighting
    pub fn ansi_color(&self) -> &'static str {
        match self {
            Self::SectionHeader => "\x1b[1;34m", // Bright Blue
            Self::FieldName => "\x1b[36m",       // Cyan
            Self::FieldValue => "\x1b[37m",      // White
            Self::EventType => "\x1b[1;32m",     // Bright Green
            Self::StyleName => "\x1b[35m",       // Magenta
            Self::TimeCode => "\x1b[33m",        // Yellow
            Self::OverrideTag => "\x1b[1;31m",   // Bright Red
            Self::TagParameter => "\x1b[31m",    // Red
            Self::Comment => "\x1b[90m",         // Bright Black (Gray)
            Self::Text => "\x1b[0m",             // Reset
            Self::Error => "\x1b[1;91m",         // Bright Red
        }
    }
}

/// A highlighted token in the document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightToken {
    /// Range of the token in the document
    pub range: Range,
    /// Type of the token
    pub token_type: TokenType,
    /// Optional semantic information
    pub semantic_info: Option<String>,
}

/// Syntax highlighting extension
pub struct SyntaxHighlightExtension {
    info: ExtensionInfo,
    state: ExtensionState,
    /// Cached tokens for performance
    token_cache: HashMap<String, Vec<HighlightToken>>,
    /// Configuration
    config: SyntaxHighlightConfig,
}

/// Configuration for syntax highlighting
#[derive(Debug, Clone)]
pub struct SyntaxHighlightConfig {
    /// Enable semantic highlighting (slower but more accurate)
    pub semantic_highlighting: bool,
    /// Highlight override tags
    pub highlight_tags: bool,
    /// Highlight errors
    pub highlight_errors: bool,
    /// Maximum tokens to process (0 = unlimited)
    pub max_tokens: usize,
}

impl Default for SyntaxHighlightConfig {
    fn default() -> Self {
        Self {
            semantic_highlighting: true,
            highlight_tags: true,
            highlight_errors: true,
            max_tokens: 10000,
        }
    }
}

impl SyntaxHighlightExtension {
    /// Create a new syntax highlighting extension
    pub fn new() -> Self {
        let info = ExtensionInfo::new(
            "syntax-highlight".to_string(),
            "1.0.0".to_string(),
            "ASS-RS Team".to_string(),
            "Built-in syntax highlighting for ASS/SSA files".to_string(),
        )
        .with_capability(ExtensionCapability::SyntaxHighlighting)
        .with_license("MIT".to_string());

        Self {
            info,
            state: ExtensionState::Uninitialized,
            token_cache: HashMap::new(),
            config: SyntaxHighlightConfig::default(),
        }
    }

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

    /// Tokenize a Script Info line
    fn tokenize_info_line(
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
    fn tokenize_format_line(
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
    fn tokenize_style_line(
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

    /// Tokenize an Event line
    fn tokenize_event_line(
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

    /// Tokenize text with override tags
    fn tokenize_text_with_tags(
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

    /// Clear token cache
    pub fn clear_cache(&mut self) {
        self.token_cache.clear();
    }

    /// Invalidate cache for a specific document
    pub fn invalidate_document(&mut self, doc_id: &str) {
        self.token_cache.remove(doc_id);
    }
}

impl Default for SyntaxHighlightExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorExtension for SyntaxHighlightExtension {
    fn info(&self) -> &ExtensionInfo {
        &self.info
    }

    fn initialize(&mut self, context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Active;

        // Load configuration
        if let Some(semantic) = context.get_config("syntax.semantic_highlighting") {
            self.config.semantic_highlighting = semantic == "true";
        }
        if let Some(tags) = context.get_config("syntax.highlight_tags") {
            self.config.highlight_tags = tags == "true";
        }
        if let Some(errors) = context.get_config("syntax.highlight_errors") {
            self.config.highlight_errors = errors == "true";
        }
        if let Some(max_tokens) = context.get_config("syntax.max_tokens") {
            if let Ok(max) = max_tokens.parse() {
                self.config.max_tokens = max;
            }
        }

        context.show_message("Syntax highlighting initialized", MessageLevel::Info)?;
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Shutdown;
        self.clear_cache();
        Ok(())
    }

    fn state(&self) -> ExtensionState {
        self.state
    }

    fn execute_command(
        &mut self,
        command_id: &str,
        _args: &HashMap<String, String>,
        context: &mut dyn ExtensionContext,
    ) -> Result<ExtensionResult> {
        match command_id {
            "syntax.highlight" => {
                if let Some(doc) = context.current_document() {
                    let tokens = self.tokenize_document(doc)?;
                    Ok(ExtensionResult::success_with_message(format!(
                        "Document highlighted with {} tokens",
                        tokens.len()
                    )))
                } else {
                    Ok(ExtensionResult::failure(
                        "No active document to highlight".to_string(),
                    ))
                }
            }
            "syntax.clear_cache" => {
                self.clear_cache();
                Ok(ExtensionResult::success_with_message(
                    "Syntax highlight cache cleared".to_string(),
                ))
            }
            "syntax.get_tokens" => {
                if let Some(doc) = context.current_document() {
                    let tokens = self.tokenize_document(doc)?;
                    let mut result = ExtensionResult::success_with_message(format!(
                        "Found {} tokens",
                        tokens.len()
                    ));
                    result
                        .data
                        .insert("token_count".to_string(), tokens.len().to_string());
                    Ok(result)
                } else {
                    Ok(ExtensionResult::failure("No active document".to_string()))
                }
            }
            _ => Ok(ExtensionResult::failure(format!(
                "Unknown command: {command_id}"
            ))),
        }
    }

    fn commands(&self) -> Vec<ExtensionCommand> {
        vec![
            ExtensionCommand::new(
                "syntax.highlight".to_string(),
                "Highlight Document".to_string(),
                "Apply syntax highlighting to the current document".to_string(),
            )
            .with_category("Syntax".to_string()),
            ExtensionCommand::new(
                "syntax.clear_cache".to_string(),
                "Clear Highlight Cache".to_string(),
                "Clear the syntax highlighting cache".to_string(),
            )
            .with_category("Syntax".to_string())
            .requires_document(false),
            ExtensionCommand::new(
                "syntax.get_tokens".to_string(),
                "Get Highlight Tokens".to_string(),
                "Get syntax highlighting tokens for the current document".to_string(),
            )
            .with_category("Syntax".to_string()),
        ]
    }

    fn config_schema(&self) -> HashMap<String, String> {
        let mut schema = HashMap::new();
        schema.insert(
            "syntax.semantic_highlighting".to_string(),
            "boolean".to_string(),
        );
        schema.insert("syntax.highlight_tags".to_string(), "boolean".to_string());
        schema.insert("syntax.highlight_errors".to_string(), "boolean".to_string());
        schema.insert("syntax.max_tokens".to_string(), "number".to_string());
        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_types() {
        assert_eq!(TokenType::SectionHeader.css_class(), "ass-section-header");
        assert_eq!(TokenType::OverrideTag.ansi_color(), "\x1b[1;31m");
    }

    #[test]
    fn test_syntax_highlight_extension_creation() {
        let ext = SyntaxHighlightExtension::new();
        assert_eq!(ext.info().name, "syntax-highlight");
        assert!(ext
            .info()
            .has_capability(&ExtensionCapability::SyntaxHighlighting));
    }

    #[test]
    fn test_simple_tokenization() {
        let mut ext = SyntaxHighlightExtension::new();
        let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

        let tokens = ext.tokenize_document(&doc).unwrap();
        assert!(!tokens.is_empty());

        // First token should be section header
        assert_eq!(tokens[0].token_type, TokenType::SectionHeader);
        assert_eq!(tokens[0].semantic_info, Some("Script Info".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let ext = SyntaxHighlightExtension::new();
        let schema = ext.config_schema();

        assert!(schema.contains_key("syntax.semantic_highlighting"));
        assert!(schema.contains_key("syntax.highlight_tags"));
    }
}

// Include extended tests
#[cfg(test)]
#[path = "syntax_highlight_tests.rs"]
mod extended_tests;
