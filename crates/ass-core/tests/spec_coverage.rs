//! Comprehensive ASS specification coverage integration tests
//!
//! Tests end-to-end parsing and analysis of complex ASS scripts that exercise
//! the full breadth of the ASS specification. Ensures compatibility with
//! libass extensions, Aegisub features, and `TCax` format variations.
//!
//! # Coverage Areas
//!
//! - All event types: Dialogue, Comment, Picture, Sound, Movie, Command
//! - Complete style override tag set including drawing commands
//! - Embedded media sections: [Fonts] and [Graphics] with UU-encoding
//! - Complex animation sequences with timing functions
//! - Unicode text handling and bidirectional content
//! - Performance validation for large scripts

use std::fmt::Write;

use ass_core::{
    analysis::ScriptAnalysis,
    parser::ast::{EventType, Section, SectionType},
    utils::format_ass_time,
    Script,
};

/// Complex ASS script with comprehensive spec coverage
const COMPREHENSIVE_SCRIPT: &str = r"[Script Info]
Title: Comprehensive Spec Coverage Test
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: TV.709
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,50,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Title,Impact,72,&H00FFD700,&H000000FF,&H00000000,&H80000000,1,0,0,0,120,120,2,0,1,3,3,2,0,0,0,1
Style: Subtitle,Calibri,45,&H00E6E6FA,&H000000FF,&H00404040,&H80000000,0,1,0,0,95,95,1,0,1,1,1,8,20,20,20,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text

; Basic dialogue event
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello world! Basic text without formatting.

; Complex style override tags
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,{\b1}Bold {\i1}and italic{\i0}{\b0} with {\u1}underline{\u0} and {\s1}strikeout{\s0}.

; Font and color changes
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,{\fn Impact}{\fs72}Large Impact font {\c&H0000FF&}in red color{\c}.

; Position and movement animations
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,{\pos(960,540)}Centered text that {\move(960,540,100,100,0,1000)}moves to corner.

; Rotation and scaling
Dialogue: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,{\frz45}{\fscx150}{\fscy75}Rotated and scaled text with distortion.

; Advanced animation with timing
Dialogue: 0,0:00:25.00,0:00:30.00,Default,,0,0,0,,{\t(0,2000,\fscx200\fscy50)}{\t(2000,4000,\frz360)}Complex multi-stage animation.

; Fade effects
Dialogue: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,{\fade(255,0,0,0,500,4000,4500)}Text with complex fade in and out.

; Drawing commands and vector graphics
Dialogue: 0,0:00:35.00,0:00:40.00,Default,,0,0,0,,{\p1}{\pos(960,540)}m 0 0 l 100 0 100 100 0 100{\p0} Vector square drawn.

; Karaoke effects
Dialogue: 0,0:00:40.00,0:00:45.00,Default,,0,0,0,,{\k50}Ka{\k30}ra{\k70}o{\k40}ke {\kf100}effect {\ko50}demo.

; Clipping and masking
Dialogue: 0,0:00:45.00,0:00:50.00,Default,,0,0,0,,{\clip(100,100,500,300)}Clipped text region for special effects.

; 3D perspective and shearing
Dialogue: 0,0:00:50.00,0:00:55.00,Default,,0,0,0,,{\frx45}{\fry30}{\fax0.5}{\fay0.2}3D rotated and sheared text.

; Border and shadow effects
Dialogue: 0,0:00:55.00,0:01:00.00,Default,,0,0,0,,{\bord5}{\shad3}{\3c&H00FF00&}{\4c&H0000FF&}Thick border with colored shadow.

; Unicode and bidirectional text
Dialogue: 0,0:01:00.00,0:01:05.00,Default,,0,0,0,,Mixed {\b1}English{\b0} and {\i1}العربية{\i0} text with {\u1}עברית{\u0} support.

; Line breaks and spacing
Dialogue: 0,0:01:05.00,0:01:10.00,Default,,0,0,0,,First line\NSecond line\nThird line with\hhard spaces.

; Blur and glow effects
Dialogue: 0,0:01:10.00,0:01:15.00,Default,,0,0,0,,{\blur2}{\be1}Blurred text with edge blur effect applied.

