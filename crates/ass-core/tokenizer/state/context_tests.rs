//! Unit tests for [`TokenContext`] and [`IssueLevel`] state types.

use super::*;

#[test]
fn token_context_transitions() {
    let mut context = TokenContext::Document;
    assert!(context.allows_whitespace_skipping());
    assert!(!context.is_delimited_block());

    context = context.enter_field_value();
    assert_eq!(context, TokenContext::FieldValue);
    assert!(!context.allows_whitespace_skipping());

    context = context.reset_to_document();
    assert_eq!(context, TokenContext::Document);
}

#[test]
fn token_context_delimiters() {
    assert_eq!(TokenContext::SectionHeader.closing_delimiter(), Some(']'));
    assert_eq!(TokenContext::StyleOverride.closing_delimiter(), Some('}'));
    assert_eq!(TokenContext::Document.closing_delimiter(), None);
}

#[test]
fn issue_level_properties() {
    assert!(!IssueLevel::Warning.is_error());
    assert!(IssueLevel::Error.is_error());
    assert!(IssueLevel::Critical.is_error());

    assert!(!IssueLevel::Warning.should_abort());
    assert!(!IssueLevel::Error.should_abort());
    assert!(IssueLevel::Critical.should_abort());
}

#[test]
fn token_context_all_variants() {
    // Test all TokenContext variants for is_delimited_block
    assert!(!TokenContext::Document.is_delimited_block());
    assert!(TokenContext::SectionHeader.is_delimited_block());
    assert!(!TokenContext::FieldValue.is_delimited_block());
    assert!(TokenContext::StyleOverride.is_delimited_block());
    assert!(!TokenContext::DrawingCommands.is_delimited_block());
    assert!(!TokenContext::UuEncodedData.is_delimited_block());
}

#[test]
fn token_context_whitespace_skipping_all_variants() {
    // Test all TokenContext variants for allows_whitespace_skipping
    assert!(TokenContext::Document.allows_whitespace_skipping());
    assert!(TokenContext::SectionHeader.allows_whitespace_skipping());
    assert!(!TokenContext::FieldValue.allows_whitespace_skipping());
    assert!(TokenContext::StyleOverride.allows_whitespace_skipping());
    assert!(TokenContext::DrawingCommands.allows_whitespace_skipping());
    assert!(!TokenContext::UuEncodedData.allows_whitespace_skipping());
}

#[test]
fn token_context_closing_delimiters_all_variants() {
    // Test all TokenContext variants for closing_delimiter
    assert_eq!(TokenContext::Document.closing_delimiter(), None);
    assert_eq!(TokenContext::SectionHeader.closing_delimiter(), Some(']'));
    assert_eq!(TokenContext::FieldValue.closing_delimiter(), None);
    assert_eq!(TokenContext::StyleOverride.closing_delimiter(), Some('}'));
    assert_eq!(TokenContext::DrawingCommands.closing_delimiter(), None);
    assert_eq!(TokenContext::UuEncodedData.closing_delimiter(), None);
}

#[test]
fn token_context_enter_field_value_all_variants() {
    // Test enter_field_value from all contexts
    assert_eq!(
        TokenContext::Document.enter_field_value(),
        TokenContext::FieldValue
    );
    assert_eq!(
        TokenContext::SectionHeader.enter_field_value(),
        TokenContext::SectionHeader
    );
    assert_eq!(
        TokenContext::FieldValue.enter_field_value(),
        TokenContext::FieldValue
    );
    assert_eq!(
        TokenContext::StyleOverride.enter_field_value(),
        TokenContext::StyleOverride
    );
    assert_eq!(
        TokenContext::DrawingCommands.enter_field_value(),
        TokenContext::DrawingCommands
    );
    assert_eq!(
        TokenContext::UuEncodedData.enter_field_value(),
        TokenContext::UuEncodedData
    );
}

#[test]
fn token_context_reset_to_document_all_variants() {
    // Test reset_to_document from all contexts
    assert_eq!(
        TokenContext::Document.reset_to_document(),
        TokenContext::Document
    );
    assert_eq!(
        TokenContext::SectionHeader.reset_to_document(),
        TokenContext::Document
    );
    assert_eq!(
        TokenContext::FieldValue.reset_to_document(),
        TokenContext::Document
    );
    assert_eq!(
        TokenContext::StyleOverride.reset_to_document(),
        TokenContext::Document
    );
    assert_eq!(
        TokenContext::DrawingCommands.reset_to_document(),
        TokenContext::Document
    );
    assert_eq!(
        TokenContext::UuEncodedData.reset_to_document(),
        TokenContext::Document
    );
}

#[test]
fn token_context_default() {
    assert_eq!(TokenContext::default(), TokenContext::Document);
}

#[test]
fn issue_level_as_str() {
    assert_eq!(IssueLevel::Warning.as_str(), "warning");
    assert_eq!(IssueLevel::Error.as_str(), "error");
    assert_eq!(IssueLevel::Critical.as_str(), "critical");
}

#[test]
fn issue_level_clone_and_copy() {
    let level1 = IssueLevel::Warning;
    let level2 = level1;
    let level3 = level1;

    assert_eq!(level1, level2);
    assert_eq!(level1, level3);
}

#[test]
fn token_context_clone_and_copy() {
    let context1 = TokenContext::StyleOverride;
    let context2 = context1;
    let context3 = context1;

    assert_eq!(context1, context2);
    assert_eq!(context1, context3);
}
