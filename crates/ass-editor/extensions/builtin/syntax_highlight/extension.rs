//! Core extension struct, configuration, and construction/cache helpers.

use super::HighlightToken;
use crate::extensions::{ExtensionCapability, ExtensionInfo, ExtensionState};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::collections::HashMap;

/// Syntax highlighting extension
pub struct SyntaxHighlightExtension {
    pub(super) info: ExtensionInfo,
    pub(super) state: ExtensionState,
    /// Cached tokens for performance
    pub(super) token_cache: HashMap<String, Vec<HighlightToken>>,
    /// Configuration
    pub(super) config: SyntaxHighlightConfig,
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
