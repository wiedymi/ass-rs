//! Tests for the karaoke management commands.

use super::*;
use crate::commands::EditorCommand;
use crate::core::{EditorDocument, Position, Range};
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
#[test]
fn generate_karaoke_basic() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(0), Position::new(11));
    let command = GenerateKaraokeCommand::new(range, 50);

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(doc.text().contains("\\k50"));
}

#[test]
fn split_karaoke() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(0), Position::new(11));
    let command = SplitKaraokeCommand::new(range, vec![5]).duration(30);

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(doc.text().contains("\\k30"));
}

#[test]
fn adjust_karaoke_scale() {
    let mut doc = EditorDocument::from_content("{\\k50}Hello").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));
    let command = AdjustKaraokeCommand::scale(range, 2.0);

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(doc.text().contains("\\k100"));
}

#[test]
fn apply_karaoke_equal() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(0), Position::new(11));
    let command = ApplyKaraokeCommand::equal(range, 40, KaraokeType::Fill);

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(doc.text().contains("\\kf40"));
}

#[test]
fn apply_karaoke_beat() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(0), Position::new(11));
    let command = ApplyKaraokeCommand::beat(range, 120, 0.5, KaraokeType::Standard);

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    // Beat timing: (60/120) * 0.5 * 100 = 25 centiseconds
    assert!(doc.text().contains("\\k25"));
}

#[test]
fn karaoke_types() {
    assert_eq!(KaraokeType::Standard.tag_string(), "k");
    assert_eq!(KaraokeType::Fill.tag_string(), "kf");
    assert_eq!(KaraokeType::Outline.tag_string(), "ko");
    assert_eq!(KaraokeType::Transition.tag_string(), "kt");
}
