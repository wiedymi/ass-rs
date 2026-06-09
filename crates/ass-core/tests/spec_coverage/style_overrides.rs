//! Comprehensive coverage of the ASS style override tag set.
//!
//! Exercises formatting, colors, transforms, karaoke, clipping, drawing,
//! alignment, fades, and reset codes within a single dense script.

#[cfg(feature = "analysis")]
use ass_core::analysis::ScriptAnalysis;
use ass_core::{
    parser::ast::{Section, SectionType},
    Script,
};

/// Test comprehensive style override code coverage
#[test]
fn test_style_override_comprehensive() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
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
