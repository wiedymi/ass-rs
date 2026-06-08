//! Unit tests for style validation issue, inheritance, and conflict types.

use super::*;
use alloc::vec;

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
