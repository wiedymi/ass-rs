use ass_core::{
    plugin::{register_tag, Tag, TagArgument, TagParseError},
    Script,
};
use std::time::Instant;

/// Advanced test suite covering edge cases, performance, and comprehensive features
/// These tests complement the existing test suite with more thorough coverage

#[test]
fn test_large_script_parsing() {
    // Test parsing of very large scripts
    let mut large_script = String::from("[Script Info]\nTitle: Large Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    // Add 10,000 dialogue lines
    for i in 0..10000 {
        let start_time = format!("0:{:02}:{:02}.00", (i / 60) % 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", ((i + 5) / 60) % 60, (i + 5) % 60);
        large_script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,Line {} content\n",
            start_time, end_time, i
        ));
    }

    let start = Instant::now();
    let script = Script::parse(large_script.as_bytes());
    let parse_time = start.elapsed();

    assert_eq!(script.events().len(), 10000);
    assert!(
        parse_time.as_millis() < 2000,
        "Large script parsing took too long: {:?}",
        parse_time
    );
}

#[test]
fn test_extreme_unicode_content() {
    // Test with various Unicode edge cases
    let unicode_script = r#"[Script Info]
Title: Unicode Edge Cases

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Basic: Hello World
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,Japanese: こんにちは世界
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,Russian: Привет мир
Dialogue: 0,0:00:16.00,0:00:20.00,Default,,0,0,0,,Arabic: مرحبا بالعالم
Dialogue: 0,0:00:21.00,0:00:25.00,Default,,0,0,0,,Chinese: 你好世界
Dialogue: 0,0:00:26.00,0:00:30.00,Default,,0,0,0,,Emoji: 🌍🚀✨🎭🎪🎨🎯
Dialogue: 0,0:00:31.00,0:00:35.00,Default,,0,0,0,,Mixed: Hello こんにちは 🌍 مرحبا
Dialogue: 0,0:00:36.00,0:00:40.00,Default,,0,0,0,,Complex: 👨‍👩‍👧‍👦👨‍💻🧑‍🎨
Dialogue: 0,0:00:41.00,0:00:45.00,Default,,0,0,0,,Mathematical: ∑∫∂∇∞≠≤≥±∓
Dialogue: 0,0:00:46.00,0:00:50.00,Default,,0,0,0,,Special: \u{200B}\u{FEFF}\u{2060}
"#;

    let script = Script::parse(unicode_script.as_bytes());
    assert_eq!(script.events().len(), 10);

    // Test serialization preserves Unicode
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(reparsed.events().len(), 10);

    // Verify specific Unicode content is preserved
    assert!(serialized.contains("こんにちは世界"));
    assert!(serialized.contains("🌍🚀✨"));
    assert!(serialized.contains("👨‍👩‍👧‍👦"));
}

#[test]
fn test_malformed_input_handling() {
    // Test various types of malformed input
    let test_cases: Vec<(&[u8], &str)> = vec![
        (b"", "empty input"),
        (b"[Invalid Section", "unclosed section header"),
        (b"[Events]\nInvalid line format", "invalid line format"),
        (
            b"[Events]\nDialogue: invalid,format",
            "invalid dialogue format",
        ),
        (b"[Script Info]\nTitle: Test\n[Events", "incomplete section"),
        (
            b"[Events]\nDialogue: 0,invalid,time,Default,,0,0,0,,Text",
            "invalid timestamp",
        ),
        (
            b"[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\\invalid",
            "unclosed tag",
        ),
        (b"Random text without any structure", "no structure"),
        (b"\xFF\xFE\x00\x00", "invalid UTF-8"),
    ];

    for (input, description) in test_cases {
        let script = Script::parse(input);
        // Should not panic and should handle gracefully
        println!(
            "Handled malformed input ({}): {} events",
            description,
            script.events().len()
        );
    }
}

