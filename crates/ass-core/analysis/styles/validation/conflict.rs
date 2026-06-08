//! Style conflict detection and classification.
//!
//! Provides [`StyleConflict`] and [`ConflictType`] for detecting and
//! describing conflicts such as duplicate names, circular inheritance, and
//! missing references between styles.

use alloc::{format, string::String, string::ToString, vec, vec::Vec};
use core::fmt;

use super::ValidationSeverity;

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

impl<'a> StyleConflict<'a> {
    /// Create new style conflict
    #[must_use]
    pub fn new(
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
    #[must_use]
    pub fn duplicate_name(style_names: Vec<&'a str>) -> Self {
        let description = format!("Duplicate style names found: {style_names:?}");
        Self::new(
            ConflictType::DuplicateName,
            style_names,
            &description,
            ValidationSeverity::Error,
        )
    }

    /// Create circular inheritance conflict
    #[must_use]
    pub fn circular_inheritance(cycle_styles: Vec<&'a str>) -> Self {
        let description = format!("Circular inheritance detected: {cycle_styles:?}");
        Self::new(
            ConflictType::CircularInheritance,
            cycle_styles,
            &description,
            ValidationSeverity::Error,
        )
    }

    /// Create missing reference conflict
    #[must_use]
    pub fn missing_reference(referencing_style: &'a str, missing_style: &'a str) -> Self {
        let description =
            format!("Style '{referencing_style}' references non-existent style '{missing_style}'");
        Self::new(
            ConflictType::MissingReference,
            vec![referencing_style],
            &description,
            ValidationSeverity::Error,
        )
    }

    /// Create missing parent conflict
    #[must_use]
    pub fn missing_parent(style_name: &'a str, parent_name: &'a str) -> Self {
        let description =
            format!("Style '{style_name}' inherits from non-existent parent '{parent_name}'");
        Self::new(
            ConflictType::MissingReference,
            vec![style_name],
            &description,
            ValidationSeverity::Warning,
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
