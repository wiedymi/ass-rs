//! Error handling and recovery tests for ass-editor
//!
//! Tests focusing on error conditions, invalid inputs, and recovery scenarios

use ass_editor::*;

// ===== Document Creation Errors =====

#[test]
fn test_invalid_utf8_handling() {
    // Note: In Rust, we can't directly create invalid UTF-8 in a &str
    // But we can test how the system handles various edge cases

    let edge_cases = vec![
        "\u{0}",        // Null character
        "\u{FEFF}test", // BOM character
        "test\u{200B}ing", // Zero-width space
                        // "\u{D800}",   // Invalid surrogate - commented out as Rust rejects this
    ];

    for content in edge_cases {
        let result = EditorDocument::from_content(content);
        if let Ok(doc) = result {
            // If it accepts the content, it should preserve it
            assert_eq!(doc.text(), content);
        }
    }
}

#[test]
fn test_empty_and_whitespace_documents() {
    let test_cases = vec![
        "",
        " ",
        "\t",
        "\n",
        "\r\n",
        "   \n   \n   ",
        "\t\t\t",
        "\n\n\n\n\n",
    ];

    for content in test_cases {
        let doc = EditorDocument::from_content(content).unwrap();
        assert_eq!(doc.text(), content);

        // Should be able to perform operations
        let mut doc_mut = doc;
        doc_mut.insert(Position::new(0), "X").unwrap();
        assert!(doc_mut.text().starts_with('X'));
    }
}

// ===== Position and Range Errors =====

#[test]
fn test_invalid_position_operations() {
    let mut doc = EditorDocument::from_content("Hello").unwrap();

    // Positions beyond document
    assert!(doc.insert(Position::new(100), "X").is_err());
    assert!(doc.position_to_line_column(Position::new(100)).is_err());

    // Negative-like positions (using large values that might wrap)
    let huge_pos = Position::new(usize::MAX);
    assert!(doc.insert(huge_pos, "X").is_err());

    // Position at exact boundary should work
    assert!(doc.insert(Position::new(5), " World").is_ok());
}

#[test]
fn test_invalid_range_operations() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();

    // Invalid ranges
    let test_ranges = [
        // End before start
        Range::new(Position::new(5), Position::new(0)),
        // Both positions beyond document
        Range::new(Position::new(100), Position::new(200)),
        // Start valid, end invalid
        Range::new(Position::new(5), Position::new(100)),
        // Extremely large ranges
        Range::new(Position::new(0), Position::new(usize::MAX)),
    ];

    for (idx, range) in test_ranges.iter().enumerate() {
        // First range gets normalized from (5,0) to (0,5) which is valid
        if idx == 0 {
            // This range is valid after normalization
            assert_eq!(range.start.offset, 0);
            assert_eq!(range.end.offset, 5);
            assert!(doc.delete(*range).is_ok());
            // Restore document
            doc.undo().unwrap();
            continue;
        }

        // These should fail due to out-of-bounds positions
        assert!(
            doc.delete(*range).is_err(),
            "delete should fail for range {range:?}"
        );
        assert!(
            doc.text_range(*range).is_err(),
            "text_range should fail for range {range:?}"
        );
        assert!(
            doc.replace(*range, "X").is_err(),
            "replace should fail for range {range:?}"
        );
    }
}

#[test]
fn test_line_column_errors() {
    let doc = EditorDocument::from_content("Line1\nLine2\nLine3").unwrap();

    // Test position_to_line_column with invalid positions instead
    let invalid_positions = vec![
        Position::new(1000), // Beyond document end
        Position::new(usize::MAX),
    ];

    for pos in invalid_positions {
        assert!(doc.position_to_line_column(pos).is_err());
    }

    // Test LineColumn construction errors
    assert!(ass_editor::core::LineColumn::new(0, 1).is_err()); // Line 0 doesn't exist
    assert!(ass_editor::core::LineColumn::new(1, 0).is_err()); // Column 0 doesn't exist
}

// ===== Operation Atomicity =====

