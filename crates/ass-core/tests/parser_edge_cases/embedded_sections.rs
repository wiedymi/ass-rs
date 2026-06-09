//! Tests for embedded `Fonts` and `Graphics` section parsing.
//!
//! Verifies that UU-encoded font and graphic payloads are recognized and
//! produce the corresponding `Section` variants.

use ass_core::{Script, Section};

/// Test Fonts section parsing (L171-L185)
#[test]
fn test_fonts_section_parsing() {
    let script_with_fonts = r"
[Script Info]
Title: Test

[Fonts]
fontname: Arial
0
M 0 0 L 100 100

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_with_fonts).expect("Script parsing should work");

    // Should successfully parse fonts section
    assert!(script
        .sections()
        .iter()
        .any(|section| matches!(section, Section::Fonts(_))));
}

/// Test Graphics section parsing (L171-L185)
#[test]
fn test_graphics_section_parsing() {
    let script_with_graphics = r"
[Script Info]
Title: Test

[Graphics]
filename: logo.png
0
89504E470D0A1A0A

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_with_graphics).expect("Script parsing should work");

    // Should successfully parse graphics section
    assert!(script
        .sections()
        .iter()
        .any(|section| matches!(section, Section::Graphics(_))));
}