; Alignment variations
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,{\an1}Bottom left aligned text for positioning test.
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,{\an5}Middle center aligned text overlay.
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,{\an9}Top right aligned text corner.

; Color alpha and transparency
Dialogue: 0,0:01:20.00,0:01:25.00,Default,,0,0,0,,{\alpha&H80&}Semi-transparent text with {\1a&HFF&}invisible primary color.

; Complex nested animations
Dialogue: 0,0:01:25.00,0:01:35.00,Default,,0,0,0,,{\t(0,5000,\move(100,100,1820,980))}{\t(2000,8000,\frz720)}{\t(4000,10000,\fscx300\fscy300)}Ultimate complex animation sequence.

; Picture event
Picture: 0,0:01:35.00,0:01:40.00,Default,,0,0,0,,logo.png

; Sound event
Sound: 0,0:01:40.00,0:01:45.00,Default,,0,0,0,,audio.wav

; Movie event
Movie: 0,0:01:45.00,0:01:50.00,Default,,0,0,0,,video.avi

; Command event
Command: 0,0:01:50.00,0:01:55.00,Default,,0,0,0,,{\c&H00FF00&}Special command event type.

; Comment events for documentation
Comment: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,This is a comment that should be ignored in rendering.
Comment: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,{\b1}Comments can also contain formatting codes.

[Fonts\]
fontname: CustomFont.ttf
M3%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
M<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
M<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
M<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
`
end

fontname: AnotherFont.otf
M9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F
M9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F
`
end

[Graphics]
filename: background.png
M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O
M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O
M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O
`
end

filename: overlay.jpg
M2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A
M2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A
`
end
";

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_comprehensive_spec_coverage() {
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");

    // Verify script info parsing
    if let Some(Section::ScriptInfo(script_info)) = script.find_section(SectionType::ScriptInfo) {
        let title_field = script_info.fields.iter().find(|(key, _)| *key == "Title");
        assert!(title_field.is_some());
        let script_type_field = script_info
            .fields
            .iter()
            .find(|(key, _)| *key == "ScriptType");
        assert_eq!(script_type_field.map(|(_, value)| *value), Some("v4.00+"));
    } else {
        panic!("Script Info section should be present");
    }

    // Verify styles parsing
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        assert_eq!(styles.len(), 3);
        let default_style = styles.iter().find(|s| s.name == "Default");
        assert!(default_style.is_some());
    } else {
        panic!("Styles section should be present");
    }

    // Verify events parsing - should include all event types
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert!(!events.is_empty());

        // Count different event types
        let dialogue_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Dialogue))
            .count();
        let comment_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Comment))
            .count();
        let picture_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Picture))
            .count();
        let sound_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Sound))
            .count();
        let movie_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Movie))
            .count();
        let command_count = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Command))
            .count();

        assert!(dialogue_count > 0, "Should have dialogue events");
        assert!(comment_count > 0, "Should have comment events");
        assert!(picture_count > 0, "Should have picture events");
        assert!(sound_count > 0, "Should have sound events");
        assert!(movie_count > 0, "Should have movie events");
        assert!(command_count > 0, "Should have command events");
    } else {
        panic!("Events section should be present");
    }

    // Verify fonts section
    if let Some(Section::Fonts(fonts)) = script.find_section(SectionType::Fonts) {
        assert_eq!(fonts.len(), 2);
        assert_eq!(fonts[0].filename, "CustomFont.ttf");
        assert_eq!(fonts[1].filename, "AnotherFont.otf");
        assert!(!fonts[0].data_lines.is_empty());
        assert!(!fonts[1].data_lines.is_empty());
    } else {
        panic!("Fonts section should be present");
    }

    // Verify graphics section
    if let Some(Section::Graphics(graphics)) = script.find_section(SectionType::Graphics) {
        assert_eq!(graphics.len(), 2);
        assert_eq!(graphics[0].filename, "background.png");
        assert_eq!(graphics[1].filename, "overlay.jpg");
        assert!(!graphics[0].data_lines.is_empty());
        assert!(!graphics[1].data_lines.is_empty());
    } else {
        panic!("Graphics section should be present");
    }
}

