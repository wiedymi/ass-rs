//! Parsing tests for ASS v4++ specification support
//!
//! Covers Style/Event parsing of v4++ format extensions (separate top/bottom
//! margins, `RelativeTo`), backward compatibility with v4+, and robustness when
//! script type and format lines disagree.

use ass_core::parser::{
    ast::{Section, SectionType},
    Script,
};

#[test]
fn test_v4plusplus_full_parsing() {
    let script_text = r"[Script Info]
ScriptType: v4.00++

[V4++ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginT, MarginB, Encoding, RelativeTo
Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,25,1,0
Style: Title,Times New Roman,28,&HFF0000,&H0,&H0,&H0,1,0,0,0,110,110,2,0,1,3,1,5,20,20,30,35,1,1

[Events]
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

[Events]
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

[Events]
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

[Events]
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
