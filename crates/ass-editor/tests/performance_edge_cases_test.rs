//! Performance and stress edge case tests for ass-editor
//!
//! These tests focus on performance boundaries and stress conditions

use ass_editor::*;
use std::time::Instant;

// ===== Performance Boundary Tests =====

#[test]
fn test_rapid_consecutive_operations() {
    let mut doc = EditorDocument::new();
    let start = Instant::now();

    // Perform 10000 rapid operations
    for i in 0..10000 {
        if i % 2 == 0 {
            doc.insert(Position::new(0), "x").unwrap();
        } else {
            doc.delete(Range::new(Position::new(0), Position::new(1)))
                .unwrap();
        }
    }

    let elapsed = start.elapsed();
    println!("10000 operations took: {elapsed:?}");

    // Document should be empty after alternating insert/delete
    assert!(doc.is_empty());
}

#[test]
fn test_nested_undo_redo_performance() {
    let mut doc = EditorDocument::new();

    // Build a complex history
    for i in 0..100 {
        doc.insert(Position::new(doc.len_bytes()), &format!("Line {i}\n"))
            .unwrap();
    }

    let start = Instant::now();

    // Undo all (up to the limit)
    let mut undo_count = 0;
    while doc.undo().is_ok() {
        undo_count += 1;
    }

    // Redo all that we undid
    let mut redo_count = 0;
    while doc.redo().is_ok() {
        redo_count += 1;
    }

    let elapsed = start.elapsed();
    println!("{undo_count} undo + {redo_count} redo operations took: {elapsed:?}");

    // Document should contain some lines (depending on undo limit)
    assert!(doc.text().contains("Line"));
}

#[test]
fn test_pathological_line_lengths() {
    // Test with extremely long lines
    let long_line = "a".repeat(100000);
    let mut doc = EditorDocument::from_content(&long_line).unwrap();

    // Operations on long lines
    doc.insert(Position::new(50000), "MIDDLE").unwrap();
    assert!(doc.text().contains("MIDDLE"));

    // Delete from middle of long line
    doc.delete(Range::new(Position::new(50000), Position::new(50006)))
        .unwrap();
    assert!(!doc.text().contains("MIDDLE"));
}

#[test]
fn test_many_small_lines() {
    // Document with many small lines
    let mut content = String::new();
    for i in 0..10000 {
        content.push_str(&format!("{i}\n"));
    }

    let mut doc = EditorDocument::from_content(&content).unwrap();
    assert_eq!(doc.len_lines(), 10001); // 10000 lines + final empty line

    // Insert at approximate middle position
    let mid_pos = Position::new(content.len() / 2);
    doc.insert(mid_pos, "INSERTED").unwrap();

    // Verify insertion
    assert!(doc.text().contains("INSERTED"));
}

#[test]
fn test_fragmented_editing_pattern() {
    let mut doc = EditorDocument::from_content("0123456789".repeat(100).as_str()).unwrap();

    // Simulate fragmented editing (many small non-contiguous edits)
    for i in (0..1000).step_by(10) {
        let pos = Position::new(i);
        doc.replace(Range::new(pos, pos.advance(1)), "X").unwrap();
    }

    // Count X's in result
    let x_count = doc.text().matches('X').count();
    assert_eq!(x_count, 100);
}

// ===== Memory Stress Tests =====

#[test]
fn test_memory_pressure_document_growth() {
    let mut doc = EditorDocument::new();

    // Grow document incrementally
    for i in 0..1000 {
        let chunk = format!("Chunk {i}: {}\n", "data".repeat(100));
        doc.insert(Position::new(doc.len_bytes()), &chunk).unwrap();

        // Every 100 chunks, do some undo/redo to stress history
        if i % 100 == 99 {
            let mut undo_count = 0;
            for _ in 0..10 {
                if doc.undo().is_ok() {
                    undo_count += 1;
                } else {
                    break;
                }
            }
            for _ in 0..undo_count {
                if doc.redo().is_err() {
                    break;
                }
            }
        }
    }

    let final_size = doc.len_bytes();
    eprintln!("Final document size: {final_size} bytes");
    // With undo/redo, document might not reach full size due to history limits
    assert!(final_size > 100000); // At least 100KB
}

#[test]
fn test_replace_performance_patterns() {
    let mut doc = EditorDocument::from_content(&"test ".repeat(10000)).unwrap();

    let start = Instant::now();

    // Replace all occurrences (simulating find-replace-all)
    let mut offset = 0;
    while let Some(pos) = doc.text()[offset..].find("test") {
        let abs_pos = offset + pos;
        doc.replace(
            Range::new(Position::new(abs_pos), Position::new(abs_pos + 4)),
            "replaced",
        )
        .unwrap();
        offset = abs_pos + 8; // length of "replaced"
    }

    let elapsed = start.elapsed();
    println!("Replace-all operation took: {elapsed:?}");

    assert!(!doc.text().contains("test"));
    assert_eq!(doc.text().matches("replaced").count(), 10000);
}

// ===== ASS-Specific Performance Tests =====

#[test]
fn test_large_ass_script_handling() {
    let mut content = String::from("[Script Info]\nTitle: Large Script\n\n[Events]\n");
    content.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    // Add 10000 dialogue lines
    for i in 0..10000 {
        let hours = i / 3600;
        let minutes = (i % 3600) / 60;
        let seconds = i % 60;
        content.push_str(&format!(
            "Dialogue: 0,{hours}:{minutes:02}:{seconds:02}.00,{hours}:{minutes:02}:{:02}.00,Default,,0,0,0,,Line {i} text\n",
            seconds + 1
        ));
    }

    let start = Instant::now();
    let doc = EditorDocument::from_content(&content).unwrap();
    let parse_time = start.elapsed();
    println!("Parsing 10000 dialogue lines took: {parse_time:?}");

    assert!(doc.text().contains("Line 9999 text"));
}