#[test]
fn test_comprehensive_analysis() {
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");
    let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze script");

    // Verify dialogue analysis
    let dialogue_info = analysis.dialogue_info();
    assert!(!dialogue_info.is_empty());

    // Check for complex animations (should have high complexity scores)
    assert!(
        dialogue_info
            .iter()
            .any(|info| info.complexity_score() > 50),
        "Should have complex animation events"
    );

    // Check for events with override tags
    assert!(
        dialogue_info
            .iter()
            .any(|info| !info.text_analysis().override_tags().is_empty()),
        "Should have events with override tags"
    );

    // Check for bidirectional text
    assert!(
        dialogue_info
            .iter()
            .any(|info| info.text_analysis().has_bidi_text()),
        "Should have bidirectional text events"
    );

    // Check for Unicode complexity
    assert!(
        dialogue_info
            .iter()
            .any(|info| info.text_analysis().has_complex_unicode()),
        "Should have complex Unicode events"
    );

    // Verify overlap detection works with multiple events
    let perf_summary = analysis.performance_summary();
    // Some events should overlap (like the alignment test events)
    assert!(
        perf_summary.overlapping_events > 0,
        "Should detect overlapping events"
    );

    // Verify style analysis
    let resolved_styles = analysis.resolved_styles();
    assert!(!resolved_styles.is_empty());
}

#[test]
fn test_drawing_commands_parsing() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\p1}m 0 0 l 100 0 100 100 0 100{\p0}Square drawn.
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,{\p2}m 50 50 b 50 25 75 25 100 50 b 100 75 75 75 50 50{\p0}Bezier curve.
";

    let script = Script::parse(script_text).expect("Failed to parse drawing script");
    let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze drawing script");

    let dialogue_info = analysis.dialogue_info();
    assert_eq!(dialogue_info.len(), 2);

    for info in dialogue_info {
        let text_analysis = info.text_analysis();
        assert!(
            !text_analysis.override_tags().is_empty(),
            "Should have drawing tags"
        );

        // Check for drawing mode tags (p1, p2, p0)
        assert!(
            text_analysis
                .override_tags()
                .iter()
                .any(|tag| tag.name().starts_with('p')),
            "Should have drawing mode tags"
        );
    }
}

#[test]
fn test_all_event_types_parsing() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Normal dialogue line.
Comment: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,This is a comment.
Picture: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,image.png
Sound: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,sound.wav
Movie: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,video.mp4
Command: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,{\special}command
";

    let script = Script::parse(script_text).expect("Failed to parse all event types");

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert_eq!(events.len(), 6);

        // Verify each event type is parsed correctly
        assert!(matches!(events[0].event_type, EventType::Dialogue));
        assert!(matches!(events[1].event_type, EventType::Comment));
        assert!(matches!(events[2].event_type, EventType::Picture));
        assert!(matches!(events[3].event_type, EventType::Sound));
        assert!(matches!(events[4].event_type, EventType::Movie));
        assert!(matches!(events[5].event_type, EventType::Command));
    } else {
        panic!("Events section should be present");
    }
}

#[test]
fn test_embedded_media_integration() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test with embedded media.

[Fonts\]
fontname: test.ttf
#0V%T
`
end

[Graphics]
filename: test.png
#0V%T
`
end
";

    let script = Script::parse(script_text).expect("Failed to parse embedded media script");

    // Test fonts decoding
    if let Some(Section::Fonts(fonts)) = script.find_section(SectionType::Fonts) {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "test.ttf");

        let decoded = fonts[0].decode_data().expect("Failed to decode font data");
        assert_eq!(decoded, b"Cat"); // Known UU-encoded data
    } else {
        panic!("Fonts section should be present");
    }

    // Test graphics decoding
    if let Some(Section::Graphics(graphics)) = script.find_section(SectionType::Graphics) {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "test.png");

        let decoded = graphics[0]
            .decode_data()
            .expect("Failed to decode graphic data");
        assert_eq!(decoded, b"Cat"); // Known working UU-encoded data
    } else {
        panic!("Graphics section should be present");
    }
}

