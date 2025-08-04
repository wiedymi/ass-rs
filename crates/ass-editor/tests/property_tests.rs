//! Property-based tests for ass-editor
//!
//! Uses proptest to verify invariants and properties of the editor
//! across a wide range of inputs.

use ass_editor::{
    commands::*,
    core::{EditorDocument, Position, Range, StyleBuilder},
};
use proptest::prelude::*;

// Removed unused arb_position and arb_range functions
// These would be useful for future property tests but are not currently used

/// Generate arbitrary text content
fn arb_text() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple ASCII text (excluding '[' to avoid malformed sections)
        "[a-zA-Z0-9 ]{0,50}",
        // Text with ASS tags
        "\\{\\\\[a-z0-9]+\\}[a-zA-Z ]{0,30}",
        // Unicode text (excluding '[' and ']')
        "[\u{0020}-\u{005A}\u{005C}-\u{007E}\u{00A0}-\u{00FF}]{0,40}",
        // Empty string
        Just("".to_string()),
    ]
}

/// Find the nearest valid UTF-8 character boundary at or before the given offset
fn find_char_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }

    let mut pos = offset;
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// Generate a valid ASS script
fn arb_ass_script() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(arb_style(), 1..5),
        prop::collection::vec(arb_event(), 0..20),
    )
        .prop_map(|(styles, events)| {
            let mut script = String::from(
                "[Script Info]\nTitle: Test\nScriptType: v4.00+\n\n[V4+ Styles]\n"
            );
            script.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");

            for style in styles {
                script.push_str(&style);
                script.push('\n');
            }

            script.push_str("\n[Events]\n");
            script.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

            for event in events {
                script.push_str(&event);
                script.push('\n');
            }

            script
        })
}

/// Generate arbitrary style lines
fn arb_style() -> impl Strategy<Value = String> {
    (
        "[A-Za-z][A-Za-z0-9]{0,15}",  // name
        prop::bool::ANY,               // bold
        prop::bool::ANY,               // italic
        10..50u32,                     // size
    )
        .prop_map(|(name, bold, italic, size)| {
            format!(
                "Style: {name},Arial,{size},&H00FFFFFF,&H000000FF,&H00000000,&H00000000,{},{},0,0,100,100,0,0,1,2,0,2,10,10,10,1",
                bold as i32,
                italic as i32
            )
        })
}

/// Generate arbitrary event lines
fn arb_event() -> impl Strategy<Value = String> {
    (
        0..3u32,    // layer
        0..300u32,  // start seconds
        1..10u32,   // duration
        arb_text(), // text content
    )
        .prop_map(|(layer, start, duration, text)| {
            let end = start + duration;
            format!(
                "Dialogue: {layer},0:{:02}:{:02}.00,0:{:02}:{:02}.00,Default,,0,0,0,,{text}",
                start / 60,
                start % 60,
                end / 60,
                end % 60
            )
        })
}

