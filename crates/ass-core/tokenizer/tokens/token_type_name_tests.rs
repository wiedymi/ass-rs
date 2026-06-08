//! Name and [`Display`](core::fmt::Display) tests for the [`TokenType`] enum.

use super::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn token_type_names() {
    assert_eq!(TokenType::Text.name(), "text");
    assert_eq!(TokenType::Number.name(), "number");
    assert_eq!(TokenType::HexValue.name(), "hex value");
    assert_eq!(TokenType::Invalid.name(), "invalid token");
}

#[test]
fn basic_token_type_names() {
    assert_eq!(TokenType::Text.name(), "text");
    assert_eq!(TokenType::Number.name(), "number");
    assert_eq!(TokenType::HexValue.name(), "hex value");
    assert_eq!(TokenType::Colon.name(), "colon");
    assert_eq!(TokenType::Comma.name(), "comma");
    assert_eq!(TokenType::Newline.name(), "newline");
    assert_eq!(TokenType::Invalid.name(), "invalid token");
    assert_eq!(TokenType::Eof.name(), "end of file");
}

#[test]
fn section_token_type_names() {
    assert_eq!(TokenType::SectionOpen.name(), "section open");
    assert_eq!(TokenType::SectionClose.name(), "section close");
    assert_eq!(TokenType::SectionName.name(), "section name");
    assert_eq!(TokenType::SectionHeader.name(), "section header");
}

#[test]
fn override_token_type_names() {
    assert_eq!(TokenType::OverrideOpen.name(), "override open");
    assert_eq!(TokenType::OverrideClose.name(), "override close");
    assert_eq!(TokenType::OverrideBlock.name(), "override block");
}

#[test]
fn special_token_type_names() {
    assert_eq!(TokenType::Comment.name(), "comment");
    assert_eq!(TokenType::Whitespace.name(), "whitespace");
    assert_eq!(TokenType::DrawingScale.name(), "drawing scale");
    assert_eq!(TokenType::UuEncodedLine.name(), "UU-encoded line");
    assert_eq!(TokenType::FontFilename.name(), "font filename");
    assert_eq!(TokenType::GraphicFilename.name(), "graphic filename");
    assert_eq!(TokenType::FormatLine.name(), "format line");
    assert_eq!(TokenType::EventType.name(), "event type");
    assert_eq!(TokenType::TimeValue.name(), "time value");
    assert_eq!(TokenType::BooleanValue.name(), "boolean value");
    assert_eq!(TokenType::PercentageValue.name(), "percentage value");
    assert_eq!(TokenType::StringLiteral.name(), "string literal");
}

#[test]
fn token_type_display() {
    assert_eq!(format!("{}", TokenType::Text), "text");
    assert_eq!(format!("{}", TokenType::Number), "number");
    assert_eq!(format!("{}", TokenType::Invalid), "invalid token");
    assert_eq!(format!("{}", TokenType::Eof), "end of file");
}
