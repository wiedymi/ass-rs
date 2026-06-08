//! `TextAnalysis` type definition and analysis entry points.
//!
//! Defines the [`TextAnalysis`] result structure plus the public `analyze`
//! constructors. The bulk of the parsing logic lives in the `parser` submodule
//! and the accessors plus Unicode helpers live in the `helpers` submodule.

use crate::{
    analysis::events::tags::{OverrideTag, TagDiagnostic},
    Result,
};

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;
use alloc::{string::String, vec::Vec};

/// Analysis results for dialogue text content
///
/// Contains extracted plain text, override tag information, and Unicode
/// complexity indicators. Uses zero-copy references where possible.
#[derive(Debug, Clone)]
pub struct TextAnalysis<'a> {
    /// Plain text with override tags removed
    pub(super) plain_text: String,
    /// Unicode character count
    pub(super) char_count: usize,
    /// Line count after processing linebreaks
    pub(super) line_count: usize,
    /// Contains bidirectional text (RTL scripts)
    pub(super) has_bidi_text: bool,
    /// Contains complex Unicode beyond basic Latin
    pub(super) has_complex_unicode: bool,
    /// Parsed override tags
    pub(super) override_tags: Vec<OverrideTag<'a>>,
    /// Parse diagnostics collected during analysis
    pub(super) parse_diagnostics: Vec<TagDiagnostic<'a>>,
}

impl<'a> TextAnalysis<'a> {
    /// Analyze dialogue text content comprehensively
    ///
    /// Extracts plain text, parses override tags, and analyzes Unicode
    /// complexity. Uses zero-copy references for tag arguments.
    ///
    /// # Arguments
    ///
    /// * `text` - Original dialogue text with potential override tags
    ///
    /// # Returns
    ///
    /// Complete text analysis results or parsing error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::analysis::events::text_analysis::TextAnalysis;
    /// let text = "Hello {\\b1}world{\\b0}!";
    /// let analysis = TextAnalysis::analyze(text)?;
    /// assert_eq!(analysis.plain_text(), "Hello world!");
    /// assert_eq!(analysis.override_tags().len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if text parsing fails or contains invalid override tags.
    pub fn analyze(text: &'a str) -> Result<Self> {
        #[cfg(feature = "plugins")]
        return Self::analyze_with_registry(text, None);
        #[cfg(not(feature = "plugins"))]
        return Self::analyze_impl(text);
    }

    /// Analyze dialogue text content with extension registry support
    ///
    /// Same as [`analyze`](Self::analyze) but allows custom tag handlers via registry.
    /// Unhandled tags fall back to standard processing.
    ///
    /// # Arguments
    ///
    /// * `text` - Original dialogue text with potential override tags
    /// * `registry` - Optional registry for custom tag handlers
    ///
    /// # Returns
    ///
    /// Complete text analysis results or parsing error.
    ///
    /// # Errors
    ///
    /// Returns an error if text parsing fails or contains invalid override tags.
    #[cfg(feature = "plugins")]
    pub fn analyze_with_registry(
        text: &'a str,
        registry: Option<&ExtensionRegistry>,
    ) -> Result<Self> {
        Self::analyze_impl_with_registry(text, registry)
    }

    /// Internal implementation without plugins support
    #[cfg(not(feature = "plugins"))]
    fn analyze_impl(text: &'a str) -> Result<Self> {
        Self::analyze_impl_with_registry(text)
    }
}
