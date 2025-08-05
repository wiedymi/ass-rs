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
///             ParseDelta::UpdateSection(index, section) => {
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
    /// Contains the section index and updated section data. Consumers should replace
    /// the existing section at the specified index with this new data.
    UpdateSection(usize, Section<'a>),

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
    #[must_use]
    pub const fn add_section(section: Section<'a>) -> Self {
        Self::AddSection(section)
    }

    /// Create delta for updating a section
    #[must_use]
    pub const fn update_section(index: usize, section: Section<'a>) -> Self {
        Self::UpdateSection(index, section)
    }

    /// Create delta for removing a section by index
    #[must_use]
    pub const fn remove_section(index: usize) -> Self {
        Self::RemoveSection(index)
    }

    /// Create delta for parsing issue
    #[must_use]
    pub const fn parse_issue(message: String) -> Self {
        Self::ParseIssue(message)
    }

    /// Check if delta represents an error condition
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::ParseIssue(_))
    }

    /// Check if delta modifies document structure
    #[must_use]
    pub const fn is_structural(&self) -> bool {
        matches!(
            self,
            Self::AddSection(_) | Self::UpdateSection(_, _) | Self::RemoveSection(_)
        )
    }

    /// Get section reference if delta contains one
    #[must_use]
    pub const fn section(&self) -> Option<&Section<'a>> {
        match self {
            Self::AddSection(section) | Self::UpdateSection(_, section) => Some(section),
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
    /// Collection of deltas representing changes to the script
    deltas: Vec<ParseDelta<'a>>,
}

impl<'a> DeltaBatch<'a> {
    /// Create new empty delta batch
    #[must_use]
    pub const fn new() -> Self {
        Self { deltas: Vec::new() }
    }

    /// Create batch from existing deltas
    #[must_use]
    pub const fn from_deltas(deltas: Vec<ParseDelta<'a>>) -> Self {
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
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn deltas(&self) -> &[ParseDelta<'a>] {
        &self.deltas
    }

    /// Convert to vector of deltas
    #[must_use]
    pub fn into_deltas(self) -> Vec<ParseDelta<'a>> {
        self.deltas
    }

    /// Check if batch is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    /// Get number of deltas in batch
    #[must_use]
    pub fn len(&self) -> usize {
        self.deltas.len()
    }

    /// Filter deltas by predicate
    #[must_use]
    pub fn filter<F>(&self, predicate: F) -> Self
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
    #[must_use]
    pub fn structural_only(&self) -> Self {
        self.filter(ParseDelta::is_structural)
    }

    /// Get only error deltas
    #[must_use]
    pub fn errors_only(&self) -> Self {
        self.filter(ParseDelta::is_error)
    }

    /// Check if batch contains any errors
    pub fn has_errors(&self) -> bool {
        self.deltas.iter().any(ParseDelta::is_error)
    }
}

