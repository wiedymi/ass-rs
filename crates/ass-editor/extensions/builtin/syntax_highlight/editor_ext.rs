//! `EditorExtension` trait implementation for the syntax highlighter.

use super::SyntaxHighlightExtension;
use crate::core::Result;
use crate::extensions::{
    EditorExtension, ExtensionCommand, ExtensionContext, ExtensionInfo, ExtensionResult,
    ExtensionState, MessageLevel,
};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
#[cfg(feature = "std")]
use std::collections::HashMap;

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
