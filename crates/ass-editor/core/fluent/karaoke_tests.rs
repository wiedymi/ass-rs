//! Tests for karaoke generation, splitting, and adjustment fluent operations.

use crate::commands::KaraokeType;
use crate::core::{EditorDocument, Position, Range};

#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn karaoke_generate() {
    let mut doc = EditorDocument::from_content("Hello World Test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Test basic karaoke generation with manual syllables to preserve text
    doc.karaoke()
        .in_range(range)
        .generate(50)
        .manual_syllables()
        .execute()
        .unwrap();

    let text = doc.text();
    assert!(text.contains("\\k50"));
    // With manual syllables, the entire text should be preserved
    assert!(text.contains("Hello World Test"));
}

#[test]
fn karaoke_generate_with_types() {
    let mut doc = EditorDocument::from_content("Test Text").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Test with fill karaoke
    doc.karaoke()
        .in_range(range)
        .generate(40)
        .karaoke_type(KaraokeType::Fill)
        .execute()
        .unwrap();

    assert!(doc.text().contains("\\kf40"));

    // Test with outline karaoke
    let mut doc2 = EditorDocument::from_content("Test Text").unwrap();
    let range2 = Range::new(Position::new(0), Position::new(doc2.text().len()));

    doc2.karaoke()
        .in_range(range2)
        .generate(30)
        .karaoke_type(KaraokeType::Outline)
        .execute()
        .unwrap();

    assert!(doc2.text().contains("\\ko30"));
}

#[test]
fn karaoke_generate_manual_syllables() {
    let mut doc = EditorDocument::from_content("Syllable Test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Test with manual syllable detection disabled
    doc.karaoke()
        .in_range(range)
        .generate(60)
        .manual_syllables()
        .execute()
        .unwrap();

    let text = doc.text();
    assert!(text.contains("\\k60"));
    assert!(text.contains("Syllable Test"));
}

#[test]
fn karaoke_split() {
    let mut doc = EditorDocument::from_content("{\\k100}Hello World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Split at position 5 (between "Hello" and " World")
    doc.karaoke()
        .in_range(range)
        .split(vec![5])
        .duration(25)
        .execute()
        .unwrap();

    let text = doc.text();
    assert!(text.contains("\\k25"));
}

#[test]
fn karaoke_adjust_scale() {
    let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Scale timing by 2.0
    doc.karaoke().in_range(range).adjust().scale(2.0).unwrap();

    let text = doc.text();
    assert!(text.contains("\\k100")); // 50 * 2.0
    assert!(text.contains("\\k60")); // 30 * 2.0
}

#[test]
fn karaoke_adjust_offset() {
    let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Add 20 centiseconds to all timings
    doc.karaoke().in_range(range).adjust().offset(20).unwrap();

    let text = doc.text();
    assert!(text.contains("\\k70")); // 50 + 20
    assert!(text.contains("\\k50")); // 30 + 20
}

#[test]
fn karaoke_adjust_set_all() {
    let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Set all timings to 45 centiseconds
    doc.karaoke().in_range(range).adjust().set_all(45).unwrap();

    let text = doc.text();
    assert!(text.contains("\\k45"));
    // Should contain exactly two instances of \\k45
    assert_eq!(text.matches("\\k45").count(), 2);
}

#[test]
fn karaoke_adjust_custom() {
    let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Apply custom timings: 80cs for first, 40cs for second
    doc.karaoke()
        .in_range(range)
        .adjust()
        .custom(vec![80, 40])
        .unwrap();

    let text = doc.text();
    assert!(text.contains("\\k80"));
    assert!(text.contains("\\k40"));
}