impl Default for DeltaBatch<'_> {
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
    use crate::parser::ast::{ScriptInfo, Span};
    #[cfg(not(feature = "std"))]
    use alloc::{format, vec, string::ToString};

    #[test]
    fn delta_creation() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
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

        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
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

    #[test]
    fn delta_update_section() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let delta = ParseDelta::update_section(3, section);

        assert!(matches!(delta, ParseDelta::UpdateSection(3, _)));
        assert!(!delta.is_error());
        assert!(delta.is_structural());
        assert!(delta.section().is_some());
    }

    #[test]
    fn delta_section_getter() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });

        let add_delta = ParseDelta::add_section(section.clone());
        assert!(add_delta.section().is_some());

        let update_delta = ParseDelta::update_section(0, section);
        assert!(update_delta.section().is_some());

        let remove_delta = ParseDelta::remove_section(1);
        assert!(remove_delta.section().is_none());

        let error_delta = ParseDelta::parse_issue("Test".to_string());
        assert!(error_delta.section().is_none());
    }

    #[test]
    fn delta_debug_formatting() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let delta = ParseDelta::add_section(section);
        let debug_str = format!("{delta:?}");
        assert!(debug_str.contains("AddSection"));

        let error_delta = ParseDelta::parse_issue("Error message".to_string());
        let error_debug = format!("{error_delta:?}");
        assert!(error_debug.contains("ParseIssue"));
        assert!(error_debug.contains("Error message"));
    }

    #[test]
    fn delta_clone() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let delta = ParseDelta::add_section(section);
        let cloned = delta.clone();

        assert!(matches!(cloned, ParseDelta::AddSection(_)));
        assert_eq!(delta.is_error(), cloned.is_error());
        assert_eq!(delta.is_structural(), cloned.is_structural());
    }

    #[test]
    fn delta_all_constructors() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });

        let add = ParseDelta::add_section(section.clone());
        assert!(matches!(add, ParseDelta::AddSection(_)));

        let update = ParseDelta::update_section(42, section);
        assert!(matches!(update, ParseDelta::UpdateSection(42, _)));

        let remove = ParseDelta::remove_section(99);
        assert!(matches!(remove, ParseDelta::RemoveSection(99)));

        let issue = ParseDelta::parse_issue("Test issue".to_string());
        assert!(matches!(issue, ParseDelta::ParseIssue(_)));
    }

    #[test]
    fn batch_default() {
        let batch = DeltaBatch::default();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert!(!batch.has_errors());
    }

    #[test]
    fn batch_debug_and_clone() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let mut batch = DeltaBatch::new();
        batch.push(ParseDelta::add_section(section));

        let debug_str = format!("{batch:?}");
        assert!(debug_str.contains("DeltaBatch"));

        let cloned = batch.clone();
        assert_eq!(batch.len(), cloned.len());
        assert_eq!(batch.is_empty(), cloned.is_empty());
    }

    #[test]
    fn batch_extend_operations() {
        let mut batch = DeltaBatch::new();
        let section1 = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let section2 = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });

        let deltas = vec![
            ParseDelta::add_section(section1),
            ParseDelta::update_section(0, section2),
            ParseDelta::remove_section(1),
        ];

        batch.extend(deltas);
        assert_eq!(batch.len(), 3);
        assert!(!batch.has_errors());
    }

    #[test]
    fn batch_from_deltas() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let deltas = vec![
            ParseDelta::add_section(section),
            ParseDelta::parse_issue("Warning".to_string()),
        ];

        let batch = DeltaBatch::from_deltas(deltas);
        assert_eq!(batch.len(), 2);
        assert!(batch.has_errors());
    }

    #[test]
    fn batch_into_deltas() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let mut batch = DeltaBatch::new();
        batch.push(ParseDelta::add_section(section));
        batch.push(ParseDelta::remove_section(0));

        let deltas = batch.into_deltas();
        assert_eq!(deltas.len(), 2);
    }

    #[test]
    fn batch_complex_filtering() {
        let section1 = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let section2 = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let mut batch = DeltaBatch::new();

        batch.push(ParseDelta::add_section(section1));
        batch.push(ParseDelta::update_section(0, section2));
        batch.push(ParseDelta::remove_section(1));
        batch.push(ParseDelta::parse_issue("Error 1".to_string()));
        batch.push(ParseDelta::parse_issue("Error 2".to_string()));

        assert_eq!(batch.len(), 5);

        let structural = batch.structural_only();
        assert_eq!(structural.len(), 3);
        assert!(!structural.has_errors());

        let errors = batch.errors_only();
        assert_eq!(errors.len(), 2);
        assert!(errors.has_errors());

        // Custom filter
        let only_adds = batch.filter(|delta| matches!(delta, ParseDelta::AddSection(_)));
        assert_eq!(only_adds.len(), 1);
    }

    #[test]
    fn batch_empty_operations() {
        let batch = DeltaBatch::new();

        let structural = batch.structural_only();
        assert!(structural.is_empty());

        let errors = batch.errors_only();
        assert!(errors.is_empty());

        assert!(!batch.has_errors());
        assert_eq!(batch.deltas().len(), 0);
    }

    #[test]
    fn delta_all_variants_coverage() {
        // Test all ParseDelta variants
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });

        // AddSection
        let add = ParseDelta::AddSection(section.clone());
        assert!(add.is_structural());
        assert!(!add.is_error());
        assert!(add.section().is_some());

        // UpdateSection
        let update = ParseDelta::UpdateSection(5, section);
        assert!(update.is_structural());
        assert!(!update.is_error());
        assert!(update.section().is_some());

        // RemoveSection
        let remove = ParseDelta::RemoveSection(10);
        assert!(remove.is_structural());
        assert!(!remove.is_error());
        assert!(remove.section().is_none());

        // ParseIssue
        let issue = ParseDelta::ParseIssue("Critical error".to_string());
        assert!(!issue.is_structural());
        assert!(issue.is_error());
        assert!(issue.section().is_none());
    }

    #[test]
    fn batch_iterator_trait() {
        let section = Section::ScriptInfo(ScriptInfo {
            fields: vec![],
            span: Span::new(0, 0, 0, 0),
        });
        let deltas = [
            ParseDelta::add_section(section),
            ParseDelta::remove_section(0),
            ParseDelta::parse_issue("Test".to_string()),
        ];

        let batch: DeltaBatch = deltas.into_iter().collect();
        assert_eq!(batch.len(), 3);

        // Test that we can collect from any iterator
        let filtered_deltas = batch
            .deltas()
            .iter()
            .filter(|&d| d.is_structural())
            .cloned();
        let filtered_batch: DeltaBatch = filtered_deltas.collect();
        assert_eq!(filtered_batch.len(), 2);
    }
}