#[test]
fn test_performance_targets() {
    use std::time::Instant;

    // Test parsing performance
    let start = Instant::now();
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");
    let parse_duration = start.elapsed();

    // Should parse within 5ms target
    assert!(
        parse_duration.as_millis() < 5,
        "Parsing took {}ms, should be <5ms",
        parse_duration.as_millis()
    );

    // Test analysis performance
    let start = Instant::now();
    let _analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze script");
    let analysis_duration = start.elapsed();

    // Analysis should complete reasonably quickly
    assert!(
        analysis_duration.as_millis() < 50,
        "Analysis took {}ms, should be <50ms",
        analysis_duration.as_millis()
    );
}

#[test]
fn test_memory_efficiency() {
    let script = Script::parse(COMPREHENSIVE_SCRIPT).expect("Failed to parse comprehensive script");

    // Verify zero-copy design by checking string references
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        if let Some(first_event) = events.first() {
            // Text should reference original source, not be copied
            let original_ptr = COMPREHENSIVE_SCRIPT.as_ptr() as usize;
            let text_ptr = first_event.text.as_ptr() as usize;

            assert!(
                text_ptr >= original_ptr,
                "Text should reference original source for zero-copy design"
            );
            assert!(
                text_ptr < original_ptr + COMPREHENSIVE_SCRIPT.len(),
                "Text should reference original source for zero-copy design"
            );
        }
    }
}

#[test]
fn test_empty_script_handling() {
    let empty_script = "";
    let script = Script::parse(empty_script).expect("Should handle empty script");

    assert!(script.sections().is_empty());
}

#[test]
fn test_malformed_script_resilience() {
    let malformed_script = r"[V4+ Styles]
Format: Name, Fontname
Style: Incomplete

[Events\]
Format: Layer, Start, End, Text
Dialogue: 0,invalid_time,another_invalid,Malformed event
";

    // Should not panic on malformed input
    let result = Script::parse(malformed_script);
    // May succeed with defaults or fail gracefully
    if let Ok(script) = result {
        // If parsing succeeds, analysis should handle gracefully
        let _analysis_result = ScriptAnalysis::analyze(&script);
    } else {
        // Graceful failure is acceptable for malformed input
    }
}

#[test]
fn test_large_script_handling() {
    // Generate a large script with many events
    let mut large_script = String::from(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    // Add 1000 dialogue events
    for i in 0..1000 {
        let start_cs = i * 500; // 5 second intervals (500 centiseconds)
        let end_cs = start_cs + 400; // 4 second duration (400 centiseconds)

        writeln!(
            large_script,
            "Dialogue: 0,{},{},Default,,0,0,0,,Event {} with some text content.",
            format_ass_time(start_cs),
            format_ass_time(end_cs),
            i
        )
        .unwrap();
    }

    let script = Script::parse(&large_script).expect("Failed to parse large script");

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert_eq!(events.len(), 1000);
    } else {
        panic!("Events section should be present");
    }

    // Analysis should handle large scripts efficiently
    let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze large script");
    assert_eq!(analysis.dialogue_info().len(), 1000);
}

/// Test comprehensive style override code coverage
#[test]
fn test_style_override_comprehensive() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text

; Basic text formatting
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\b1}Bold{\b0} {\i1}Italic{\i0} {\u1}Underline{\u0} {\s1}Strikeout{\s0}

; Font modifications
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,{\fn Arial}{\fs24}Font change {\fscx150}{\fscy75}Scaling

; Color changes (all formats)
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,{\c&H0000FF&}Red {\1c&H00FF00&}Green {\2c&H00FF00&}{\3c&HFF0000&}{\4c&H0000FF&}Colors

; Alpha and transparency
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,{\alpha&H80&}Semi-transparent {\1a&HFF&}{\2a&H00&}{\3a&H40&}{\4a&H80&}Alpha

; Position and movement
Dialogue: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,{\pos(100,200)}Position {\move(100,200,300,400)}{\org(150,150)}Movement

; Rotation and transforms
Dialogue: 0,0:00:25.00,0:00:30.00,Default,,0,0,0,,{\frx15}{\fry30}{\frz45}Rotation {\fax0.5}{\fay0.2}Shearing

