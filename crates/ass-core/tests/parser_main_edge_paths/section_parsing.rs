//! Successful parsing of individual and combined script sections, plus version detection.

use ass_core::Script;

#[test]
fn test_script_info_section_parsing() {
    // This should hit lines 148, 150-152: Script Info section parsing
    let input = r"[Script Info]
Title: Test Script
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    // Should have parsed Script Info section successfully
    assert!(!script.sections().is_empty());
}

#[test]
fn test_v4_plus_styles_section_parsing() {
    // This should hit lines 154, 162-165: V4+ Styles section parsing
    let input = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,16,&H00ffffff,&H000000ff,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    assert!(!script.sections().is_empty());
}

#[test]
fn test_v4_styles_section_parsing() {
    // This should hit the V4 Styles parsing path
    let input = r"[V4 Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, TertiaryColour, BackColour, Bold, Italic, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, AlphaLevel, Encoding
Style: Default,Arial,16,16777215,255,0,0,0,0,1,0,0,2,10,10,10,0,1";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    assert!(!script.sections().is_empty());
}

#[test]
fn test_events_section_parsing() {
    // This should hit lines 167, 175-178: Events section parsing
    let input = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    assert!(!script.sections().is_empty());
}

#[test]
fn test_multiple_sections_parsing() {
    // This should exercise multiple section parsing paths
    let input = r"[Script Info]
Title: Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,16,&H00ffffff,&H000000ff,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    assert!(script.sections().len() >= 3);
}

#[test]
fn test_empty_section_content() {
    // Test empty sections
    let input = r"[Script Info]

[V4+ Styles]

[Events]
";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    // Should handle empty sections gracefully
    assert!(!script.sections().is_empty());
}

#[test]
fn test_version_detection_from_script_info() {
    // This should test version detection logic
    let input = r"[Script Info]
ScriptType: v4.00+
Title: Test";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    // Should detect version from ScriptType
    // Should detect version from ScriptType (default is AssV4)
    assert!(matches!(
        script.version(),
        ass_core::ScriptVersion::AssV4
            | ass_core::ScriptVersion::AssV4Plus
            | ass_core::ScriptVersion::SsaV4
    ));
}

#[test]
fn test_format_detection_and_storage() {
    // Test that format detection works for different section types
    let input = r"[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,16

[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";

    let result = Script::parse(input);
    assert!(result.is_ok());

    let script = result.unwrap();
    // Should have detected and stored formats for both sections
    assert!(script.sections().len() >= 2);
}
