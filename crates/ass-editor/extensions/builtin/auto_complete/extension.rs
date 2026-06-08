//! `AutoCompleteExtension` definition and completion orchestration.
//!
//! Holds the extension struct, its constructor, the top-level
//! `get_completions` entry point, and document style-name extraction. The
//! per-context completion generators live in sibling submodules.

use super::types::{AutoCompleteConfig, CompletionItem};
use crate::core::{EditorDocument, Position, Result};
use crate::extensions::{ExtensionCapability, ExtensionInfo, ExtensionState};
use ass_core::parser::{Script, Section};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Auto-completion extension
pub struct AutoCompleteExtension {
    pub(super) info: ExtensionInfo,
    pub(super) state: ExtensionState,
    /// Known style names from the document
    pub(super) style_names: Vec<String>,
    /// Configuration
    pub(super) config: AutoCompleteConfig,
}

impl AutoCompleteExtension {
    /// Create a new auto-complete extension
    pub fn new() -> Self {
        let info = ExtensionInfo::new(
            "auto-complete".to_string(),
            "1.0.0".to_string(),
            "ASS-RS Team".to_string(),
            "Built-in auto-completion for ASS/SSA files".to_string(),
        )
        .with_capability(ExtensionCapability::CodeCompletion)
        .with_license("MIT".to_string());

        Self {
            info,
            state: ExtensionState::Uninitialized,
            style_names: Vec::new(),
            config: AutoCompleteConfig::default(),
        }
    }

    /// Get completions at a position
    pub fn get_completions(
        &mut self,
        document: &EditorDocument,
        position: Position,
    ) -> Result<Vec<CompletionItem>> {
        // Update style names from document
        self.update_style_names(document)?;

        // Get completion context
        let context = self.get_completion_context(document, position)?;

        // Generate completions based on context
        let mut completions = Vec::new();

        // Section completions
        if context.line.is_empty() || context.line.starts_with('[') {
            completions.extend(self.get_section_completions(&context));
        }

        // Field completions
        if let Some(ref section) = context.section {
            if !context.in_override_tag && self.config.complete_fields {
                completions.extend(self.get_field_completions(section, &context));
            }
        }

        // Override tag completions
        if context.in_override_tag && self.config.complete_tags {
            completions.extend(self.get_tag_completions(&context));
        }

        // Style reference completions
        if self.should_complete_style(&context) && self.config.complete_styles {
            completions.extend(self.get_style_completions(&context));
        }

        // Sort and limit completions
        completions.sort_by_key(|c| c.sort_order);
        completions.truncate(self.config.max_suggestions);

        Ok(completions)
    }

    /// Update known style names from document
    pub(super) fn update_style_names(&mut self, document: &EditorDocument) -> Result<()> {
        self.style_names.clear();

        if let Ok(script) = Script::parse(&document.text()) {
            for section in script.sections() {
                if let Section::Styles(styles) = section {
                    for style in styles {
                        self.style_names.push(style.name.to_string());
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for AutoCompleteExtension {
    fn default() -> Self {
        Self::new()
    }
}
