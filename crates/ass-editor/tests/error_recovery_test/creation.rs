//! Document creation error tests.
//!
//! Tests for invalid inputs and edge cases encountered while creating
//! documents.

use ass_editor::{EditorDocument, Position};

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