#[test]
fn test_complex_ass_formatting_tags() {
    let complex_line = r"Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\pos(192,108)\fad(200,200)\1c&H00FF00&\3c&H000000&\bord2\shad1\be1\fs24\fn Arial\i1\b1\u1\s1\frz30\frx15\fry20\fscx120\fscy80\fsp3\q2\a6\k100}Complex{\k50}Text{\r}Normal";

    let mut doc =
        EditorDocument::from_content(&format!("[Script Info]\n\n[Events]\n{complex_line}"))
            .unwrap();

    // Ensure complex tags are preserved
    assert!(doc.text().contains(r"{\pos(192,108)"));
    assert!(doc.text().contains(r"{\k50}"));

    // Edit within the complex tags
    let tag_pos = doc.text().find(r"\fs24").unwrap();
    doc.replace(
        Range::new(Position::new(tag_pos + 3), Position::new(tag_pos + 5)),
        "48",
    )
    .unwrap();

    assert!(doc.text().contains(r"\fs48"));
}

// ===== Edge Cases for Position Calculations =====

#[test]
fn test_position_calculation_edge_cases() {
    // Test with mixed character widths
    let content = "ASCII 中文 🎭 Émojis 👨‍👩‍👧‍👦\nLine2\r\nLine3";
    let doc = EditorDocument::from_content(content).unwrap();

    // Test position at each character boundary
    let positions = vec![
        (0, 1, 1),   // Start
        (5, 1, 6),   // After "ASCII"
        (6, 1, 7),   // After space
        (9, 1, 8),   // After first Chinese char
        (12, 1, 9),  // After second Chinese char
        (13, 1, 10), // After space
        (17, 1, 11), // After theater emoji
    ];

    for (byte_offset, expected_line, expected_col) in positions {
        let lc = doc
            .position_to_line_column(Position::new(byte_offset))
            .unwrap();
        assert_eq!(lc.line, expected_line);
        assert_eq!(lc.column, expected_col);
    }
}

#[test]
fn test_boundary_crossing_operations() {
    let mut doc = EditorDocument::from_content("Line1\nLine2\nLine3").unwrap();

    // Delete across line boundary
    // "Line1\nLine2\nLine3" positions:
    // L=0 i=1 n=2 e=3 1=4 \n=5 L=6 i=7 n=8 e=9 2=10 \n=11 L=12 i=13 n=14 e=15 3=16
    let start = Position::new(5); // Start of newline after Line1
    let end = Position::new(11); // Start of newline after Line2

    doc.delete(Range::new(start, end)).unwrap();
    assert_eq!(doc.text(), "Line1\nLine3");

    // Insert to restore
    doc.insert(Position::new(6), "Line2\n").unwrap();
    assert_eq!(doc.text(), "Line1\nLine2\nLine3");
}

#[test]
fn test_zero_width_operations() {
    let mut doc = EditorDocument::from_content("Test").unwrap();

    // Zero-width delete (should succeed but do nothing)
    let before = doc.text();
    doc.delete(Range::new(Position::new(2), Position::new(2)))
        .unwrap();
    assert_eq!(doc.text(), before);

    // Zero-width replace (acts as insert)
    doc.replace(Range::new(Position::new(2), Position::new(2)), "X")
        .unwrap();
    assert_eq!(doc.text(), "TeXst");
}

// ===== Concurrent Modification Patterns =====

// Concurrent position query test removed - EditorDocument is not thread-safe

// ===== Extension System Performance =====

#[cfg(feature = "plugins")]
#[test]
fn test_extension_manager_under_load() {
    let mut manager = ExtensionManager::new();

    // Load multiple extensions
    manager
        .load_extension(Box::new(
            ass_editor::extensions::builtin::syntax_highlight::SyntaxHighlightExtension::new(),
        ))
        .unwrap();

    manager
        .load_extension(Box::new(
            ass_editor::extensions::builtin::auto_complete::AutoCompleteExtension::new(),
        ))
        .unwrap();

    // Set many config values
    for i in 0..1000 {
        manager.set_config(format!("key_{i}"), format!("value_{i}"));
    }

    // Query config values
    let start = Instant::now();
    for i in 0..1000 {
        assert_eq!(
            manager.get_config(&format!("key_{i}")).as_deref(),
            Some(&format!("value_{i}")[..])
        );
    }
    let elapsed = start.elapsed();
    println!("1000 config lookups took: {elapsed:?}");
}

// ===== Error Recovery Under Stress =====

#[test]
fn test_error_recovery_stress() {
    let mut doc = EditorDocument::from_content("Initial content").unwrap();
    let mut success_count = 0;
    let mut error_count = 0;

    // Mix valid and invalid operations
    for i in 0..1000 {
        let result = match i % 4 {
            0 => doc.insert(Position::new(i % doc.len_bytes()), "x"),
            1 => doc.delete(Range::new(Position::new(0), Position::new(1))),
            2 => doc.insert(Position::new(999999), "invalid"), // Should fail
            _ => doc.delete(Range::new(Position::new(999999), Position::new(1000000))), // Should fail
        };

        match result {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    // Should have ~500 successes and ~500 errors
    assert!(success_count > 400 && success_count < 600);
    assert!(error_count > 400 && error_count < 600);

    // Document should still be valid
    assert!(!doc.text().is_empty());
}
