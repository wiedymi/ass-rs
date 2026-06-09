//! Integration tests for the Fonts and Graphics management APIs.

use ass_editor::EditorDocument;

#[test]
fn test_fonts_management_integration() {
    let mut doc = EditorDocument::new();

    // Start with empty document
    assert_eq!(doc.fonts().count().unwrap(), 0);
    assert!(doc.fonts().list().unwrap().is_empty());

    // Add fonts using UU-encoded data
    let font_data1 = vec![
        "begin 644 arial.ttf".to_string(),
        "M1234567890ABCDEF".to_string(),
        "end".to_string(),
    ];

    let font_data2 = vec![
        "begin 644 times.ttf".to_string(),
        "MFEDCBA0987654321".to_string(),
        "end".to_string(),
    ];

    doc.fonts().add("arial.ttf", font_data1.clone()).unwrap();
    doc.fonts().add("times.ttf", font_data2).unwrap();

    // Verify fonts were added
    assert_eq!(doc.fonts().count().unwrap(), 2);
    let fonts_list = doc.fonts().list().unwrap();
    assert_eq!(fonts_list.len(), 2);
    assert!(fonts_list.contains(&"arial.ttf".to_string()));
    assert!(fonts_list.contains(&"times.ttf".to_string()));

    // Test exists check
    assert!(doc.fonts().exists("arial.ttf").unwrap());
    assert!(doc.fonts().exists("times.ttf").unwrap());
    assert!(!doc.fonts().exists("comic.ttf").unwrap());

    // Add font from binary data
    let binary_data = b"This is fake font binary data for testing!";
    doc.fonts().add_binary("custom.otf", binary_data).unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 3);
    assert!(doc.fonts().exists("custom.otf").unwrap());

    // Verify the document structure
    assert!(doc.text().contains("[Fonts]"));
    assert!(doc.text().contains("fontname: arial.ttf"));
    assert!(doc.text().contains("fontname: times.ttf"));
    assert!(doc.text().contains("fontname: custom.otf"));
    assert!(doc.text().contains("begin 644"));
    assert!(doc.text().contains("end"));

    // Remove a specific font
    doc.fonts().remove("times.ttf").unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 2);
    assert!(!doc.fonts().exists("times.ttf").unwrap());
    assert!(doc.fonts().exists("arial.ttf").unwrap());
    assert!(doc.fonts().exists("custom.otf").unwrap());

    // Clear all fonts
    doc.fonts().clear().unwrap();
    assert_eq!(doc.fonts().count().unwrap(), 0);
    assert!(doc.fonts().list().unwrap().is_empty());

    // Section should still exist but be empty
    assert!(doc.text().contains("[Fonts]"));
}

#[test]
fn test_graphics_management_integration() {
    let mut doc = EditorDocument::new();

    // Start with empty document
    assert_eq!(doc.graphics().count().unwrap(), 0);
    assert!(doc.graphics().list().unwrap().is_empty());

    // Add graphics using UU-encoded data
    let graphic_data1 = vec![
        "begin 644 logo.png".to_string(),
        "M89PNG0D0A1A0A0000".to_string(),
        "end".to_string(),
    ];

    let graphic_data2 = vec![
        "begin 644 banner.jpg".to_string(),
        "MFFD8FFE000104A464946".to_string(),
        "end".to_string(),
    ];

    doc.graphics().add("logo.png", graphic_data1).unwrap();
    doc.graphics().add("banner.jpg", graphic_data2).unwrap();

    // Verify graphics were added
    assert_eq!(doc.graphics().count().unwrap(), 2);
    let graphics_list = doc.graphics().list().unwrap();
    assert_eq!(graphics_list.len(), 2);
    assert!(graphics_list.contains(&"logo.png".to_string()));
    assert!(graphics_list.contains(&"banner.jpg".to_string()));

    // Test exists check
    assert!(doc.graphics().exists("logo.png").unwrap());
    assert!(doc.graphics().exists("banner.jpg").unwrap());
    assert!(!doc.graphics().exists("icon.gif").unwrap());

    // Add graphic from binary data
    let binary_data = b"GIF89a fake image data for testing!";
    doc.graphics().add_binary("icon.gif", binary_data).unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 3);
    assert!(doc.graphics().exists("icon.gif").unwrap());

    // Verify the document structure
    assert!(doc.text().contains("[Graphics]"));
    assert!(doc.text().contains("filename: logo.png"));
    assert!(doc.text().contains("filename: banner.jpg"));
    assert!(doc.text().contains("filename: icon.gif"));

    // Remove a specific graphic
    doc.graphics().remove("banner.jpg").unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 2);
    assert!(!doc.graphics().exists("banner.jpg").unwrap());

    // Clear all graphics
    doc.graphics().clear().unwrap();
    assert_eq!(doc.graphics().count().unwrap(), 0);
    assert!(doc.graphics().list().unwrap().is_empty());
}
