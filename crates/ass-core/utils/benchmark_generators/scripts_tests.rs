//! Tests for the realistic-preset generators and the standalone script
//! builder functions (`create_test_event`, issue and overlap generators).

use super::*;
use crate::parser::ast::EventType;

#[test]
fn create_test_event_basic() {
    let event = create_test_event("0:00:00.00", "0:00:05.00", "Test text");
    assert_eq!(event.start, "0:00:00.00");
    assert_eq!(event.end, "0:00:05.00");
    assert_eq!(event.text, "Test text");
    assert_eq!(event.style, "Default");
    assert!(matches!(event.event_type, EventType::Dialogue));
}

#[test]
fn generate_script_with_issues_basic() {
    let script = generate_script_with_issues(5);
    assert!(script.contains("[Script Info]"));
    assert!(script.contains("[V4+ Styles]"));
    assert!(script.contains("[Events]"));
    assert!(script.contains("Dialogue:"));
}

#[test]
fn generate_script_with_issues_contains_problems() {
    let script = generate_script_with_issues(20);
    // Should contain some problematic content
    assert!(script.lines().count() > 10);
    // At least one event should have issues (every 10th event)
    assert!(script.contains("empty tag") || script.contains("unknown tag"));
}

#[test]
fn generate_overlapping_script_basic() {
    let script = generate_overlapping_script(3);
    assert!(script.contains("[V4+ Styles]"));
    assert!(script.contains("[Events]"));
    assert!(script.contains("Event 0 text"));
    assert!(script.contains("Event 1 text"));
    assert!(script.contains("Event 2 text"));
}

#[test]
fn generate_overlapping_script_timing() {
    let script = generate_overlapping_script(2);
    // First event: 0:00:00.00 to 0:00:05.00
    // Second event: 0:00:02.00 to 0:00:07.00 (overlaps with first)
    assert!(script.contains("0:00:00.00"));
    assert!(script.contains("0:00:05.00"));
    assert!(script.contains("0:00:02.00"));
    assert!(script.contains("0:00:07.00"));
}

#[test]
fn anime_realistic_generator() {
    let generator = ScriptGenerator::anime_realistic(5);
    assert_eq!(generator.events_count, 5);
    assert_eq!(generator.styles_count, 15);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::AnimeRealistic
    ));

    let script = generator.generate();
    assert!(script.contains("Anime Subtitles"));
}

#[test]
fn movie_realistic_generator() {
    let generator = ScriptGenerator::movie_realistic(3);
    assert_eq!(generator.events_count, 3);
    assert_eq!(generator.styles_count, 3);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::MovieRealistic
    ));
}

#[test]
fn karaoke_realistic_generator() {
    let generator = ScriptGenerator::karaoke_realistic(2);
    assert_eq!(generator.events_count, 2);
    assert_eq!(generator.styles_count, 8);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::KaraokeRealistic
    ));

    let text = generator.generate_dialogue_text(0);
    assert!(text.contains(r"{\k"));
}

#[test]
fn sign_realistic_generator() {
    let generator = ScriptGenerator::sign_realistic(4);
    assert_eq!(generator.events_count, 4);
    assert_eq!(generator.styles_count, 12);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::SignRealistic
    ));

    let text = generator.generate_dialogue_text(0);
    assert!(text.contains(r"{\pos(") || text.contains(r"{\an"));
}

#[test]
fn educational_realistic_generator() {
    let generator = ScriptGenerator::educational_realistic(6);
    assert_eq!(generator.events_count, 6);
    assert_eq!(generator.styles_count, 6);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::EducationalRealistic
    ));

    let text = generator.generate_dialogue_text(1);
    assert!(text.contains("Question") || text.contains("Answer") || text.contains("Definition"));
}
