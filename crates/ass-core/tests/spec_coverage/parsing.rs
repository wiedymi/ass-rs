//! Focused parsing tests for drawing commands, event types, and embedded media.
//!
//! Each test feeds a small, self-contained script to confirm individual ASS
//! features parse and analyze as expected.

use ass_core::{
    analysis::ScriptAnalysis,
    parser::ast::{EventType, Section, SectionType},
    Script,
};

#[test]
fn test_drawing_commands_parsing() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
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

[Events]
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

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test with embedded media.

[Fonts]
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
