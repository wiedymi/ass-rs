//! Classification predicate tests for the [`TokenType`] enum.

use super::*;

#[test]
fn token_type_checks() {
    assert!(TokenType::Comma.is_delimiter());
    assert!(TokenType::SectionHeader.is_structural());
    assert!(TokenType::Text.is_content());
    assert!(TokenType::Whitespace.is_skippable());
    assert!(TokenType::Comment.is_skippable());
}

#[test]
fn delimiter_token_types_are_delimiters() {
    assert!(TokenType::Colon.is_delimiter());
    assert!(TokenType::Comma.is_delimiter());
    assert!(TokenType::SectionOpen.is_delimiter());
    assert!(TokenType::SectionClose.is_delimiter());
    assert!(TokenType::OverrideOpen.is_delimiter());
    assert!(TokenType::OverrideClose.is_delimiter());
}

#[test]
fn non_delimiter_token_types_are_not_delimiters() {
    assert!(!TokenType::Text.is_delimiter());
    assert!(!TokenType::Number.is_delimiter());
    assert!(!TokenType::HexValue.is_delimiter());
    assert!(!TokenType::SectionName.is_delimiter());
    assert!(!TokenType::SectionHeader.is_delimiter());
    assert!(!TokenType::OverrideBlock.is_delimiter());
    assert!(!TokenType::Comment.is_delimiter());
    assert!(!TokenType::Whitespace.is_delimiter());
    assert!(!TokenType::Newline.is_delimiter());
    assert!(!TokenType::DrawingScale.is_delimiter());
    assert!(!TokenType::UuEncodedLine.is_delimiter());
    assert!(!TokenType::FontFilename.is_delimiter());
    assert!(!TokenType::GraphicFilename.is_delimiter());
    assert!(!TokenType::FormatLine.is_delimiter());
    assert!(!TokenType::EventType.is_delimiter());
    assert!(!TokenType::TimeValue.is_delimiter());
    assert!(!TokenType::BooleanValue.is_delimiter());
    assert!(!TokenType::PercentageValue.is_delimiter());
    assert!(!TokenType::StringLiteral.is_delimiter());
    assert!(!TokenType::Invalid.is_delimiter());
    assert!(!TokenType::Eof.is_delimiter());
}

#[test]
fn structural_token_types_are_structural() {
    assert!(TokenType::SectionHeader.is_structural());
    assert!(TokenType::SectionOpen.is_structural());
    assert!(TokenType::SectionClose.is_structural());
    assert!(TokenType::FormatLine.is_structural());
    assert!(TokenType::Newline.is_structural());
}

#[test]
fn non_structural_token_types_are_not_structural() {
    assert!(!TokenType::Text.is_structural());
    assert!(!TokenType::Number.is_structural());
    assert!(!TokenType::HexValue.is_structural());
    assert!(!TokenType::Colon.is_structural());
    assert!(!TokenType::Comma.is_structural());
    assert!(!TokenType::SectionName.is_structural());
    assert!(!TokenType::OverrideOpen.is_structural());
    assert!(!TokenType::OverrideClose.is_structural());
    assert!(!TokenType::OverrideBlock.is_structural());
    assert!(!TokenType::Comment.is_structural());
    assert!(!TokenType::Whitespace.is_structural());
    assert!(!TokenType::DrawingScale.is_structural());
    assert!(!TokenType::UuEncodedLine.is_structural());
    assert!(!TokenType::FontFilename.is_structural());
    assert!(!TokenType::GraphicFilename.is_structural());
    assert!(!TokenType::EventType.is_structural());
    assert!(!TokenType::TimeValue.is_structural());
    assert!(!TokenType::BooleanValue.is_structural());
    assert!(!TokenType::PercentageValue.is_structural());
    assert!(!TokenType::StringLiteral.is_structural());
    assert!(!TokenType::Invalid.is_structural());
    assert!(!TokenType::Eof.is_structural());
}

#[test]
fn content_token_types_are_content() {
    assert!(TokenType::Text.is_content());
    assert!(TokenType::Number.is_content());
    assert!(TokenType::HexValue.is_content());
    assert!(TokenType::TimeValue.is_content());
    assert!(TokenType::BooleanValue.is_content());
    assert!(TokenType::PercentageValue.is_content());
    assert!(TokenType::StringLiteral.is_content());
}

#[test]
fn non_content_token_types_are_not_content() {
    assert!(!TokenType::Colon.is_content());
    assert!(!TokenType::Comma.is_content());
    assert!(!TokenType::Newline.is_content());
    assert!(!TokenType::SectionOpen.is_content());
    assert!(!TokenType::SectionClose.is_content());
    assert!(!TokenType::SectionName.is_content());
    assert!(!TokenType::SectionHeader.is_content());
    assert!(!TokenType::OverrideOpen.is_content());
    assert!(!TokenType::OverrideClose.is_content());
    assert!(!TokenType::OverrideBlock.is_content());
    assert!(!TokenType::Comment.is_content());
    assert!(!TokenType::Whitespace.is_content());
    assert!(!TokenType::DrawingScale.is_content());
    assert!(!TokenType::UuEncodedLine.is_content());
    assert!(!TokenType::FontFilename.is_content());
    assert!(!TokenType::GraphicFilename.is_content());
    assert!(!TokenType::FormatLine.is_content());
    assert!(!TokenType::EventType.is_content());
    assert!(!TokenType::Invalid.is_content());
    assert!(!TokenType::Eof.is_content());
}

#[test]
fn skippable_token_types_are_skippable() {
    assert!(TokenType::Whitespace.is_skippable());
    assert!(TokenType::Comment.is_skippable());
}

#[test]
fn basic_non_skippable_token_types() {
    assert!(!TokenType::Text.is_skippable());
    assert!(!TokenType::Number.is_skippable());
    assert!(!TokenType::HexValue.is_skippable());
    assert!(!TokenType::Colon.is_skippable());
    assert!(!TokenType::Comma.is_skippable());
    assert!(!TokenType::Newline.is_skippable());
}

#[test]
fn section_non_skippable_token_types() {
    assert!(!TokenType::SectionOpen.is_skippable());
    assert!(!TokenType::SectionClose.is_skippable());
    assert!(!TokenType::SectionName.is_skippable());
    assert!(!TokenType::SectionHeader.is_skippable());
}

#[test]
fn override_non_skippable_token_types() {
    assert!(!TokenType::OverrideOpen.is_skippable());
    assert!(!TokenType::OverrideClose.is_skippable());
    assert!(!TokenType::OverrideBlock.is_skippable());
}

#[test]
fn special_non_skippable_token_types() {
    assert!(!TokenType::DrawingScale.is_skippable());
    assert!(!TokenType::UuEncodedLine.is_skippable());
    assert!(!TokenType::FontFilename.is_skippable());
    assert!(!TokenType::GraphicFilename.is_skippable());
    assert!(!TokenType::FormatLine.is_skippable());
    assert!(!TokenType::EventType.is_skippable());
    assert!(!TokenType::TimeValue.is_skippable());
    assert!(!TokenType::BooleanValue.is_skippable());
    assert!(!TokenType::PercentageValue.is_skippable());
    assert!(!TokenType::StringLiteral.is_skippable());
    assert!(!TokenType::Invalid.is_skippable());
    assert!(!TokenType::Eof.is_skippable());
}
