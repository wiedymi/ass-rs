//! Tests for fonts and graphics management commands

use super::*;
use crate::commands::EditorCommand;
use crate::core::EditorDocument;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

#[test]
fn test_add_font() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = AddFontCommand::new(
        "custom.ttf".to_string(),
        vec![
            "begin 644 custom.ttf".to_string(),
            "M1234...".to_string(),
            "end".to_string(),
        ],
    );
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("[Fonts]"));
    assert!(doc.text().contains("fontname: custom.ttf"));
}

#[test]
fn test_remove_font() {
    let mut doc = EditorDocument::from_content(
        "[Script Info]\n[Fonts]\nfontname: test.ttf\nbegin 644 test.ttf\nM123\nend\n",
    )
    .unwrap();

    let command = RemoveFontCommand::new("test.ttf".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(!doc.text().contains("fontname: test.ttf"));
}

#[test]
fn test_list_fonts() {
    let doc = EditorDocument::from_content(
        "[Fonts]\nfontname: font1.ttf\ndata\nfontname: font2.otf\ndata\n",
    )
    .unwrap();

    let command = ListFontsCommand::new();
    let fonts = command.list(&doc).unwrap();

    assert_eq!(fonts.len(), 2);
    assert_eq!(fonts[0], "font1.ttf");
    assert_eq!(fonts[1], "font2.otf");
}

#[test]
fn test_uuencode() {
    let data = b"Hello World!";
    let encoded = uuencode_data("test.txt", data);

    assert_eq!(encoded[0], "begin 644 test.txt");
    assert_eq!(encoded[encoded.len() - 1], "end");
    assert_eq!(encoded[encoded.len() - 2], "`");
}
