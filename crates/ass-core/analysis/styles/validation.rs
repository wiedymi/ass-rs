//! Style validation and conflict detection for ASS subtitle styles
//!
//! Provides comprehensive validation of style definitions including property
//! validation, inheritance analysis, and conflict detection. Designed for
//! editor integration and script quality assurance.
//!
//! # Features
//!
//! - Property validation with configurable severity levels
//! - Circular inheritance detection and resolution
//! - Duplicate name and missing reference detection
//! - Performance impact assessment for validation issues
//! - Zero-copy validation with lifetime-generic references
//!
//! # Performance
//!
//! - Target: <0.5ms per style validation
//! - Memory: Minimal allocations via zero-copy issue references
//! - Validation: Configurable depth limits for inheritance analysis

use alloc::{format, string::String, vec::Vec};
use core::fmt;

/// Severity level for style validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational message about style properties
    Info,
    /// Warning about potential rendering or performance issues
    Warning,
    /// Error that violates ASS specification or causes problems
    Error,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Style validation issue with context and suggestions
#[derive(Debug, Clone)]
pub struct StyleValidationIssue {
    /// Issue severity level
    pub severity: ValidationSeverity,
    /// Human-readable issue description
    pub message: String,
    /// Style field that caused the issue
    pub field: String,
    /// Optional suggested fix or improvement
    pub suggestion: Option<String>,
}

/// Style inheritance tracking and analysis
#[derive(Debug, Clone)]
pub struct StyleInheritance<'a> {
    /// Style name (zero-copy reference)
    pub name: &'a str,
    /// Parent styles in inheritance chain
    pub parents: Vec<&'a str>,
    /// Child styles that inherit from this style
    pub children: Vec<&'a str>,
    /// Inheritance depth (0 = root style, no parents)
    pub depth: usize,
    /// Whether circular inheritance was detected
    pub has_circular_inheritance: bool,
}

/// Style conflict detection and classification
#[derive(Debug, Clone)]
pub struct StyleConflict<'a> {
    /// Names of conflicting styles
    pub styles: Vec<&'a str>,
    /// Type of conflict detected
    pub conflict_type: ConflictType,
    /// Detailed conflict description
    pub description: String,
    /// Severity of the conflict
    pub severity: ValidationSeverity,
}

/// Types of style conflicts that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    /// Multiple styles with identical names
    DuplicateName,
    /// Circular reference in style inheritance
    CircularInheritance,
    /// Conflicting property values between related styles
    PropertyConflict,
    /// Reference to non-existent style
    MissingReference,
}

impl StyleValidationIssue {
    /// Create new validation issue
    #[must_use] pub fn new(
        severity: ValidationSeverity,
        field: &str,
        message: &str,
        suggestion: Option<&str>,
    ) -> Self {
        Self {
            severity,
            message: message.to_string(),
            field: field.to_string(),
            suggestion: suggestion.map(std::string::ToString::to_string),
        }
    }

    /// Create error-level issue
    #[must_use] pub fn error(field: &str, message: &str) -> Self {
        Self::new(ValidationSeverity::Error, field, message, None)
    }

    /// Create warning-level issue
    #[must_use] pub fn warning(field: &str, message: &str) -> Self {
        Self::new(ValidationSeverity::Warning, field, message, None)
    }

    /// Create info-level issue with suggestion
    #[must_use] pub fn info_with_suggestion(field: &str, message: &str, suggestion: &str) -> Self {
        Self::new(ValidationSeverity::Info, field, message, Some(suggestion))
    }
}

impl<'a> StyleInheritance<'a> {
    /// Create new inheritance tracker for style
    #[must_use] pub const fn new(name: &'a str) -> Self {
        Self {
            name,
            parents: Vec::new(),
            children: Vec::new(),
            depth: 0,
            has_circular_inheritance: false,
        }
    }

    /// Add parent style to inheritance chain
    pub fn add_parent(&mut self, parent: &'a str) {
        if !self.parents.contains(&parent) {
            self.parents.push(parent);
        }
    }

    /// Add child style that inherits from this one
    pub fn add_child(&mut self, child: &'a str) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Check if style has inheritance relationships
    #[must_use] pub fn has_inheritance(&self) -> bool {
        !self.parents.is_empty() || !self.children.is_empty()
    }

    /// Check if style is root (no parents)
    #[must_use] pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    /// Check if style is leaf (no children)
    #[must_use] pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

impl<'a> StyleConflict<'a> {
    /// Create new style conflict
    #[must_use] pub fn new(
        conflict_type: ConflictType,
        styles: Vec<&'a str>,
        description: &str,
        severity: ValidationSeverity,
    ) -> Self {
        Self {
            styles,
            conflict_type,
            description: description.to_string(),
            severity,
        }
    }

    /// Create duplicate name conflict
    #[must_use] pub fn duplicate_name(style_names: Vec<&'a str>) -> Self {
        let description = format!("Duplicate style names found: {style_names:?}");
        Self::new(
            ConflictType::DuplicateName,
            style_names,
            &description,
            ValidationSeverity::Error,
        )
    }

    /// Create circular inheritance conflict
    #[must_use] pub fn circular_inheritance(cycle_styles: Vec<&'a str>) -> Self {
        let description = format!("Circular inheritance detected: {cycle_styles:?}");
        Self::new(
            ConflictType::CircularInheritance,
            cycle_styles,
            &description,
            ValidationSeverity::Error,
        )
    }

    /// Create missing reference conflict
    #[must_use] pub fn missing_reference(referencing_style: &'a str, missing_style: &'a str) -> Self {
        let description = format!(
            "Style '{referencing_style}' references non-existent style '{missing_style}'"
        );
        Self::new(
            ConflictType::MissingReference,
            vec![referencing_style],
            &description,
            ValidationSeverity::Error,
        )
    }
}

impl fmt::Display for ConflictType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateName => write!(f, "duplicate_name"),
            Self::CircularInheritance => write!(f, "circular_inheritance"),
            Self::PropertyConflict => write!(f, "property_conflict"),
            Self::MissingReference => write!(f, "missing_reference"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_issue_creation() {
        let issue = StyleValidationIssue::error("font_size", "Invalid font size value");
        assert_eq!(issue.severity, ValidationSeverity::Error);
        assert_eq!(issue.field, "font_size");
        assert!(issue.suggestion.is_none());

        let info = StyleValidationIssue::info_with_suggestion(
            "font_name",
            "Font may not be available",
            "Use Arial as fallback",
        );
        assert_eq!(info.severity, ValidationSeverity::Info);
        assert!(info.suggestion.is_some());
    }

    #[test]
    fn inheritance_tracking() {
        let mut inheritance = StyleInheritance::new("Child");
        assert!(inheritance.is_root());
        assert!(inheritance.is_leaf());

        inheritance.add_parent("Parent");
        assert!(!inheritance.is_root());
        assert!(inheritance.has_inheritance());

        inheritance.add_child("Grandchild");
        assert!(!inheritance.is_leaf());
    }

    #[test]
    fn conflict_creation() {
        let conflict = StyleConflict::duplicate_name(vec!["Style1", "Style1"]);
        assert_eq!(conflict.conflict_type, ConflictType::DuplicateName);
        assert_eq!(conflict.severity, ValidationSeverity::Error);

        let missing = StyleConflict::missing_reference("Child", "MissingParent");
        assert_eq!(missing.conflict_type, ConflictType::MissingReference);
        assert!(missing.description.contains("MissingParent"));
    }

    #[test]
    fn severity_ordering() {
        assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
    }
}
