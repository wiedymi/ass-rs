//! Coverage tests for Unicode handling, error recovery, version detection, and
//! binary data in `parser/main.rs`.

use ass_core::parser::Script;

#[test]
fn test_parser_with_unicode_edge_cases() {
    // Test various Unicode edge cases
    let unicode_content = "[Script Info]\n\
                          Title: Test with 🎵 emojis and ñáéíóú accents\n\
                          ScriptType: v4.00+\n\
                          \n\
                          [V4+ Styles]\n\
                          Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
                          Style: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\
                          \n\
                          [Events]\n\
                          Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
                          Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Unicode text: 日本語 العربية Русский\n";

    let script = Script::parse(unicode_content);
    assert!(script.is_ok());

    let script = script.unwrap();
    assert!(!script.sections().is_empty());
}

#[test]
fn test_parser_error_recovery() {
    // Test parser's ability to recover from errors and continue parsing
    let content_with_errors = "[Script Info]\n\
                              Title: Test\n\
                              Invalid: Line\n\
                              \n\
                              [Invalid Section Name]\n\
                              Some content\n\
                              \n\
                              [V4+ Styles]\n\
                              Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
                              Style: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\
                              Invalid: Style line\n\
                              \n\
                              [Events]\n\
                              Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
                              Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Valid event\n\
                              Invalid: Event line\n";

    let script = Script::parse(content_with_errors);
    assert!(script.is_ok());

    let script = script.unwrap();

    // Should still parse valid parts
    assert!(!script.sections().is_empty());

    // Should have parsing issues for invalid parts
    assert!(!script.issues().is_empty());

    // Should have parsed at least some sections
    assert!(!script.sections().is_empty());
}

#[test]
fn test_parser_version_detection() {
    // Test different ASS version strings and formats
    let version_tests = vec![
        "[Script Info]\nScriptType: v4.00+\n",
        "[Script Info]\nScriptType: v4.00\n",
        "[Script Info]\nScriptType: ASS\n",
        "[Script Info]\n!: This is v4.00+ style\n",
        "[Script Info]\nTitle: Test\n", // No explicit version
    ];

    for content in version_tests {
        let script = Script::parse(content);
        assert!(script.is_ok(), "Failed to parse version test: {content}");

        let script = script.unwrap();
        // Should detect some version information
        println!(
            "Version detected for '{}': {:?}",
            content.lines().nth(1).unwrap_or(""),
            script.version()
        );
    }
}

#[test]
fn test_parser_with_binary_data() {
    // Test parser behavior when encountering binary-like data
    let mut binary_content = b"[Script Info]\nTitle: Test\n".to_vec();
    binary_content.extend_from_slice(&[0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD]);
    binary_content.extend_from_slice(b"\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    let content_str = String::from_utf8_lossy(&binary_content);
    let script = Script::parse(&content_str);

    assert!(script.is_ok());
    let script = script.unwrap();

    // Should handle binary data gracefully (likely with replacement characters)
    assert!(!script.sections().is_empty());
}