; Blur and border effects
Dialogue: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,{\blur2}{\be1}Blur {\bord3}{\shad2}Border and shadow

; Animation and transitions
Dialogue: 0,0:00:35.00,0:00:40.00,Default,,0,0,0,,{\t(0,1000,\fscx200)}{\t(1000,2000,\fscy50)}Complex animations

; Karaoke effects (all variants)
Dialogue: 0,0:00:40.00,0:00:45.00,Default,,0,0,0,,{\k50}Ka{\kf100}ra{\ko75}o{\kt25}ke effects

; Clipping and masking
Dialogue: 0,0:00:45.00,0:00:50.00,Default,,0,0,0,,{\clip(50,50,150,150)}Rectangular clip {\iclip(m 0 0 l 100 0 100 100 0 100)}Vector clip

; Drawing mode and vector graphics
Dialogue: 0,0:00:50.00,0:00:55.00,Default,,0,0,0,,{\p1}{\pos(200,200)}m 0 0 l 50 0 50 50 0 50{\p0} Vector drawing

; Alignment variations
Dialogue: 0,0:00:55.00,0:01:00.00,Default,,0,0,0,,{\an1}Bottom left {\an2}Bottom center {\an3}Bottom right
Dialogue: 0,0:00:55.00,0:01:00.00,Default,,0,0,0,,{\an4}Middle left {\an5}Middle center {\an6}Middle right
Dialogue: 0,0:00:55.00,0:01:00.00,Default,,0,0,0,,{\an7}Top left {\an8}Top center {\an9}Top right

; Fade effects (all variants)
Dialogue: 0,0:01:00.00,0:01:05.00,Default,,0,0,0,,{\fad(500,1000)}Simple fade {\fade(255,0,0,0,500,4000,4500)}Complex fade

; Spacing and line breaks
Dialogue: 0,0:01:05.00,0:01:10.00,Default,,0,0,0,,{\fsp5}Letter spacing\N{\fsp-2}Tight spacing\n{\fsp0}Normal spacing

; Complex nested overrides
Dialogue: 0,0:01:10.00,0:01:15.00,Default,,0,0,0,,{\b1\i1\u1\s1\fscx150\fscy75\frz45\c&H0000FF&\alpha&H80&}All effects combined

; Reset codes
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,{\b1}Bold{\r}Reset to default{\rDefault}Reset to style
";

    let script = Script::parse(script_text).expect("Failed to parse style override script");

    #[cfg(feature = "analysis")]
    {
        let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze script");

        // Verify all events were parsed
        assert!(!analysis.dialogue_info().is_empty());

        // Check that complex animations are detected
        let complex_events = analysis
            .dialogue_info()
            .iter()
            .filter(|info| info.animation_score() > 2)
            .count();
        assert!(complex_events > 0, "Should detect complex animations");

        // Verify performance scoring accounts for complexity
        let performance = analysis.performance_summary();
        assert!(performance.performance_score <= 100);
    }

    // Verify events section parsing
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert!(
            events.len() >= 15,
            "Should have parsed all override test events"
        );

        // Verify various event types and content
        let text_with_overrides = events
            .iter()
            .filter(|e| e.text.contains('{') && e.text.contains('}'))
            .count();
        assert!(
            text_with_overrides >= 10,
            "Should have events with style overrides"
        );
    } else {
        panic!("Events section should be present");
    }
}

/// Test text analysis edge cases and Unicode handling
#[test]
fn test_text_analysis_edge_cases() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text

; Empty and whitespace-only events
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,{\b1}{\b0}

; Unicode text (various scripts)
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,English text with unicode: cafe, naive, resume
Dialogue: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,Cyrillic text sample
Dialogue: 0,0:00:25.00,0:00:30.00,Default,,0,0,0,,Arabic: العربية (bidirectional text)
Dialogue: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,Hebrew: עברית (right-to-left)
Dialogue: 0,0:00:35.00,0:00:40.00,Default,,0,0,0,,Chinese CJK characters
Dialogue: 0,0:00:40.00,0:00:45.00,Default,,0,0,0,,Japanese hiragana katakana kanji
Dialogue: 0,0:00:45.00,0:00:50.00,Default,,0,0,0,,Korean Hangul script
Dialogue: 0,0:00:50.00,0:00:55.00,Default,,0,0,0,,Unicode emoji test

