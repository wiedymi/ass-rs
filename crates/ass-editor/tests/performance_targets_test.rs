//! Tests to verify performance targets for incremental parsing
//!
//! Ensures we meet:
//! - <1ms for single edit operations
//! - <5ms for full reparse operations

#![cfg(all(test, feature = "stream"))]

use ass_editor::core::{EditorDocument, Position, Range};
use std::time::Instant;

/// Generate a medium-sized realistic ASS script
fn generate_test_script() -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Performance Test Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,4,0,5,10,10,10,1
Style: Sign,Arial,30,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,3,0,8,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    // Add 200 dialogue events (realistic episode size)
    for i in 0..200 {
        let start_time = format!("0:{:02}:{:02}.{:02}", i / 60, i % 60, (i * 17) % 100);
        let end_time = format!(
            "0:{:02}:{:02}.{:02}",
            (i + 3) / 60,
            (i + 3) % 60,
            ((i + 3) * 17) % 100
        );
        let style = if i % 10 == 0 {
            "Title"
        } else if i % 7 == 0 {
            "Sign"
        } else {
            "Default"
        };

        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},{style},,0,0,0,,Line {i}: {{\\pos(960,540)}}Some dialogue with {{\\b1}}bold{{\\b0}} and {{\\i1}}italic{{\\i0}} text\n"
        ));
    }

    script
}

#[test]
fn test_single_edit_under_1ms() {
    let script = generate_test_script();
    let mut doc = EditorDocument::from_content(&script).unwrap();

    // Warm up the incremental parser
    let warm_up_pos = Position::new(script.len() / 3);
    let _ = doc
        .edit_incremental(Range::new(warm_up_pos, warm_up_pos), "X")
        .unwrap();

    // Test single character insertion
    let pos = Position::new(script.len() / 2);
    let range = Range::new(pos, pos);

    let start = Instant::now();
    let _ = doc.edit_incremental(range, "A").unwrap();
    let elapsed = start.elapsed();

    println!("Single character edit took: {elapsed:?}");
    assert!(
        elapsed.as_micros() < 1000,
        "Single edit took {}µs, expected <1000µs (1ms)",
        elapsed.as_micros()
    );
}

#[test]
fn test_word_replacement_under_1ms() {
    let script = generate_test_script();
    let mut doc = EditorDocument::from_content(&script).unwrap();

    // Find a word to replace
    let word_pos = doc.text().find("dialogue").unwrap();
    let range = Range::new(Position::new(word_pos), Position::new(word_pos + 8));

    let start = Instant::now();
    let _ = doc.edit_incremental(range, "subtitle").unwrap();
    let elapsed = start.elapsed();

    println!("Word replacement took: {elapsed:?}");
    assert!(
        elapsed.as_micros() < 1000,
        "Word replacement took {}µs, expected <1000µs (1ms)",
        elapsed.as_micros()
    );
}

#[test]
fn test_line_insertion_under_1ms() {
    let script = generate_test_script();
    let mut doc = EditorDocument::from_content(&script).unwrap();

    // Insert a new dialogue line
    let insert_pos = Position::new(script.len() - 10);
    let range = Range::new(insert_pos, insert_pos);
    let new_line = "\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,New line inserted";

    let start = Instant::now();
    let _ = doc.edit_incremental(range, new_line).unwrap();
    let elapsed = start.elapsed();

    println!("Line insertion took: {elapsed:?}");
    assert!(
        elapsed.as_micros() < 1000,
        "Line insertion took {}µs, expected <1000µs (1ms)",
        elapsed.as_micros()
    );
}

#[test]
fn test_full_reparse_under_5ms() {
    let script = generate_test_script();
    let mut doc = EditorDocument::from_content(&script).unwrap();

    // Force a full reparse by making many small edits
    for i in 0..50 {
        let pos = Position::new((i * 100) % (script.len() - 1));
        let _ = doc.edit_incremental(Range::new(pos, pos), "x").unwrap();
    }

    // This edit should trigger a full reparse
    let pos = Position::new(script.len() / 2);
    let range = Range::new(pos, pos);

    let start = Instant::now();
    let _ = doc.edit_incremental(range, "FULL_REPARSE").unwrap();
    let elapsed = start.elapsed();

    println!("Full reparse took: {elapsed:?}");
    assert!(
        elapsed.as_micros() < 5000,
        "Full reparse took {}µs, expected <5000µs (5ms)",
        elapsed.as_micros()
    );
}

#[test]
fn test_multiple_rapid_edits() {
    let script = generate_test_script();
    let mut doc = EditorDocument::from_content(&script).unwrap();

    // Simulate rapid typing (10 characters)
    let start_pos = script.len() / 2;
    let start_time = Instant::now();

    for i in 0..10 {
        let pos = Position::new(start_pos + i);
        let range = Range::new(pos, pos);
        let _ = doc.edit_incremental(range, &i.to_string()).unwrap();
    }

    let total_elapsed = start_time.elapsed();
    let avg_per_edit = total_elapsed.as_micros() / 10;

    println!("10 rapid edits took: {total_elapsed:?} total, {avg_per_edit:?} average per edit");
    assert!(
        avg_per_edit < 1500,
        "Average edit time {avg_per_edit}µs, expected <1500µs (1.5ms) for rapid consecutive edits"
    );
}

#[test]
fn test_safe_edit_fallback_performance() {
    let script = generate_test_script();
    let mut doc = EditorDocument::from_content(&script).unwrap();

    // Test safe edit which includes fallback logic
    let pos = Position::new(script.len() / 2);
    let range = Range::new(pos, pos);

    let start = Instant::now();
    doc.edit_safe(range, "SAFE").unwrap();
    let elapsed = start.elapsed();

    println!("Safe edit with fallback took: {elapsed:?}");
    assert!(
        elapsed.as_micros() < 5000,
        "Safe edit took {}µs, expected <5000µs (5ms) including fallback logic",
        elapsed.as_micros()
    );
}
