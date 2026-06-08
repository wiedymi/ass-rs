//! Tests for [`EventType`] parsing and string conversion.

use super::*;

#[test]
fn event_type_parsing() {
    assert_eq!(EventType::parse_type("Dialogue"), Some(EventType::Dialogue));
    assert_eq!(EventType::parse_type("Comment"), Some(EventType::Comment));
    assert_eq!(EventType::parse_type("Picture"), Some(EventType::Picture));
    assert_eq!(EventType::parse_type("Sound"), Some(EventType::Sound));
    assert_eq!(EventType::parse_type("Movie"), Some(EventType::Movie));
    assert_eq!(EventType::parse_type("Command"), Some(EventType::Command));
    assert_eq!(EventType::parse_type("Unknown"), None);
    assert_eq!(
        EventType::parse_type("  Dialogue  "),
        Some(EventType::Dialogue)
    );
}

#[test]
fn event_type_string_conversion() {
    assert_eq!(EventType::Dialogue.as_str(), "Dialogue");
    assert_eq!(EventType::Comment.as_str(), "Comment");
    assert_eq!(EventType::Picture.as_str(), "Picture");
    assert_eq!(EventType::Sound.as_str(), "Sound");
    assert_eq!(EventType::Movie.as_str(), "Movie");
    assert_eq!(EventType::Command.as_str(), "Command");
}

#[test]
fn event_type_properties() {
    assert_eq!(EventType::Dialogue, EventType::Dialogue);
    assert_ne!(EventType::Dialogue, EventType::Comment);
}

#[test]
fn event_type_parse_edge_cases() {
    // Test case sensitivity
    assert_eq!(EventType::parse_type("dialogue"), None);
    assert_eq!(EventType::parse_type("DIALOGUE"), None);

    // Test empty and whitespace
    assert_eq!(EventType::parse_type(""), None);
    assert_eq!(EventType::parse_type("   "), None);

    // Test with extra whitespace
    assert_eq!(
        EventType::parse_type("  Comment  "),
        Some(EventType::Comment)
    );
    assert_eq!(
        EventType::parse_type("\tPicture\n"),
        Some(EventType::Picture)
    );
}
