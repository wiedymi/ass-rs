//! Karaoke `\kt` tag tests for ASS v4++ specification support
//!
//! Exercises the `\kt` tag handler in isolation via the extension registry and
//! verifies that scripts containing multiple `\kt` tags parse correctly
//! alongside v4++ margins.

use ass_core::{
    parser::{
        ast::{Section, SectionType},
        Script,
    },
    plugin::{tags::karaoke::create_karaoke_handlers, ExtensionRegistry, TagResult},
};

#[test]
fn test_kt_tag_handler() {
    // Test that the \kt tag handler is working correctly
    let mut registry = ExtensionRegistry::new();

    // Register karaoke handlers
    for handler in create_karaoke_handlers() {
        registry.register_tag_handler(handler).unwrap();
    }

    // Test that kt handler is registered
    assert!(registry.has_tag_handler("kt"));

    // Test valid kt tag processing
    let result = registry.process_tag("kt", "500");
    assert!(matches!(result, Some(TagResult::Processed)));

    // Test invalid kt tag processing
    let result = registry.process_tag("kt", "invalid");
    assert!(matches!(result, Some(TagResult::Failed(_))));

    // Test zero value
    let result = registry.process_tag("kt", "0");
    assert!(matches!(result, Some(TagResult::Processed)));

    // Test large value
    let result = registry.process_tag("kt", "999999");
    assert!(matches!(result, Some(TagResult::Processed)));
}

#[test]
fn test_v4plusplus_script_with_kt_tags() {
    // Integration test with actual \kt tags in script text
    let script_text = r"[Script Info]
ScriptType: v4.00++

[V4++ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginT, MarginB, Encoding, RelativeTo
Style: Karaoke,Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,20,20,1,0

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginT, MarginB, Effect, Text
Dialogue: 0,0:00:00.00,0:00:10.00,Karaoke,,0,0,0,0,,{\kt100}Ka{\kt150}ra{\kt200}o{\kt100}ke {\kt300}test
";

    let script = Script::parse(script_text).unwrap();

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        let event = &events[0];

        // Verify the text contains multiple \kt tags
        assert!(event.text.contains("{\\kt100}"));
        assert!(event.text.contains("{\\kt150}"));
        assert!(event.text.contains("{\\kt200}"));
        assert!(event.text.contains("{\\kt300}"));

        // Verify v4++ margins are parsed
        assert_eq!(event.margin_t, Some("0"));
        assert_eq!(event.margin_b, Some("0"));
    } else {
        panic!("Events section not found");
    }
}
