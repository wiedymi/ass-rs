use ass_core::Script;

#[test]
fn parse_serialize_roundtrip() {
    let src = b"[Script Info]\nTitle: RoundTrip\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!\n";
    let script = Script::parse(src);
    let out = script.serialize();
    // Ensure we can parse serialized output again and retain same number of sections
    let reparsed = Script::parse(out.as_bytes());
    assert_eq!(script.sections().len(), reparsed.sections().len());
}

#[test]
fn roundtrip_complex_script() {
    let src = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(src);
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());

    // Should maintain the same number of sections
    assert_eq!(script.sections().len(), reparsed.sections().len());

    // Should be able to serialize again
    let reserialized = reparsed.serialize();
    assert!(!reserialized.is_empty());
}

#[test]
fn roundtrip_script_info() {
    let src = b"[Script Info]\nTitle: Complex Title with Spaces\nOriginalScript: Author Name\nPlayResX: 1920\nPlayResY: 1080\nWrapStyle: 2\nScaledBorderAndShadow: yes\n";
    let script = Script::parse(src);
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(script.sections().len(), reparsed.sections().len());
}

#[test]
fn roundtrip_styles() {
    let src = b"[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,32,&H00FFFFFF,&H000000FF,&H00000000,&H64000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\nStyle: Bold,Arial,32,&H00FFFFFF,&H000000FF,&H00000000,&H64000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n";
    let script = Script::parse(src);
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(script.sections().len(), reparsed.sections().len());
}

#[test]
fn roundtrip_events_with_complex_text() {
    let src = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\\b1}Bold{\\b0} and {\\i1}italic{\\i0} text\nDialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,{\\c&HFF0000&}Red text{\\c} with colors\nComment: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,This is a comment line\n".as_bytes();
    let script = Script::parse(src);
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(script.sections().len(), reparsed.sections().len());
}

#[test]
fn roundtrip_unicode_content() {
    let src = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,こんにちは世界！\nDialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Привет мир! 🌍\nDialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,Emoji test: 🎉🚀✨🎭\n".as_bytes();
    let script = Script::parse(src);
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(script.sections().len(), reparsed.sections().len());
}

#[test]
fn roundtrip_empty_script() {
    let src = b"";
    let script = Script::parse(src);
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(script.sections().len(), reparsed.sections().len());
}

#[test]
fn roundtrip_comments_and_formatting() {
    let src = b"; This is a comment\n[Script Info]\n; Another comment\nTitle: Test with Comments\n\n; Section separator\n[Events]\n; Event comment\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Test line\n";
    let script = Script::parse(src);
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(script.sections().len(), reparsed.sections().len());
}

#[test]
fn roundtrip_multiple_iterations() {
    let src = b"[Script Info]\nTitle: Multiple Iterations\n\n[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Test content\n";
    let mut current = Script::parse(src);

    // Perform multiple serialize/parse cycles
    for _ in 0..5 {
        let serialized = current.serialize();
        current = Script::parse(serialized.as_bytes());
    }

    // Should still have the same structure
    assert!(!current.sections().is_empty());
}

#[test]
fn roundtrip_preserves_content_structure() {
    let src = b"[Script Info]\nTitle: Structure Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,32\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Test\n";
    let script = Script::parse(src);
    let original_sections = script.sections().len();

    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());

    assert_eq!(original_sections, reparsed.sections().len());
    assert!(original_sections >= 3); // Should have at least Script Info, Styles, and Events
}
