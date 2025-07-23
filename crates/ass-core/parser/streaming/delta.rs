//! Parse delta operations for streaming updates
//!
//! Provides delta tracking for efficient incremental parsing and editor
//! integration. Deltas represent minimal changes between parsing states.

use crate::parser::ast::Section;
use alloc::{string::String, vec::Vec};

/// Delta operations for streaming updates
///
/// Represents atomic changes detected during incremental parsing.
/// Used by editors and streaming applications to efficiently update
/// internal state without full re-parsing.
///
/// # Performance
///
/// Deltas use zero-copy design where possible to minimize allocations.
/// Section references point to parsed AST nodes using lifetime parameters.
///
/// # Example
///
/// ```rust
/// use ass_core::parser::streaming::ParseDelta;
///
/// // Handle delta operations
/// fn apply_deltas(deltas: Vec<ParseDelta>) {
///     for delta in deltas {
///         match delta {
///             ParseDelta::AddSection(section) => {
///                 // Add new section to document
///             }
///             ParseDelta::UpdateSection(section) => {
///                 // Update existing section
///             }
///             ParseDelta::RemoveSection(index) => {
///                 // Remove section at index
///             }
///             ParseDelta::ParseIssue(issue) => {
///                 // Handle parsing error/warning
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum ParseDelta<'a> {
    /// Section was added to the script
    ///
    /// Contains the complete parsed section with zero-copy references
    /// to source text spans.
    AddSection(Section<'a>),

    /// Section was modified during incremental parsing
    ///
    /// Contains the updated section data. Consumers should replace
    /// the existing section with this new data.
    UpdateSection(Section<'a>),

    /// Section was removed by index
    ///
    /// Contains the zero-based index of the section that should be
    /// removed from the script.
    RemoveSection(usize),

    /// Parsing issue detected during processing
    ///
    /// Contains error or warning message about parsing problems.
    /// Parsing may continue despite issues for error recovery.
    ParseIssue(String),
}

impl<'a> ParseDelta<'a> {
    /// Create delta for adding a section
    pub fn add_section(section: Section<'a>) -> Self {
        Self::AddSection(section)
    }

    /// Create delta for updating a section
    pub fn update_section(section: Section<'a>) -> Self {
        Self::UpdateSection(section)
    }

    /// Create delta for removing a section by index
    pub fn remove_section(index: usize) -> Self {
        Self::RemoveSection(index)
    }

    /// Create delta for parsing issue
    pub fn parse_issue(message: String) -> Self {
        Self::ParseIssue(message)
    }

    /// Check if delta represents an error condition
    pub fn is_error(&self) -> bool {
        matches!(self, Self::ParseIssue(_))
    }

    /// Check if delta modifies document structure
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            Self::AddSection(_) | Self::UpdateSection(_) | Self::RemoveSection(_)
        )
    }

    /// Get section reference if delta contains one
    pub fn section(&self) -> Option<&Section<'a>> {
        match self {
            Self::AddSection(section) | Self::UpdateSection(section) => Some(section),
            _ => None,
        }
    }
}

/// Collection of parse deltas with batch operations
///
/// Provides utilities for working with multiple deltas efficiently,
/// including filtering, merging, and validation.
#[derive(Debug, Clone)]
pub struct DeltaBatch<'a> {
    deltas: Vec<ParseDelta<'a>>,
}

impl<'a> DeltaBatch<'a> {
    /// Create new empty delta batch
    pub fn new() -> Self {
        Self { deltas: Vec::new() }
    }

    /// Create batch from existing deltas
    pub fn from_deltas(deltas: Vec<ParseDelta<'a>>) -> Self {
        Self { deltas }
    }

    /// Add delta to batch
    pub fn push(&mut self, delta: ParseDelta<'a>) {
        self.deltas.push(delta);
    }

    /// Extend batch with multiple deltas
    pub fn extend(&mut self, other_deltas: impl IntoIterator<Item = ParseDelta<'a>>) {
        self.deltas.extend(other_deltas);
    }

    /// Get all deltas
    pub fn deltas(&self) -> &[ParseDelta<'a>] {
        &self.deltas
    }

    /// Convert to vector of deltas
    pub fn into_deltas(self) -> Vec<ParseDelta<'a>> {
        self.deltas
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    /// Get number of deltas in batch
    pub fn len(&self) -> usize {
        self.deltas.len()
    }

    /// Filter deltas by predicate
    pub fn filter<F>(&self, predicate: F) -> DeltaBatch<'a>
    where
        F: Fn(&ParseDelta<'a>) -> bool,
    {
        let filtered = self
            .deltas
            .iter()
            .filter(|d| predicate(d))
            .cloned()
            .collect();
        DeltaBatch::from_deltas(filtered)
    }

    /// Get only structural deltas (add/update/remove)
    pub fn structural_only(&self) -> DeltaBatch<'a> {
        self.filter(|d| d.is_structural())
    }

    /// Get only error deltas
    pub fn errors_only(&self) -> DeltaBatch<'a> {
        self.filter(|d| d.is_error())
    }

    /// Check if batch contains any errors
    pub fn has_errors(&self) -> bool {
        self.deltas.iter().any(|d| d.is_error())
    }
}

impl<'a> Default for DeltaBatch<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FromIterator<ParseDelta<'a>> for DeltaBatch<'a> {
    fn from_iter<T: IntoIterator<Item = ParseDelta<'a>>>(iter: T) -> Self {
        Self::from_deltas(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{ScriptInfo, Section};
    use alloc::vec;

    #[test]
    fn delta_creation() {
        let section = Section::ScriptInfo(ScriptInfo { fields: vec![] });
        let delta = ParseDelta::add_section(section);
        assert!(matches!(delta, ParseDelta::AddSection(_)));
        assert!(!delta.is_error());
        assert!(delta.is_structural());
    }

    #[test]
    fn delta_properties() {
        let remove_delta = ParseDelta::remove_section(5);
        assert!(remove_delta.is_structural());
        assert!(!remove_delta.is_error());
        assert_eq!(remove_delta.section(), None);

        let error_delta = ParseDelta::parse_issue("Test error".to_string());
        assert!(!error_delta.is_structural());
        assert!(error_delta.is_error());
    }

    #[test]
    fn delta_batch_operations() {
        let mut batch = DeltaBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);

        let section = Section::ScriptInfo(ScriptInfo { fields: vec![] });
        batch.push(ParseDelta::add_section(section));
        batch.push(ParseDelta::parse_issue("Warning".to_string()));

        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 2);
        assert!(batch.has_errors());

        let structural = batch.structural_only();
        assert_eq!(structural.len(), 1);
        assert!(!structural.has_errors());

        let errors = batch.errors_only();
        assert_eq!(errors.len(), 1);
        assert!(errors.has_errors());
    }

    #[test]
    fn batch_from_iterator() {
        let deltas = vec![
            ParseDelta::remove_section(0),
            ParseDelta::parse_issue("Error".to_string()),
        ];

        let batch: DeltaBatch = deltas.into_iter().collect();
        assert_eq!(batch.len(), 2);
    }
}
