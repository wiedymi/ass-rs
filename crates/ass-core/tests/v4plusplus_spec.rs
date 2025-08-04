//! Integration tests for ASS v4++ specification support
//!
//! Tests parsing, analysis, and rendering of v4++ format extensions including
//! separate top/bottom margins, `RelativeTo` positioning, and \kt karaoke tags.
//! Ensures backward compatibility with v4+ format while properly handling
//! new v4++ features.

use ass_core::{
    analysis::styles::resolved_style::ResolvedStyle,
    parser::{
        ast::{Section, SectionType, Span, Style},
        Script,
    },
    plugin::{tags::karaoke::create_karaoke_handlers, ExtensionRegistry, TagResult},
};

#[test]
fn test_v4plusplus_full_parsing() {
    let script_text = r"[Script Info]
ScriptType: v4.00++

[V4++ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginT, MarginB, Encoding, RelativeTo
Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,25,1,0
Style: Title,Times New Roman,28,&HFF0000,&H0,&H0,&H0,1,0,0,0,110,110,2,0,1,3,1,5,20,20,30,35,1,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginT, MarginB, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,5,5,,{\kt500}Hello v4++
Dialogue: 1,0:00:05.00,0:00:10.00,Title,,0,0,10,15,,Advanced margins test
";

    let script = Script::parse(script_text).unwrap();

    // Verify Style parsing with v4++ fields
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        let default_style = &styles[0];
        assert_eq!(default_style.name, "Default");
        assert_eq!(default_style.margin_l, "10");
        assert_eq!(default_style.margin_r, "10");
        assert_eq!(default_style.margin_v, ""); // Should be empty in v4++ format
        assert_eq!(default_style.margin_t, Some("15"));
        assert_eq!(default_style.margin_b, Some("25"));
        assert_eq!(default_style.relative_to, Some("0"));

        let title_style = &styles[1];
        assert_eq!(title_style.name, "Title");
        assert_eq!(title_style.margin_t, Some("30"));
        assert_eq!(title_style.margin_b, Some("35"));
        assert_eq!(title_style.relative_to, Some("1"));
    } else {
        panic!("Styles section not found");
    }

    // Verify Event parsing with v4++ fields
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        let first_event = &events[0];
        assert_eq!(first_event.margin_l, "0");
        assert_eq!(first_event.margin_r, "0");
        assert_eq!(first_event.margin_v, ""); // Should be empty in v4++ format
        assert_eq!(first_event.margin_t, Some("5"));
        assert_eq!(first_event.margin_b, Some("5"));
        assert!(first_event.text.contains("{\\kt500}"));

        let second_event = &events[1];
        assert_eq!(second_event.margin_t, Some("10"));
        assert_eq!(second_event.margin_b, Some("15"));
    } else {
        panic!("Events section not found");
    }
}

#[test]
fn test_v4plus_backward_compatibility() {
    let script_text = r"[Script Info]
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,10,,Hello v4+
";

    let script = Script::parse(script_text).unwrap();

    // Verify Style parsing maintains v4+ compatibility
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        let style = &styles[0];
        assert_eq!(style.name, "Default");
        assert_eq!(style.margin_l, "10");
        assert_eq!(style.margin_r, "10");
        assert_eq!(style.margin_v, "15");
        assert_eq!(style.margin_t, None); // Should be None in v4+ format
        assert_eq!(style.margin_b, None); // Should be None in v4+ format
        assert_eq!(style.relative_to, None); // Should be None in v4+ format
    } else {
        panic!("Styles section not found");
    }

    // Verify Event parsing maintains v4+ compatibility
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        let event = &events[0];
        assert_eq!(event.margin_l, "0");
        assert_eq!(event.margin_r, "0");
        assert_eq!(event.margin_v, "10");
        assert_eq!(event.margin_t, None); // Should be None in v4+ format
        assert_eq!(event.margin_b, None); // Should be None in v4+ format
    } else {
        panic!("Events section not found");
    }
}

#[test]
fn test_mixed_format_robustness() {
    // Test v4++ script type with v4+ style format (missing MarginT, MarginB, RelativeTo)
    let script_text = r"[Script Info]
ScriptType: v4.00++

[V4++ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: OldStyle,Arial,16,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,OldStyle,,0,0,10,,Mixed format test
";

    let script = Script::parse(script_text).unwrap();

    // Should handle gracefully - v4++ fields should be None when not in format
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        let style = &styles[0];
        assert_eq!(style.name, "OldStyle");
        assert_eq!(style.margin_v, "15"); // v4+ margin
        assert_eq!(style.margin_t, None); // v4++ fields should be None
        assert_eq!(style.margin_b, None);
        assert_eq!(style.relative_to, None);
    } else {
        panic!("Styles section not found");
    }
}

#[test]
fn test_malformed_v4plusplus_lines() {
    // Test script with MarginT but not MarginB in format
    let script_text = r"[Script Info]
ScriptType: v4.00++

[V4++ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginT, Encoding
Style: IncompleteStyle,Arial,16,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,20,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginT, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,IncompleteStyle,,0,0,5,,Incomplete margin format
";

    let script = Script::parse(script_text).unwrap();

    // Parser should handle partial v4++ format gracefully
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        let style = &styles[0];
        assert_eq!(style.margin_t, Some("20"));
        assert_eq!(style.margin_b, None); // Not in format, should be None
        assert_eq!(style.relative_to, None);
    }

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        let event = &events[0];
        assert_eq!(event.margin_t, Some("5"));
        assert_eq!(event.margin_b, None); // Not in format, should be None
    }
}

#[test]
fn test_resolved_style_margin_logic() {
    // Test ResolvedStyle correctly resolves v4+ vs v4++ margins

    // v4+ style with single vertical margin
    let v4plus_style = Style {
        name: "V4Plus",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H80000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "2",
        shadow: "0",
        alignment: "2",
        margin_l: "10",
        margin_r: "10",
        margin_v: "15",
        margin_t: None,
        margin_b: None,
        encoding: "1",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    let resolved_v4plus = ResolvedStyle::from_style(&v4plus_style).unwrap();
    assert_eq!(resolved_v4plus.margin_l(), 10);
    assert_eq!(resolved_v4plus.margin_r(), 10);
    assert_eq!(resolved_v4plus.margin_t(), 15); // Should use margin_v
    assert_eq!(resolved_v4plus.margin_b(), 15); // Should use margin_v

    // v4++ style with separate top/bottom margins
    let v4plusplus_style = Style {
        name: "V4PlusPlus",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H80000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "2",
        shadow: "0",
        alignment: "2",
        margin_l: "10",
        margin_r: "10",
        margin_v: "0", // Should be ignored when margin_t/margin_b are present
        margin_t: Some("20"),
        margin_b: Some("25"),
        encoding: "1",
        relative_to: Some("1"),
        span: Span::new(0, 0, 0, 0),
    };

    let resolved_v4plusplus = ResolvedStyle::from_style(&v4plusplus_style).unwrap();
    assert_eq!(resolved_v4plusplus.margin_l(), 10);
    assert_eq!(resolved_v4plusplus.margin_r(), 10);
    assert_eq!(resolved_v4plusplus.margin_t(), 20); // Should use margin_t
    assert_eq!(resolved_v4plusplus.margin_b(), 25); // Should use margin_b
}

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

[Events\]
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