; Complex bidirectional text mixing
Dialogue: 0,0:00:55.00,0:01:00.00,Default,,0,0,0,,Mixed: English العربية English עברית English

; Line breaks and spacing variations
Dialogue: 0,0:01:00.00,0:01:05.00,Default,,0,0,0,,Line 1\NLine 2\nLine 3\hHard\hSpaces
Dialogue: 0,0:01:05.00,0:01:10.00,Default,,0,0,0,,Multiple\N\NEmpty\n\nLines

; Special Unicode characters
Dialogue: 0,0:01:10.00,0:01:15.00,Default,,0,0,0,,Zero-width joiners and special spacing
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,Combining diacritical marks
Dialogue: 0,0:01:20.00,0:01:25.00,Default,,0,0,0,,Control characters and direction marks

; Very long text lines
Dialogue: 0,0:01:25.00,0:01:30.00,Default,,0,0,0,,Very long text that exceeds normal subtitle length limits and should test text analysis algorithms for performance and correctness when dealing with extended content that might wrap across multiple lines or cause rendering performance issues in complex subtitle rendering scenarios.

; Malformed Unicode sequences (should be handled gracefully)
Dialogue: 0,0:01:30.00,0:01:35.00,Default,,0,0,0,,Malformed sequences with replacement characters

; Mixed content with overrides
Dialogue: 0,0:01:35.00,0:01:40.00,Default,,0,0,0,,{\b1}Bold English{\b0} {\i1}italic Arabic{\i0} {\u1}underlined Russian{\u0}

; Complex escape sequences
Dialogue: 0,0:01:40.00,0:01:45.00,Default,,0,0,0,,Escaped: \{ \} \\ \n literal braces and backslashes

; Performance stress test with many overrides
Dialogue: 0,0:01:45.00,0:01:50.00,Default,,0,0,0,,{\b1}A{\b0}{\i1}B{\i0}{\u1}C{\u0}{\s1}D{\s0}{\fscx120}E{\fscx100}{\fscy80}F{\fscy100}{\frz10}G{\frz0}H
";

    let script = Script::parse(script_text).expect("Failed to parse Unicode test script");

    #[cfg(feature = "analysis")]
    {
        let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze Unicode script");

        // Verify all events were parsed including Unicode content
        assert!(analysis.dialogue_info().len() >= 15);

        // Check that text analysis handles various Unicode scripts
        let has_bidi_content = analysis.dialogue_info().iter().any(|info| {
            info.event().text.contains("العربية") || info.event().text.contains("עברית")
        });
        assert!(has_bidi_content, "Should detect bidirectional text");

        // Verify empty/whitespace events are handled
        let empty_or_whitespace = analysis
            .dialogue_info()
            .iter()
            .filter(|info| {
                info.event().text.trim().is_empty()
                    || info
                        .event()
                        .text
                        .chars()
                        .all(|c| c.is_whitespace() || c == '{' || c == '}')
            })
            .count();
        assert!(
            empty_or_whitespace >= 2,
            "Should handle empty/whitespace events"
        );

        // Check performance with complex content
        let performance = analysis.performance_summary();
        assert!(performance.performance_score > 0);
    }
}