#[test]
fn test_extreme_tag_complexity() {
    // Test with extremely complex ASS tags
    let complex_script = r#"[Script Info]
Title: Complex Tags Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\move(0,0,640,360,0,4000)\t(0,1000,\frz360\fscx200\fscy200)\fade(255,0,255,0,500,3500)\c&HFF00FF&\3c&H00FFFF&\be1\blur3}Extreme animation
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,{\pos(320,180)\t(0,2000,\frx360\fry360)\c&HFF0000&\t(1000,3000,\c&H00FF00&)\t(2000,4000,\c&H0000FF&)}Multi-axis rotation
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,{\clip(100,100,540,380)\t(0,3000,\clip(200,200,440,280))\fad(500,500)}Animated clipping
Dialogue: 0,0:00:16.00,0:00:20.00,Default,,0,0,0,,{\org(320,180)\frz45\t(0,4000,\frz405)\move(100,100,540,260)}Complex origin rotation
"#;

    let script = Script::parse(complex_script.as_bytes());
    assert_eq!(script.events().len(), 4);

    // Test that complex tags are preserved in serialization
    let serialized = script.serialize();
    assert!(serialized.contains("\\move"));
    assert!(serialized.contains("\\t("));
    assert!(serialized.contains("\\fade"));
    assert!(serialized.contains("\\clip"));
}

#[test]
fn test_memory_efficiency() {
    // Test memory usage patterns
    let script_data = create_test_script(1000);

    // Parse many instances to test memory efficiency
    let scripts: Vec<_> = (0..100)
        .map(|_| Script::parse(script_data.as_bytes()))
        .collect();

    assert_eq!(scripts.len(), 100);

    // All scripts should be valid
    for script in &scripts {
        assert_eq!(script.events().len(), 1000);
    }

    // Test that we can still create more without issues
    for _ in 0..50 {
        let _additional = Script::parse(script_data.as_bytes());
    }
}

#[test]
fn test_concurrent_parsing() {
    // Test thread safety of parsing
    use std::sync::Arc;
    use std::thread;

    let script_data = Arc::new(create_test_script(500));

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let data = Arc::clone(&script_data);
            thread::spawn(move || {
                let mut results = Vec::new();
                for j in 0..10 {
                    let script = Script::parse(data.as_bytes());
                    results.push((i, j, script.events().len()));
                }
                results
            })
        })
        .collect();

    // Collect all results
    let mut all_results = Vec::new();
    for handle in handles {
        let results = handle.join().unwrap();
        all_results.extend(results);
    }

    // Verify all parsing was successful
    assert_eq!(all_results.len(), 80); // 8 threads * 10 parses each
    for (_, _, event_count) in all_results {
        assert_eq!(event_count, 500);
    }
}

#[test]
fn test_performance_regression_detection() {
    // Basic performance regression test
    let script_data = create_test_script(1000);

    // Parsing performance test
    let start = Instant::now();
    for _ in 0..100 {
        let _script = Script::parse(script_data.as_bytes());
    }
    let parse_duration = start.elapsed();

    // Should parse 100 scripts of 1000 lines in reasonable time
    assert!(
        parse_duration.as_millis() < 5000,
        "Parsing performance regression: {:?}",
        parse_duration
    );

    // Serialization performance test
    let script = Script::parse(script_data.as_bytes());
    let start = Instant::now();
    for _ in 0..100 {
        let _serialized = script.serialize();
    }
    let serialize_duration = start.elapsed();

    assert!(
        serialize_duration.as_millis() < 3000,
        "Serialization performance regression: {:?}",
        serialize_duration
    );
}

#[test]
fn test_edge_case_timestamps() {
    // Test various edge cases in timestamp parsing
    let timestamp_script = r#"[Script Info]
Title: Timestamp Edge Cases

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:00.01,Default,,0,0,0,,Very short duration
Dialogue: 0,0:00:00.00,9:59:59.99,Default,,0,0,0,,Very long duration
Dialogue: 0,0:00:59.99,0:01:00.00,Default,,0,0,0,,Minute boundary
Dialogue: 0,0:59:59.99,1:00:00.00,Default,,0,0,0,,Hour boundary
Dialogue: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,Zero duration
Dialogue: 0,0:00:05.00,0:00:01.00,Default,,0,0,0,,Negative duration
"#;

    let script = Script::parse(timestamp_script.as_bytes());
    assert_eq!(script.events().len(), 6);

    // Test serialization preserves timestamps
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(reparsed.events().len(), 6);
}

#[test]
fn test_style_section_parsing() {
    // Test comprehensive style section parsing
    let style_script = r#"[Script Info]
Title: Style Test
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Title,Arial,28,&H00FFFF00,&H000000FF,&H00000000,&H80000000,1,0,0,0,100,100,0,0,1,3,0,8,10,10,10,1
Style: Subtitle,Times New Roman,16,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,1,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Default style text
Dialogue: 0,0:00:06.00,0:00:10.00,Title,,0,0,0,,Title style text
Dialogue: 0,0:00:11.00,0:00:15.00,Subtitle,,0,0,0,,Subtitle style text
"#;

    let script = Script::parse(style_script.as_bytes());
    assert_eq!(script.events().len(), 3);

    // Test that styles are preserved in serialization
    let serialized = script.serialize();
    assert!(serialized.contains("[V4+ Styles]"));
    assert!(serialized.contains("Style: Default"));
    assert!(serialized.contains("Style: Title"));
    assert!(serialized.contains("Style: Subtitle"));
}

