//! Unit tests for the [`DelimiterType`] enum.

use super::*;

#[test]
fn delimiter_type_matching() {
    assert!(DelimiterType::FieldSeparator.matches(':'));
    assert!(DelimiterType::ValueSeparator.matches(','));
    assert!(DelimiterType::SectionBoundary.matches('['));
    assert!(DelimiterType::SectionBoundary.matches(']'));
    assert!(DelimiterType::LineTerminator.matches('\n'));

    assert!(!DelimiterType::FieldSeparator.matches(','));
    assert!(!DelimiterType::ValueSeparator.matches(':'));
}

#[test]
fn all_delimiter_types_chars() {
    assert_eq!(DelimiterType::FieldSeparator.chars(), &[':']);
    assert_eq!(DelimiterType::ValueSeparator.chars(), &[',']);
    assert_eq!(DelimiterType::SectionBoundary.chars(), &['[', ']']);
    assert_eq!(DelimiterType::OverrideBoundary.chars(), &['{', '}']);
    assert_eq!(DelimiterType::CommentMarker.chars(), &[';']);
    assert_eq!(DelimiterType::LineTerminator.chars(), &['\n', '\r']);
    assert_eq!(DelimiterType::DrawingSeparator.chars(), &[' ', '\t']);
    assert_eq!(DelimiterType::TimeSeparator.chars(), &[':', '.']);
    assert_eq!(DelimiterType::ColorSeparator.chars(), &['&', 'H']);
}

#[test]
fn field_and_value_separator_matching() {
    assert!(DelimiterType::FieldSeparator.matches(':'));
    assert!(!DelimiterType::FieldSeparator.matches(','));
    assert!(!DelimiterType::FieldSeparator.matches('['));

    assert!(DelimiterType::ValueSeparator.matches(','));
    assert!(!DelimiterType::ValueSeparator.matches(':'));
    assert!(!DelimiterType::ValueSeparator.matches('['));
}

#[test]
fn boundary_delimiter_matching() {
    assert!(DelimiterType::SectionBoundary.matches('['));
    assert!(DelimiterType::SectionBoundary.matches(']'));
    assert!(!DelimiterType::SectionBoundary.matches('{'));

    assert!(DelimiterType::OverrideBoundary.matches('{'));
    assert!(DelimiterType::OverrideBoundary.matches('}'));
    assert!(!DelimiterType::OverrideBoundary.matches('['));
}

#[test]
fn line_and_comment_delimiter_matching() {
    assert!(DelimiterType::CommentMarker.matches(';'));
    assert!(!DelimiterType::CommentMarker.matches('#'));

    assert!(DelimiterType::LineTerminator.matches('\n'));
    assert!(DelimiterType::LineTerminator.matches('\r'));
    assert!(!DelimiterType::LineTerminator.matches('\t'));
}

#[test]
fn special_delimiter_matching() {
    assert!(DelimiterType::DrawingSeparator.matches(' '));
    assert!(DelimiterType::DrawingSeparator.matches('\t'));
    assert!(!DelimiterType::DrawingSeparator.matches('\n'));

    assert!(DelimiterType::TimeSeparator.matches(':'));
    assert!(DelimiterType::TimeSeparator.matches('.'));
    assert!(!DelimiterType::TimeSeparator.matches(','));

    assert!(DelimiterType::ColorSeparator.matches('&'));
    assert!(DelimiterType::ColorSeparator.matches('H'));
    assert!(!DelimiterType::ColorSeparator.matches('#'));
}