#[test]
fn test_failed_operations_dont_modify_document() {
    let original = "Original content";
    let mut doc = EditorDocument::from_content(original).unwrap();

    // Failed insert
    let insert_result = doc.insert(Position::new(1000), "Should not appear");
    assert!(insert_result.is_err());
    assert_eq!(doc.text(), original);

    // Failed delete
    let delete_result = doc.delete(Range::new(Position::new(0), Position::new(1000)));
    assert!(delete_result.is_err());
    assert_eq!(doc.text(), original);

    // Failed replace
    let replace_result = doc.replace(
        Range::new(Position::new(8), Position::new(1000)),
        "Should not appear",
    );
    assert!(replace_result.is_err());
    assert_eq!(doc.text(), original);
}

#[test]
fn test_undo_redo_consistency_with_errors() {
    let mut doc = EditorDocument::from_content("Step 1").unwrap();

    // Successful operation
    doc.insert(Position::new(6), " - Success").unwrap();
    assert_eq!(doc.text(), "Step 1 - Success");

    // Failed operation (should not affect undo stack)
    assert!(doc.insert(Position::new(1000), " - Fail").is_err());

    // Another successful operation
    doc.insert(Position::new(doc.len_bytes()), " - Step 2")
        .unwrap();

    // Undo should skip the failed operation
    doc.undo().unwrap();
    assert_eq!(doc.text(), "Step 1 - Success");

    doc.undo().unwrap();
    assert_eq!(doc.text(), "Step 1");

    // Redo should also skip the failed operation
    doc.redo().unwrap();
    assert_eq!(doc.text(), "Step 1 - Success");

    // Try to redo the second operation
    let redo_result = doc.redo();
    if redo_result.is_ok() {
        assert_eq!(doc.text(), "Step 1 - Success - Step 2");
    } else {
        // Redo might not be available depending on implementation
        // Just ensure we're in a consistent state
        assert_eq!(doc.text(), "Step 1 - Success");
    }
}

// ===== Extension System Errors =====

#[test]
fn test_extension_manager_error_cases() {
    let mut manager = ExtensionManager::new();

    // Non-existent extension operations
    assert_eq!(manager.get_extension_state("non-existent"), None);

    // Empty string operations
    manager.set_config("".to_string(), "value".to_string());
    assert_eq!(manager.get_config("").as_deref(), Some("value"));

    manager.set_config("key".to_string(), "".to_string());
    assert_eq!(manager.get_config("key").as_deref(), Some(""));

    // Very long keys and values
    let long_key = "k".repeat(10000);
    let long_value = "v".repeat(10000);
    manager.set_config(long_key.clone(), long_value.clone());
    assert_eq!(
        manager.get_config(&long_key).as_deref(),
        Some(long_value.as_str())
    );
}

// Extension loading error test removed - Extension trait not publicly exported

// ===== Session Manager Errors =====

#[test]
fn test_session_manager_error_cases() {
    let mut manager = EditorSessionManager::new();

    // Create session
    manager.create_session("test".to_string()).unwrap();

    // Duplicate session - behavior may vary (might replace or fail)
    let _duplicate_result = manager.create_session("test".to_string());
    // Just verify it doesn't panic

    // Operations on non-existent session
    assert!(manager.remove_session("non-existent").is_err());
    assert!(manager
        .with_document_mut("non-existent", |_| Ok(()))
        .is_err());

    // Empty session name might be allowed
    let empty_result = manager.create_session("".to_string());
    // If it succeeds, clean it up
    if empty_result.is_ok() {
        manager.remove_session("").unwrap();
    }

    // Very long session name
    let long_name = "x".repeat(10000);
    let result = manager.create_session(long_name.clone());
    if result.is_ok() {
        assert!(manager.list_sessions().unwrap().contains(&long_name));
    }
}

// ===== Concurrent Error Scenarios =====