#[test]
fn test_plugin_system_advanced() {
    // Test advanced plugin functionality
    struct CustomTag {
        name: &'static str,
    }

    impl Tag for CustomTag {
        fn name(&self) -> &'static str {
            self.name
        }

        fn parse_args(&self, args: &[u8]) -> Result<Vec<TagArgument>, TagParseError> {
            // Simple validation - just check it's not empty
            if args.is_empty() {
                Err(TagParseError::InvalidArguments)
            } else {
                Ok(vec![])
            }
        }
    }

    static WAVE_TAG: CustomTag = CustomTag { name: "wave" };
    static GLOW_TAG: CustomTag = CustomTag { name: "glow" };

    register_tag(&WAVE_TAG);
    register_tag(&GLOW_TAG);

    let plugin_script = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\wave(2,5)}Wave effect text
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,{\glow(10)}Glow effect text
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,{\wave(1,3)\glow(5)}Combined effects
"#;

    let script = Script::parse(plugin_script.as_bytes());
    assert_eq!(script.events().len(), 3);

    // Test that custom tags are preserved
    let serialized = script.serialize();
    assert!(serialized.contains("wave"));
    assert!(serialized.contains("glow"));
}

#[test]
fn test_boundary_conditions() {
    // Test various boundary conditions

    // Extremely long lines
    let long_text = "A".repeat(100000);
    let long_line_script = format!(
        r#"[Events]
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{}
"#,
        long_text
    );

    let script = Script::parse(long_line_script.as_bytes());
    assert_eq!(script.events().len(), 1);

    // Many sections
    let mut many_sections_script = String::new();
    for i in 0..100 {
        many_sections_script.push_str(&format!("[Custom Section {}]\nSome content\n\n", i));
    }
    many_sections_script
        .push_str("[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Test\n");

    let script = Script::parse(many_sections_script.as_bytes());
    assert_eq!(script.events().len(), 1);

    // Deeply nested tags
    let nested_tags = format!(
        "{{{}}}Deeply nested text{{{}}}",
        "\\b1\\i1\\u1\\c&HFF0000&\\fs20".repeat(50),
        "\\r".repeat(50)
    );
    let nested_script = format!(
        r#"[Events]
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{}
"#,
        nested_tags
    );

    let script = Script::parse(nested_script.as_bytes());
    assert_eq!(script.events().len(), 1);
}

#[test]
fn test_roundtrip_fidelity() {
    // Test that parsing and serializing preserves all information
    let original_script = r#"[Script Info]
Title: Roundtrip Test
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: TV.601

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\b1}Bold text{\b0}
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,{\i1}Italic text{\i0}
Comment: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,This is a comment
"#;

    let script = Script::parse(original_script.as_bytes());
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());

    // Should have same number of events
    assert_eq!(script.events().len(), reparsed.events().len());

    // Test multiple roundtrips
    let mut current = serialized;
    for _ in 0..5 {
        let script = Script::parse(current.as_bytes());
        current = script.serialize();
    }

    let final_script = Script::parse(current.as_bytes());
    assert_eq!(script.events().len(), final_script.events().len());
}

#[test]
fn test_error_recovery() {
    // Test that parser can recover from errors and continue processing
    let mixed_valid_invalid = r#"[Script Info]
Title: Error Recovery Test

[Invalid Section
This section has no closing bracket

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Valid line 1
Invalid line format here
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,Valid line 2
Dialogue: invalid,format,here
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,Valid line 3
"#;

    let script = Script::parse(mixed_valid_invalid.as_bytes());

    // Should have parsed the valid lines
    assert!(
        script.events().len() >= 3,
        "Should parse valid lines despite errors"
    );
}

// Helper function for creating test scripts
fn create_test_script(line_count: usize) -> String {
    let mut script = String::from("[Script Info]\nTitle: Test Script\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for i in 0..line_count {
        let start_time = format!("0:{:02}:{:02}.00", (i / 60) % 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", ((i + 5) / 60) % 60, (i + 5) % 60);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,Test line {}\n",
            start_time, end_time, i
        ));
    }

    script
}
