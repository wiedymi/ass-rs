//! Tests for script info, fonts, and graphics fluent operations.

use crate::core::EditorDocument;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

#[test]
fn test_script_info_operations() {
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Original Title
ScriptType: v4.00+

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test getting existing property
    let title = doc.info().get_title().unwrap();
    assert_eq!(title, Some("Original Title".to_string()));

    // Test setting property
    doc.info().title("New Title").unwrap();
    let new_title = doc.info().get_title().unwrap();
    assert_eq!(new_title, Some("New Title".to_string()));

    // Test adding new property
    doc.info().author("John Doe").unwrap();
    let author = doc.info().get_author().unwrap();
    assert_eq!(author, Some("John Doe".to_string()));

    // Test resolution
    doc.info().resolution(1920, 1080).unwrap();
    let res = doc.info().get_resolution().unwrap();
    assert_eq!(res, Some((1920, 1080)));

    // Test getting all properties
    let all = doc.info().all().unwrap();
    assert!(all.contains(&("Title".to_string(), "New Title".to_string())));
    assert!(all.contains(&("Original Script".to_string(), "John Doe".to_string())));
    assert!(all.contains(&("PlayResX".to_string(), "1920".to_string())));
    assert!(all.contains(&("PlayResY".to_string(), "1080".to_string())));

    // Test deleting property
    doc.info().delete("ScriptType").unwrap();
    let script_type = doc.info().get("ScriptType").unwrap();
    assert_eq!(script_type, None);
}

#[test]
fn test_script_info_special_properties() {
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test wrap style
    doc.info().wrap_style(2).unwrap();
    let wrap = doc.info().get_wrap_style().unwrap();
    assert_eq!(wrap, Some(2));

    // Test scaled border and shadow
    doc.info().scaled_border_and_shadow(true).unwrap();
    let scaled = doc.info().get_scaled_border_and_shadow().unwrap();
    assert_eq!(scaled, Some(true));

    // Test with "no" value
    doc.info().scaled_border_and_shadow(false).unwrap();
    let not_scaled = doc.info().get_scaled_border_and_shadow().unwrap();
    assert_eq!(not_scaled, Some(false));
}

#[test]
fn test_fonts_operations() {
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test adding a font
    let font_data = vec![
        "begin 644 custom.ttf".to_string(),
        "M1234567890".to_string(),
        "end".to_string(),
    ];
    doc.fonts().add("custom.ttf", font_data.clone()).unwrap();

    // Verify font was added
    let fonts = doc.fonts().list().unwrap();
    assert_eq!(fonts.len(), 1);
    assert_eq!(fonts[0], "custom.ttf");
    assert!(doc.fonts().exists("custom.ttf").unwrap());

    // Test adding another font
    doc.fonts().add("another.otf", font_data).unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 2);

    // Test removing a font
    doc.fonts().remove("custom.ttf").unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 1);
    assert!(!doc.fonts().exists("custom.ttf").unwrap());
    assert!(doc.fonts().exists("another.otf").unwrap());

    // Test clearing all fonts
    doc.fonts().clear().unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 0);
    assert!(doc.fonts().list().unwrap().is_empty());
}

#[test]
fn test_fonts_binary_add() {
    let mut doc = EditorDocument::new();

    // Test adding font from binary data
    let binary_data = b"Hello Font Data!";
    doc.fonts().add_binary("test.ttf", binary_data).unwrap();

    // Verify font was added
    assert!(doc.fonts().exists("test.ttf").unwrap());
    assert!(doc.text().contains("[Fonts]"));
    assert!(doc.text().contains("fontname: test.ttf"));
    assert!(doc.text().contains("begin 644 test.ttf"));
    assert!(doc.text().contains("end"));
}

#[test]
fn test_graphics_operations() {
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test adding a graphic
    let graphic_data = vec![
        "begin 644 logo.png".to_string(),
        "M89PNG1234".to_string(),
        "end".to_string(),
    ];
    doc.graphics()
        .add("logo.png", graphic_data.clone())
        .unwrap();

    // Verify graphic was added
    let graphics = doc.graphics().list().unwrap();
    assert_eq!(graphics.len(), 1);
    assert_eq!(graphics[0], "logo.png");
    assert!(doc.graphics().exists("logo.png").unwrap());

    // Test adding another graphic
    doc.graphics().add("banner.jpg", graphic_data).unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 2);

    // Test removing a graphic
    doc.graphics().remove("logo.png").unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 1);
    assert!(!doc.graphics().exists("logo.png").unwrap());
    assert!(doc.graphics().exists("banner.jpg").unwrap());

    // Test clearing all graphics
    doc.graphics().clear().unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 0);
    assert!(doc.graphics().list().unwrap().is_empty());
}

#[test]
fn test_graphics_binary_add() {
    let mut doc = EditorDocument::new();

    // Test adding graphic from binary data
    let binary_data = b"PNG Image Data Here!";
    doc.graphics().add_binary("image.png", binary_data).unwrap();

    // Verify graphic was added
    assert!(doc.graphics().exists("image.png").unwrap());
    assert!(doc.text().contains("[Graphics]"));
    assert!(doc.text().contains("filename: image.png"));
    assert!(doc.text().contains("begin 644 image.png"));
    assert!(doc.text().contains("end"));
}
