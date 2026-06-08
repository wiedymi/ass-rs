//! Tests for karaoke template application and chaining fluent operations.

use crate::commands::KaraokeType;
use crate::core::{EditorDocument, Position, Range};

#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn karaoke_apply_equal() {
    let mut doc = EditorDocument::from_content("Hello World Test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Apply equal timing of 35cs with fill karaoke
    doc.karaoke()
        .in_range(range)
        .apply()
        .equal(35, KaraokeType::Fill)
        .unwrap();

    let text = doc.text();
    assert!(text.contains("\\kf35"));
    assert!(text.contains("Hello"));
    assert!(text.contains("World"));
    assert!(text.contains("Test"));
}

#[test]
fn karaoke_apply_beat() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Apply beat-based timing: 120 BPM, 0.5 beats per syllable
    // Expected duration: (60/120) * 0.5 * 100 = 25 centiseconds
    doc.karaoke()
        .in_range(range)
        .apply()
        .beat(120, 0.5, KaraokeType::Standard)
        .unwrap();

    let text = doc.text();
    assert!(text.contains("\\k25"));
}

#[test]
fn karaoke_apply_pattern() {
    let mut doc = EditorDocument::from_content("Hello World Test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Apply pattern-based timing: 40cs, 60cs, repeating
    doc.karaoke()
        .in_range(range)
        .apply()
        .pattern(vec![40, 60], KaraokeType::Outline)
        .unwrap();

    let text = doc.text();
    assert!(text.contains("\\ko40"));
    assert!(text.contains("\\ko60"));
}

#[test]
fn karaoke_apply_import() {
    let mut doc = EditorDocument::from_content("Source text for import").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Apply import timing (simplified test - would import from event 0)
    doc.karaoke()
        .in_range(range)
        .apply()
        .import_from(0)
        .unwrap();

    // Since import is simplified and returns original text, verify no crash
    assert!(doc.text().contains("Source text for import"));
}

#[test]
fn karaoke_chaining_operations() {
    let mut doc = EditorDocument::from_content("Chain test").unwrap();

    // Test that karaoke operations can be chained with other fluent operations
    doc.at_pos(Position::new(0))
        .insert_text("Prefix: ")
        .unwrap();

    assert_eq!(doc.text(), "Prefix: Chain test");

    // Now apply karaoke to the appended part with manual syllables
    let karaoke_range = Range::new(Position::new(8), Position::new(doc.text().len()));
    doc.karaoke()
        .in_range(karaoke_range)
        .generate(45)
        .manual_syllables()
        .execute()
        .unwrap();

    let text = doc.text();
    assert!(text.starts_with("Prefix: "));
    assert!(text.contains("\\k45"));
    // With manual syllables, the original appended text is preserved
    assert!(text.contains("Chain test"));
}
