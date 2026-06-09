//! ASS format parsing edge cases for ass-editor.
//!
//! Section validation, event line handling, malformed headers, and comments.

use ass_editor::EditorDocument;

#[test]
fn test_ass_format_validation() {
    // Valid ASS content
    let valid_cases = vec![
        "[Script Info]\nTitle: Test",
        "[Script Info]\n\n[Events]\n",
        "[Script Info]\n[V4+ Styles]\n[Events]\n",
    ];

    for content in valid_cases {
        assert!(EditorDocument::from_content(content).is_ok());
    }

    // These should still load (parser might be lenient)
    let edge_cases = vec![
        "",                             // Empty document
        "\n\n\n",                       // Only newlines
        "[Script Info]",                // No newline at end
        "Random text\n[Script Info]\n", // Text before section
    ];

    for content in edge_cases {
        let result = EditorDocument::from_content(content);
        // Document creation might succeed even if not valid ASS
        if let Ok(doc) = result {
            // Should be able to get text back
            assert_eq!(doc.text(), content);
        }
    }
}

#[test]
fn test_event_line_edge_cases() {
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,Simple text
Comment: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,This is a comment
Dialogue: 1,0:00:02.00,0:00:03.00,Default,,0,0,0,,{\an8}Text with tags
Dialogue: 0,0:00:03.00,0:00:04.00,Default,,0,0,0,,Line with\Nhard break
Dialogue: 0,0:00:04.00,0:00:05.00,Default,,0,0,0,,Special chars: <>&"'
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // Document should handle all these cases
    assert!(doc.text().contains("Simple text"));
    assert!(doc.text().contains("This is a comment"));
    assert!(doc.text().contains(r"{\an8}"));
    assert!(doc.text().contains(r"\N"));
    assert!(doc.text().contains("<>&\"'"));
}

#[test]
fn test_malformed_section_headers() {
    let test_cases = vec![
        "[Script Info",          // Missing closing bracket
        "Script Info]",          // Missing opening bracket
        "[ Script Info ]",       // Extra spaces
        "[SCRIPT INFO]",         // Different case
        "[Script  Info]",        // Double space
        "[]",                    // Empty section
        "[Script Info][Events]", // Multiple sections on one line
        "[\tScript Info\t]",     // Tabs instead of spaces
    ];

    for header in test_cases {
        let content = format!("{header}\nTitle: Test");
        let result = EditorDocument::from_content(&content);

        // Parser should either handle gracefully or fail predictably
        if let Ok(doc) = result {
            // If it parses, content should be preserved
            assert!(doc.text().contains(header));
        }
    }
}

#[test]
fn test_comment_variations() {
    let content = r#"[Script Info]
; Standard comment
;No space after semicolon
    ; Indented comment
Title: Test ; Inline comment
; Multi-line comment \
  continuation

[Events]
Comment: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,Commented dialogue
Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,Normal ; with semicolon
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // All comment variations should be preserved
    assert!(doc.text().contains("; Standard comment"));
    assert!(doc.text().contains(";No space"));
    assert!(doc.text().contains("    ; Indented"));
    assert!(doc.text().contains("Test ; Inline"));
    assert!(doc.text().contains("Comment: 0,"));
}
