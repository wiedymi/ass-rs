//! Tests for complex karaoke workflows, type coverage, and edge cases.

use crate::commands::KaraokeType;
use crate::core::{EditorDocument, Position, Range};

#[test]
fn karaoke_complex_workflow() {
    let mut doc = EditorDocument::from_content("Complex karaoke test with multiple words").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // 1. Generate initial karaoke with standard timing and manual syllables
    doc.karaoke()
        .in_range(range)
        .generate(50)
        .karaoke_type(KaraokeType::Standard)
        .manual_syllables()
        .execute()
        .unwrap();

    let mut text = doc.text();
    assert!(text.contains("\\k50"));

    // 2. Scale the timing by 1.5
    let current_range = Range::new(Position::new(0), Position::new(doc.text().len()));
    doc.karaoke()
        .in_range(current_range)
        .adjust()
        .scale(1.5)
        .unwrap();

    text = doc.text();
    assert!(text.contains("\\k75")); // 50 * 1.5

    // 3. Add 10cs offset
    let final_range = Range::new(Position::new(0), Position::new(doc.text().len()));
    doc.karaoke()
        .in_range(final_range)
        .adjust()
        .offset(10)
        .unwrap();

    text = doc.text();
    assert!(text.contains("\\k85")); // 75 + 10

    // With manual syllables, the entire original text is preserved
    assert!(text.contains("Complex karaoke test with multiple words"));
}

#[test]
fn karaoke_different_types_workflow() {
    // Test all karaoke types in sequence
    let test_text = "Test karaoke types";

    // Standard karaoke
    let mut doc1 = EditorDocument::from_content(test_text).unwrap();
    let range1 = Range::new(Position::new(0), Position::new(doc1.text().len()));
    doc1.karaoke()
        .in_range(range1)
        .generate(30)
        .karaoke_type(KaraokeType::Standard)
        .execute()
        .unwrap();
    assert!(doc1.text().contains("\\k30"));

    // Fill karaoke
    let mut doc2 = EditorDocument::from_content(test_text).unwrap();
    let range2 = Range::new(Position::new(0), Position::new(doc2.text().len()));
    doc2.karaoke()
        .in_range(range2)
        .generate(40)
        .karaoke_type(KaraokeType::Fill)
        .execute()
        .unwrap();
    assert!(doc2.text().contains("\\kf40"));

    // Outline karaoke
    let mut doc3 = EditorDocument::from_content(test_text).unwrap();
    let range3 = Range::new(Position::new(0), Position::new(doc3.text().len()));
    doc3.karaoke()
        .in_range(range3)
        .generate(50)
        .karaoke_type(KaraokeType::Outline)
        .execute()
        .unwrap();
    assert!(doc3.text().contains("\\ko50"));

    // Transition karaoke
    let mut doc4 = EditorDocument::from_content(test_text).unwrap();
    let range4 = Range::new(Position::new(0), Position::new(doc4.text().len()));
    doc4.karaoke()
        .in_range(range4)
        .generate(60)
        .karaoke_type(KaraokeType::Transition)
        .execute()
        .unwrap();
    assert!(doc4.text().contains("\\kt60"));
}

#[test]
fn karaoke_error_conditions() {
    // Test with text containing override blocks (should fail)
    let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\b0}").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    let result = doc.karaoke().in_range(range).generate(50).execute();

    // Should fail because text contains override blocks
    assert!(result.is_err());
}

#[test]
fn karaoke_edge_cases() {
    // Test with empty text
    let mut doc = EditorDocument::from_content("").unwrap();
    let range = Range::new(Position::new(0), Position::new(0));

    let result = doc.karaoke().in_range(range).generate(50).execute();

    // Should handle empty text gracefully
    assert!(result.is_ok());

    // Test with single character
    let mut doc2 = EditorDocument::from_content("A").unwrap();
    let range2 = Range::new(Position::new(0), Position::new(1));

    doc2.karaoke()
        .in_range(range2)
        .generate(25)
        .execute()
        .unwrap();

    assert!(doc2.text().contains("\\k25"));
    assert!(doc2.text().contains("A"));
}
