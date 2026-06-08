//! Parse delta enum for streaming updates
//!
//! Defines [`ParseDelta`], the atomic change representation produced during
//! incremental parsing along with its zero-copy constructors and inspectors.

use crate::parser::ast::Section;
use alloc::string::String;

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