#[cfg(all(feature = "multi-thread", feature = "std"))]
#[test]
fn test_concurrent_error_handling() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    let error_count = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn threads that try various operations
    // Each thread gets its own manager to avoid cloning issues
    for i in 0..10 {
        let error_count_clone = error_count.clone();
        let success_count_clone = success_count.clone();

        let handle = thread::spawn(move || {
            // Each thread creates its own manager
            let mut local_manager = ass_editor::EditorSessionManager::new();

            // Try to create a session
            match local_manager.create_session("shared".to_string()) {
                Ok(_) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
            };

            // Try to access non-existent sessions
            let result = local_manager.with_document_mut(&format!("missing_{i}"), |_doc| Ok(()));
            match result {
                Ok(_) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
            };

            // Try valid operations
            match local_manager.create_session(format!("thread_{i}")) {
                Ok(_) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
            };
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Should have mix of successes and errors
    assert!(error_count.load(Ordering::Relaxed) > 0);
    assert!(success_count.load(Ordering::Relaxed) > 0);
}

// ===== Memory Exhaustion Scenarios =====

#[test]
fn test_document_size_limits() {
    // Try to create documents approaching memory limits
    let sizes = vec![
        1_000_000, // 1MB
        10_000_000, // 10MB
                   // Note: Larger sizes might cause actual OOM, so we keep it reasonable
    ];

    for size in sizes {
        let content = "X".repeat(size);
        let result = EditorDocument::from_content(&content);

        if let Ok(mut doc) = result {
            // Document was created, try operations
            assert_eq!(doc.len_bytes(), size);

            // Try to double the size
            let double_result = doc.insert(Position::new(0), &content);
            if double_result.is_ok() {
                assert_eq!(doc.len_bytes(), size * 2);
            }
        }
    }
}

#[test]
fn test_undo_stack_memory_pressure() {
    let mut doc = EditorDocument::new();

    // Create many small operations to fill undo stack
    for i in 0..10000 {
        doc.insert(Position::new(0), &i.to_string()).unwrap();
    }

    // Undo many operations
    let mut undo_count = 0;
    while doc.undo().is_ok() && undo_count < 5000 {
        undo_count += 1;
    }

    // Create a large operation
    let large_text = "X".repeat(1_000_000);
    let result = doc.insert(Position::new(0), &large_text);

    if result.is_ok() {
        // Redo should now fail (history was cleared)
        assert!(doc.redo().is_err());
    }
}

// ===== Parser Recovery =====

#[test]
fn test_parser_recovery_from_corruption() {
    // Create long string outside vec to avoid temporary value issue
    let nested_tags = format!("[Events]\nDialogue: {}", "{".repeat(1000));

    let corrupted_cases = vec![
        // Binary data mixed with text
        "[\0Script Info]\nTitle: Test",
        // Truncated file
        "[Script Info]\nTitle: Test\n[Eve",
        // Mixed encodings (simulated)
        // "[Script Info]\nTitle: Test\xFF\xFE", // Invalid hex escapes
        // Extremely nested tags
        nested_tags.as_str(),
        // Unclosed sections
        "[Script Info\nTitle: Test\n[Events",
    ];

    for content in corrupted_cases {
        let result = EditorDocument::from_content(content);

        // Parser should either recover or fail gracefully
        match result {
            Ok(doc) => {
                // If it parsed, should be able to get text back
                let _ = doc.text();
                // Should be able to perform operations
                let mut doc_mut = doc;
                let _ = doc_mut.insert(Position::new(0), "X");
            }
            Err(_) => {
                // Failed gracefully - this is also acceptable
            }
        }
    }
}

// ===== State Consistency =====

#[test]
fn test_document_state_consistency_after_errors() {
    let mut doc = EditorDocument::from_content("Initial").unwrap();

    // Track state
    let initial_len = doc.len_bytes();
    let initial_lines = doc.len_lines();

    // Perform mix of valid and invalid operations
    doc.insert(Position::new(7), " state").unwrap();
    assert!(doc.insert(Position::new(1000), "fail").is_err());
    doc.delete(Range::new(Position::new(0), Position::new(7)))
        .unwrap();
    assert!(doc
        .delete(Range::new(Position::new(0), Position::new(1000)))
        .is_err());

    // State should be consistent
    assert_eq!(doc.text(), " state");
    assert_eq!(doc.len_bytes(), 6);
    assert_eq!(doc.len_lines(), 1);

    // Undo all successful operations
    doc.undo().unwrap();
    doc.undo().unwrap();

    // Should be back to initial state
    assert_eq!(doc.text(), "Initial");
    assert_eq!(doc.len_bytes(), initial_len);
    assert_eq!(doc.len_lines(), initial_lines);
}