proptest! {
    /// Test that insert followed by undo restores original state
    #[test]
    fn test_insert_undo_invariant(
        script in arb_ass_script(),
        text in arb_text(),
    ) {
        let mut doc = EditorDocument::from_content(&script)?;
        let original_text = doc.text();
        let max_pos = doc.len();

        if max_pos > 0 && !text.is_empty() {
            // Ensure we're inserting at a valid UTF-8 boundary
            let insert_offset = find_char_boundary(&original_text, max_pos / 2);
            let pos = Position::new(insert_offset);

            // Insert text
            doc.insert(pos, &text)?;
            prop_assert_ne!(doc.text(), original_text.clone());

            // Undo should restore original
            doc.undo()?;
            prop_assert_eq!(doc.text(), original_text);

            // Redo should reapply
            doc.redo()?;
            prop_assert!(doc.text().contains(&text));
        }
    }

    /// Test that replace operations maintain document validity
    #[test]
    fn test_replace_maintains_validity(
        script in arb_ass_script(),
        new_text in arb_text(),
    ) {
        let mut doc = EditorDocument::from_content(&script)?;
        let len = doc.len();

        if len > 10 {
            let doc_text = doc.text();
            let start = find_char_boundary(&doc_text, len / 4);
            let end = find_char_boundary(&doc_text, len / 2);
            let range = Range::new(Position::new(start), Position::new(end));

            // Replace should succeed
            doc.replace(range, &new_text)?;

            // Document should still be valid
            doc.validate()?;

            // Length should be adjusted correctly
            let expected_len = len - (end - start) + new_text.len();
            prop_assert_eq!(doc.len(), expected_len);
        }
    }

    /// Test that commands are reversible
    #[test]
    fn test_command_reversibility(
        script in arb_ass_script(),
        style_name in "[A-Za-z][A-Za-z0-9]{0,15}",
        font_size in 10.0..50.0f32,
    ) {
        let mut doc = EditorDocument::from_content(&script)?;
        let original_text = doc.text();

        // Create and execute a style command
        let style_builder = StyleBuilder::default()
            .font("Arial")
            .size(font_size as u32);
        let command = CreateStyleCommand::new(style_name, style_builder);

        command.execute(&mut doc)?;
        prop_assert_ne!(doc.text(), original_text.clone());

        // Undo should restore
        doc.undo()?;
        prop_assert_eq!(doc.text(), original_text);
    }

    /// Test that batch commands execute all sub-commands
    /// Note: Batch command undo is not currently implemented in the command system
    #[test]
    fn test_batch_command_execution(
        script in arb_ass_script(),
        texts in prop::collection::vec(arb_text(), 2..5),
    ) {
        let mut doc = EditorDocument::from_content(&script)?;

        // Filter out empty texts
        let non_empty_texts: Vec<_> = texts.iter()
            .filter(|t| !t.is_empty())
            .cloned()
            .collect();

        if non_empty_texts.is_empty() {
            // Skip test if all texts are empty - nothing to test
            return Ok(());
        }

        // Create batch with multiple insertions (only non-empty texts)
        let mut batch = BatchCommand::new("Test batch".to_string());
        let doc_text = doc.text();
        for (i, text) in non_empty_texts.iter().enumerate() {
            let offset = (i * 100).min(doc.len());
            let valid_offset = find_char_boundary(&doc_text, offset);
            let pos = Position::new(valid_offset);
            batch = batch.add_command(Box::new(
                InsertTextCommand::new(pos, text.clone())
            ));
        }

        // Execute batch
        let result = batch.execute(&mut doc)?;
        prop_assert!(result.success);
        prop_assert!(result.content_changed);

        // All non-empty texts should be present
        for text in &non_empty_texts {
            prop_assert!(doc.text().contains(text));
        }
    }

    /// Test that document positions remain valid after edits
    #[test]
    fn test_position_validity_after_edits(
        script in arb_ass_script(),
        edits in prop::collection::vec((arb_text(), 0..100usize), 1..10),
    ) {
        let mut doc = EditorDocument::from_content(&script)?;

        for (text, offset_percent) in edits {
            let max_offset = doc.len();
            let raw_offset = (max_offset * offset_percent) / 100;
            let doc_text = doc.text();
            let offset = find_char_boundary(&doc_text, raw_offset.min(max_offset));
            let pos = Position::new(offset);

            // Insert should succeed
            doc.insert(pos, &text)?;

            // Document length should increase
            prop_assert_eq!(doc.len(), max_offset + text.len());

            // All positions should be valid
            prop_assert!(Position::new(0).offset <= doc.len());
            prop_assert!(Position::new(doc.len()).offset <= doc.len());
        }
    }

    /// Test that search operations handle edge cases
    #[test]
    fn test_search_edge_cases(
        script in arb_ass_script(),
        pattern in "[a-zA-Z]{1,10}",
        case_sensitive in prop::bool::ANY,
        whole_words in prop::bool::ANY,
    ) {
        use ass_editor::utils::search::{DocumentSearch, DocumentSearchImpl, SearchOptions, SearchScope};

        let doc = EditorDocument::from_content(&script)?;
        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc)?;

        let options = SearchOptions {
            case_sensitive,
            whole_words,
            use_regex: false,
            scope: SearchScope::All,
            max_results: 100,
        };

        // Search should not panic
        let results = search.search(&pattern, &options)?;

        // Results should be within document bounds
        for result in results {
            prop_assert!(result.start.offset <= doc.len());
            prop_assert!(result.end.offset <= doc.len());
            prop_assert!(result.start.offset <= result.end.offset);
        }
    }

    /// Test style command validation
    #[test]
    fn test_style_command_validation(
        script in arb_ass_script(),
        new_font_name in "[A-Za-z][A-Za-z0-9]{0,30}",
        font_size in 1.0..200.0f32,
        bold in prop::bool::ANY,
        italic in prop::bool::ANY,
    ) {
        let mut doc = EditorDocument::from_content(&script)?;

        // Get the first style name from the script
        let first_style_name = doc.parse_script_with(|script| {
            script.sections()
                .iter()
                .find_map(|section| match section {
                    ass_core::parser::Section::Styles(styles) => {
                        styles.first().map(|style| style.name.to_string())
                    },
                    _ => None,
                })
        })?;

        if let Some(style_name) = first_style_name {
            // Edit existing style
            let command = EditStyleCommand::new(style_name)
                .set_font(&new_font_name)
                .set_size(font_size as u32)
                .set_bold(bold)
                .set_italic(italic);

            let result = command.execute(&mut doc)?;
            prop_assert!(result.success);

            // Document should remain valid
            doc.validate()?;
        }
    }

    /// Test that undo/redo stack has proper limits
    #[test]
    fn test_undo_stack_limits(
        script in arb_ass_script(),
        num_ops in 50..200usize,
    ) {
        use ass_editor::core::UndoStackConfig;

        let mut doc = EditorDocument::from_content(&script)?;

        // Set a small limit
        let config = UndoStackConfig {
            max_entries: 10,
            ..Default::default()
        };
        doc.undo_manager_mut().set_config(config);

        // Perform many operations
        for i in 0..num_ops {
            let doc_text = doc.text();
            let raw_offset = i % doc.len();
            let offset = find_char_boundary(&doc_text, raw_offset);
            let pos = Position::new(offset);
            doc.insert(pos, "X")?;
        }

        // Should only be able to undo up to the limit
        let mut undo_count = 0;
        while doc.can_undo() && undo_count < 20 {
            doc.undo()?;
            undo_count += 1;
        }

        prop_assert!(undo_count <= 10);
    }

    /// Test incremental parsing maintains consistency
    #[cfg(feature = "stream")]
    #[test]
    fn test_incremental_consistency(
        script in arb_ass_script(),
        edits in prop::collection::vec((arb_text(), 0..100usize), 1..5),
    ) {
        let mut doc_incremental = EditorDocument::from_content(&script)?;
        let mut doc_regular = EditorDocument::from_content(&script)?;

        for (text, offset_percent) in edits {
            let max_offset = doc_incremental.len();
            let raw_offset = (max_offset * offset_percent) / 100;
            let doc_text = doc_incremental.text();
            let offset = find_char_boundary(&doc_text, raw_offset.min(max_offset));
            let pos = Position::new(offset);
            let range = Range::new(pos, pos);

            // Apply edit both ways
            doc_incremental.edit_incremental(range, &text)?;
            doc_regular.replace(range, &text)?;

            // Results should match
            prop_assert_eq!(doc_incremental.text(), doc_regular.text());
        }
    }
}
