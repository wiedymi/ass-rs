//! Synthetic ASS content variation tests for benchmark utilities.
//!
//! Validates malformed handling, time formats, style variations, and
//! memory-efficiency content covering additional parser paths.

use ass_core::parser::{
    ast::{Section, SectionType},
    Script,
};

#[test]
fn test_malformed_content_generation() {
    // Test content that has intentional issues for testing error handling
    let malformed_script = r"[Script Info]
Title: Malformed Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&H00FFFFFF

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Normal dialogue
Dialogue: 0,0:99:99.99,0:00:05.00,Default,,0,0,0,,Invalid start time (after end)
Dialogue: 0,0:00:00.00,0:00:05.00,NonexistentStyle,,0,0,0,,Reference to non-existent style
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\invalid_tag}Text with invalid tag
";

    // This should still parse but may generate warnings/issues
    let result = Script::parse(malformed_script);
    assert!(
        result.is_ok(),
        "Parser should handle malformed content gracefully"
    );

    let script = result.unwrap();
    let events_section = script.find_section(SectionType::Events).unwrap();
    if let Section::Events(events) = events_section {
        assert_eq!(events.len(), 4);
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn test_time_format_generation() {
    // Test various time formats that might be generated
    let time_formats = vec![
        "0:00:00.00",
        "0:00:59.99",
        "0:59:59.99",
        "1:00:00.00",
        "23:59:59.99",
    ];

    for time_str in time_formats {
        let script_content = format!(
            r"[Script Info]
Title: Time Format Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,{time_str},{time_str},Default,,0,0,0,,Test dialogue
"
        );

        let result = Script::parse(&script_content);
        assert!(
            result.is_ok(),
            "Failed to parse script with time format: {time_str}"
        );
    }
}

#[test]
fn test_style_generation_variations() {
    // Test different style configurations that benchmarks might generate
    let complex_styles = r"[Script Info]
Title: Style Variations Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1
Style: Bold,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1
Style: Italic,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,1,0,0,100,100,0,0,1,0,0,2,0,0,0,1
Style: Large,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1
Style: Colored,Arial,20,&H000000FF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1
Style: Rotated,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,45,1,0,0,2,0,0,0,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Default style
Dialogue: 0,0:00:05.00,0:00:10.00,Bold,,0,0,0,,Bold style
Dialogue: 0,0:00:10.00,0:00:15.00,Italic,,0,0,0,,Italic style
Dialogue: 0,0:00:15.00,0:00:20.00,Large,,0,0,0,,Large style
Dialogue: 0,0:00:20.00,0:00:25.00,Colored,,0,0,0,,Colored style
Dialogue: 0,0:00:25.00,0:00:30.00,Rotated,,0,0,0,,Rotated style
";

    let result = Script::parse(complex_styles);
    assert!(result.is_ok(), "Failed to parse complex styles script");

    let script = result.unwrap();
    let styles_section = script.find_section(SectionType::Styles).unwrap();
    if let Section::Styles(styles) = styles_section {
        assert_eq!(styles.len(), 6);
    } else {
        panic!("Expected Styles section");
    }

    let events_section = script.find_section(SectionType::Events).unwrap();
    if let Section::Events(events) = events_section {
        assert_eq!(events.len(), 6);
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn test_memory_efficiency_content() {
    use std::fmt::Write;

    // Test content designed to test memory efficiency
    let mut efficient_content = String::from(
        r"[Script Info]
Title: Memory Efficiency Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    // Add events with repeated content to test deduplication
    let repeated_text = "This is repeated dialogue content that appears multiple times.";
    for i in 0..50 {
        writeln!(
            efficient_content,
            "Dialogue: 0,0:00:{:02}.00,0:00:{:02}.00,Default,,0,0,0,,{repeated_text}",
            i % 60,
            (i + 3) % 60
        )
        .unwrap();
    }

    let result = Script::parse(&efficient_content);
    assert!(
        result.is_ok(),
        "Failed to parse memory efficiency test script"
    );

    let script = result.unwrap();
    let events_section = script.find_section(SectionType::Events).unwrap();
    if let Section::Events(events) = events_section {
        assert_eq!(events.len(), 50);

        // Verify all events have the expected text
        for event in events {
            assert_eq!(event.text, repeated_text);
        }
    } else {
        panic!("Expected Events section");
    }
}
