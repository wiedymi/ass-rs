//! Core data types for override-tag analysis.
//!
//! Defines the [`OverrideTag`] parsed-tag representation along with the
//! [`TagDiagnostic`] and [`DiagnosticKind`] types used to report malformed or
//! empty override syntax during parsing.

use alloc::string::String;

/// Diagnostic information for tag parsing issues
#[derive(Debug, Clone)]
pub struct TagDiagnostic<'a> {
    /// Text span containing the issue
    pub span: &'a str,
    /// Byte offset in original text
    pub offset: usize,
    /// Type of diagnostic issue
    pub kind: DiagnosticKind,
}

/// Types of tag parsing diagnostics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Empty override tag like {}
    EmptyOverride,
    /// Malformed tag syntax
    MalformedTag,
    /// Unknown or invalid tag name
    UnknownTag(String),
}

/// Single ASS override tag with analysis results
///
/// Represents a parsed override tag like `{\b1}` or `{\pos(100,200)}`.
/// Contains zero-copy references to original text for efficiency.
#[derive(Debug, Clone)]
pub struct OverrideTag<'a> {
    /// Tag name (e.g., "b", "pos", "move")
    pub(super) name: &'a str,
    /// Tag arguments as original text slice
    pub(super) args: &'a str,
    /// Complexity score for rendering (0-5)
    pub(super) complexity: u8,
    /// Byte position in original text
    pub(super) position: usize,
}

impl<'a> OverrideTag<'a> {
    /// Create a new override tag
    #[must_use]
    pub const fn new(name: &'a str, args: &'a str, complexity: u8, position: usize) -> Self {
        Self {
            name,
            args,
            complexity,
            position,
        }
    }

    /// Get tag name
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Get tag arguments
    #[must_use]
    pub const fn args(&self) -> &'a str {
        self.args
    }

    /// Get complexity score
    #[must_use]
    pub const fn complexity(&self) -> u8 {
        self.complexity
    }

    /// Get position in original text
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
}