/// Test comprehensive error recovery and malformed input handling
#[test]
fn test_error_recovery_comprehensive() {
    let malformed_script = r"[Script Info]
Title: Error Recovery Test
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Incomplete,Arial,20  // Missing fields should be handled
Style: ,,,,,,,,,,,,,,,,,,,,,,   // Empty fields

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text

; Valid events
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Normal dialogue

; Malformed timing
Dialogue: 0,invalid_time,0:00:10.00,Default,,0,0,0,,Bad start time
Dialogue: 0,0:00:15.00,invalid_time,Default,,0,0,0,,Bad end time
Dialogue: 0,0:00:25.00,0:00:20.00,Default,,0,0,0,,End before start

; Missing required fields
Dialogue: 0,0:00:30.00,0:00:35.00  // Incomplete line
Dialogue: 0,0:00:40.00,0:00:45.00,NonexistentStyle,,0,0,0,,Missing style reference

; Malformed style overrides
Dialogue: 0,0:00:50.00,0:00:55.00,Default,,0,0,0,,{Unclosed override
Dialogue: 0,0:01:00.00,0:01:05.00,Default,,0,0,0,,{Invalid}override}content
Dialogue: 0,0:01:10.00,0:01:15.00,Default,,0,0,0,,{\invalid_tag}Unknown tag
Dialogue: 0,0:01:20.00,0:01:25.00,Default,,0,0,0,,{\\}Empty tag

; Binary/control characters
Dialogue: 0,0:01:30.00,0:01:35.00,Default,,0,0,0,,Content with binary data

; Invalid section
[Unknown Section]
SomeKey: SomeValue
AnotherKey: AnotherValue

; Partial sections
[Incomplete Section

[Another Section]
";

    // Should parse successfully despite errors
    let script = Script::parse(malformed_script).expect("Should parse with error recovery");

    // Should have accumulated issues but still produce a usable script
    assert!(!script.issues().is_empty(), "Should detect parsing issues");

    // Should still have valid sections
    assert!(script.find_section(SectionType::ScriptInfo).is_some());
    assert!(script.find_section(SectionType::Styles).is_some());
    assert!(script.find_section(SectionType::Events).is_some());

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        // Should have parsed at least the valid events
        let valid_events = events
            .iter()
            .filter(|e| !e.start.is_empty() && !e.end.is_empty())
            .count();
        assert!(valid_events > 0, "Should parse some valid events");
    }

    #[cfg(feature = "analysis")]
    {
        // Analysis should handle malformed script gracefully
        let analysis_result = ScriptAnalysis::analyze(&script);
        assert!(
            analysis_result.is_ok(),
            "Analysis should handle errors gracefully"
        );

        if let Ok(analysis) = analysis_result {
            // Should detect issues through linting
            assert!(
                !analysis.lint_issues().is_empty(),
                "Should detect lint issues"
            );
        }
    }
}

/// Test performance edge cases and stress scenarios
#[test]
fn test_performance_edge_cases() {
    // Test very dense overlapping events
    let mut dense_script = String::from(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    // Create many overlapping events in a short time span
    for i in 0..100 {
        let start_cs = i * 10; // Every 0.1 seconds
        let end_cs = start_cs + 500; // 5 second duration each
        writeln!(
            dense_script,
            "Dialogue: {},{},{},Default,,0,0,0,,Overlapping event {} content.",
            i % 10, // Different layers
            format_ass_time(start_cs),
            format_ass_time(end_cs),
            i
        )
        .unwrap();
    }

    let script = Script::parse(&dense_script).expect("Failed to parse dense script");

    #[cfg(feature = "analysis")]
    {
        let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze dense script");

        // Should detect many overlaps
        let performance = analysis.performance_summary();
        assert!(
            performance.overlapping_events > 50,
            "Should detect many overlapping events"
        );
        assert!(
            performance.performance_score < 90,
            "Performance score should reflect complexity"
        );
    }

    // Test deeply nested style overrides
    let nested_overrides = format!(
        "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{}Text{}",
        "{\\b1\\i1\\u1\\s1\\fscx150\\fscy75\\frz45\\c&H0000FF&\\alpha&H80&\\pos(100,200)\\move(100,200,300,400)\\t(0,1000,\\fscx200)\\t(1000,2000,\\fscy50)\\blur2\\be1\\bord3\\shad2}".repeat(5),
        "{\\r}".repeat(5)
    );

    let complex_script = format!(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
{nested_overrides}
"
    );

    let script = Script::parse(&complex_script).expect("Failed to parse complex script");

    #[cfg(feature = "analysis")]
    {
        let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze complex script");

        // Should handle complex analysis
        assert!(!analysis.dialogue_info().is_empty());
        let complex_animation_count = analysis
            .dialogue_info()
            .iter()
            .filter(|info| info.animation_score() > 5)
            .count();
        assert!(
            complex_animation_count > 0,
            "Should detect very complex animations"
        );
    }
}
