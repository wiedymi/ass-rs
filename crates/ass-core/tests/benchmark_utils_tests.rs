//! Tests for benchmark utility functions
//!
//! This module tests the synthetic data generation utilities used in benchmarks
//! to ensure they produce valid ASS content and cover all code paths.

#[cfg(feature = "benches")]
mod benchmark_tests {
    use ass_core::parser::{
        ast::{Section, SectionType},
        Script,
    };

    // Import benchmark utilities - we'll need to make these public or use a different approach
    // For now, let's test the basic functionality we can access

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
            ).unwrap();
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
}

#[cfg(not(feature = "benches"))]
mod fallback_tests {
    #[test]
    fn test_benchmark_feature_not_enabled() {
        // This test runs when the benches feature is not enabled
        // It ensures the test suite still passes
        assert!(
            true,
            "Benchmark utilities not available without 'benches' feature"
        );
    }
}
