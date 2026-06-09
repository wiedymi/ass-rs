//! ASS dialogue and timing edge cases for ass-editor.
//!
//! Special dialogue text, karaoke timing tags, and time format variations.

use ass_editor::EditorDocument;

#[test]
fn test_dialogue_text_special_cases() {
    let test_texts = vec![
        r"Plain text",
        r"Text with\Nhard break",
        r"Text with\nsoft break",
        r"{\an8}Top aligned",
        r"{\pos(100,200)}Positioned",
        r"{\1c&HFF0000&}Blue text",
        r"Multiple{\i1}tags{\i0}here",
        r"Nested{\b1\i1}tags{\b0\i0}",
        r"Escaped \{ brace",
        r"Unicode: 你好 мир 🌍",
        r"Empty tags{}text",
        r"{\}Malformed tag",
        r"{No closing brace",
        r"Line1\NLine2\NLine3\N\N\NMany breaks",
        r"Special chars: <>&",
        r"Tab	character",
        r"", // Empty text
    ];

    let mut content = String::from("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for (i, text) in test_texts.iter().enumerate() {
        content.push_str(&format!(
            "Dialogue: 0,0:{i:02}:00.00,0:{i:02}:01.00,Default,,0,0,0,,{text}\n"
        ));
    }

    let doc = EditorDocument::from_content(&content).unwrap();

    // All special text cases should be preserved
    for text in test_texts {
        if !text.is_empty() {
            assert!(doc.text().contains(text));
        }
    }
}

#[test]
fn test_karaoke_timing_tags() {
    let content = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\k100}Simple{\k50}Karaoke
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,{\K100}Karaoke{\kf200}Fill{\ko300}Outline
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,Pre{\k50}Mid{\k0}Zero{\k-50}Negative
"#;

    let doc = EditorDocument::from_content(content).unwrap();

    // Karaoke tags should be preserved
    assert!(doc.text().contains(r"{\k100}"));
    assert!(doc.text().contains(r"{\K100}"));
    assert!(doc.text().contains(r"{\kf200}"));
    assert!(doc.text().contains(r"{\ko300}"));
    assert!(doc.text().contains(r"{\k0}"));
    assert!(doc.text().contains(r"{\k-50}"));
}

#[test]
fn test_time_format_variations() {
    let times = vec![
        "0:00:00.00",   // Standard
        "00:00:00.00",  // Leading zero hour
        "0:0:0.0",      // Minimal digits
        "0:00:00.000",  // Three decimal places
        "9:59:59.99",   // Max single digit hour
        "10:00:00.00",  // Double digit hour
        "100:00:00.00", // Triple digit hour
        "0:60:00.00",   // Invalid minutes
        "0:00:60.00",   // Invalid seconds
        "0:00:00.100",  // Three decimal places
        "0:00:00",      // No decimals
        "-0:00:01.00",  // Negative time
    ];

    let mut content = String::from("[Events]\nFormat: Start, End, Text\n");

    for (i, time) in times.iter().enumerate() {
        content.push_str(&format!(
            "Dialogue: {time},0:{:02}:00.00,Time test {i}\n",
            i + 1
        ));
    }

    let result = EditorDocument::from_content(&content);
    if let Ok(doc) = result {
        // Parser should handle or preserve all time formats
        for time in times {
            assert!(doc.text().contains(time));
        }
    }
}
