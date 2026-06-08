//! `EditorExtension` trait implementation for `AutoCompleteExtension`.
//!
//! Wires the auto-completion logic into the editor extension lifecycle:
//! initialization, configuration loading, command execution, and schema.

use super::extension::AutoCompleteExtension;
use crate::core::{Position, Result};
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

impl EditorExtension for AutoCompleteExtension {
    fn info(&self) -> &ExtensionInfo {
        &self.info
    }

    fn initialize(&mut self, context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Active;

        // Load configuration
        if let Some(fields) = context.get_config("autocomplete.complete_fields") {
            self.config.complete_fields = fields == "true";
        }
        if let Some(styles) = context.get_config("autocomplete.complete_styles") {
            self.config.complete_styles = styles == "true";
        }
        if let Some(tags) = context.get_config("autocomplete.complete_tags") {
            self.config.complete_tags = tags == "true";
        }
        if let Some(values) = context.get_config("autocomplete.complete_values") {
            self.config.complete_values = values == "true";
        }
        if let Some(max) = context.get_config("autocomplete.max_suggestions") {
            if let Ok(max_val) = max.parse() {
                self.config.max_suggestions = max_val;
            }
        }

        context.show_message("Auto-completion initialized", MessageLevel::Info)?;
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Shutdown;
        self.style_names.clear();
        Ok(())
    }

    fn state(&self) -> ExtensionState {
        self.state
    }

    fn execute_command(
        &mut self,
        command_id: &str,
        args: &HashMap<String, String>,
        context: &mut dyn ExtensionContext,
    ) -> Result<ExtensionResult> {
        match command_id {
            "autocomplete.trigger" => {
                if let Some(doc) = context.current_document() {
                    // Get position from args or use end of document
                    let position = if let Some(offset_str) = args.get("position") {
                        if let Ok(offset) = offset_str.parse() {
                            Position::new(offset)
                        } else {
                            Position::new(doc.len_bytes())
                        }
                    } else {
                        Position::new(doc.len_bytes())
                    };

                    let completions = self.get_completions(doc, position)?;
                    let mut result = ExtensionResult::success_with_message(format!(
                        "Found {} completions",
                        completions.len()
                    ));

                    // Add completion data
                    for (i, completion) in completions.iter().take(10).enumerate() {
                        result
                            .data
                            .insert(format!("completion_{i}"), completion.insert_text.clone());
                    }

                    Ok(result)
                } else {
                    Ok(ExtensionResult::failure("No active document".to_string()))
                }
            }
            "autocomplete.update_styles" => {
                if let Some(doc) = context.current_document() {
                    self.update_style_names(doc)?;
                    Ok(ExtensionResult::success_with_message(format!(
                        "Updated {} style names",
                        self.style_names.len()
                    )))
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
                "autocomplete.trigger".to_string(),
                "Trigger Completion".to_string(),
                "Get completion suggestions at cursor position".to_string(),
            )
            .with_category("Completion".to_string()),
            ExtensionCommand::new(
                "autocomplete.update_styles".to_string(),
                "Update Style Names".to_string(),
                "Update known style names from document".to_string(),
            )
            .with_category("Completion".to_string()),
        ]
    }

    fn config_schema(&self) -> HashMap<String, String> {
        let mut schema = HashMap::new();
        schema.insert(
            "autocomplete.complete_fields".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.complete_styles".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.complete_tags".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.complete_values".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.max_suggestions".to_string(),
            "number".to_string(),
        );
        schema.insert("autocomplete.min_chars".to_string(), "number".to_string());
        schema
    }
}
