//! Synthetic ASS content generation tests for benchmark utilities.
//!
//! Validates parsing of basic, complex, stress, and Unicode synthetic
//! scripts to ensure they produce valid ASS content covering parser paths.

use ass_core::parser::{
    ast::{Section, SectionType},
    Script,
};

#[test]
fn test_synthetic_script_generation() {
    // Test basic script parsing with synthetic content
    let simple_script = r"[Script Info]
Title: Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Simple test dialogue
";

    let result = Script::parse(simple_script);
    assert!(result.is_ok(), "Failed to parse simple synthetic script");

    let script = result.unwrap();
    assert!(script.find_section(SectionType::Events).is_some());
    assert!(script.find_section(SectionType::Styles).is_some());
}

#[test]
fn test_complex_synthetic_content() {
    // Test script with complex formatting and override tags
    let complex_script = r"[Script Info]
Title: Complex Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1
Style: Complex,Arial,24,&H0000FFFF,&H000000FF,&H00000000,&H00000000,1,1,0,0,110,110,2,15,1,2,1,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\b1}Bold text{\b0} with {\i1}italic{\i0}
Dialogue: 0,0:00:05.00,0:00:10.00,Complex,,0,0,0,,{\c&H0000FF&}Red text{\c} with {\pos(100,200)}positioning
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,{\k50}Ka{\k30}ra{\k40}o{\k35}ke {\k45}text
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,{\fad(500,500)}Fade in and out
";

    let result = Script::parse(complex_script);
    assert!(result.is_ok(), "Failed to parse complex synthetic script");

    let script = result.unwrap();
    let events_section = script.find_section(SectionType::Events).unwrap();
    if let Section::Events(events) = events_section {
        assert_eq!(events.len(), 4);
    } else {
        panic!("Expected Events section");
    }

    // Verify complex formatting is preserved
    let events_section = script.find_section(SectionType::Events).unwrap();
    if let Section::Events(events) = events_section {
        let first_event = &events[0];
        assert!(first_event.text.contains(r"{\b1}"));
        assert!(first_event.text.contains(r"{\i1}"));

        let karaoke_event = &events[2];
        assert!(karaoke_event.text.contains(r"{\k"));
    }
}

#[test]
fn test_stress_test_content_generation() {
    use std::fmt::Write;

    // Generate content that would stress the parser
    let mut large_content = String::from(
        r"[Script Info]
Title: Stress Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    // Add many events to test performance
    for i in 0..100 {
        let start_time = format!("0:00:{:02}.00", i % 60);
        let end_time = format!("0:00:{:02}.00", (i + 5) % 60);
        writeln!(
            large_content,
            "Dialogue: 0,{start_time},{end_time},Default,,0,0,0,,Event {i} with some text content"
        )
        .unwrap();
    }

    let result = Script::parse(&large_content);
    assert!(result.is_ok(), "Failed to parse large synthetic script");

    let script = result.unwrap();
    let events_section = script.find_section(SectionType::Events).unwrap();
    if let Section::Events(events) = events_section {
        assert_eq!(events.len(), 100);
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn test_unicode_content_generation() {
    // Test with Unicode content that might be used in benchmarks
    let unicode_script = r"[Script Info]
Title: Unicode Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello, 世界! 🌍
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Здравствуй, мир! 🚀
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,مرحبا بالعالم! 🎉
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,こんにちは、世界！ 🦀
";

    let result = Script::parse(unicode_script);
    assert!(result.is_ok(), "Failed to parse Unicode synthetic script");

    let script = result.unwrap();
    let events_section = script.find_section(SectionType::Events).unwrap();
    if let Section::Events(events) = events_section {
        assert_eq!(events.len(), 4);

        // Verify Unicode content is preserved
        assert!(events[0].text.contains("世界"));
        assert!(events[1].text.contains("мир"));
        assert!(events[2].text.contains("بالعالم"));
        assert!(events[3].text.contains("こんにちは"));
    } else {
        panic!("Expected Events section");
    }
}
