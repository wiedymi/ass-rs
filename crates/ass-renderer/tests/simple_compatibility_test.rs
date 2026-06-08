//! Simple compatibility test without complex dependencies

use ass_core::parser::Script;

#[cfg(feature = "software-backend")]
use ass_renderer::backends::BackendType;
#[cfg(feature = "software-backend")]
use ass_renderer::renderer::{RenderContext, Renderer};

#[test]
fn test_minimal_script_parsing() {
    // Test basic parsing functionality available in minimal builds
    let script_content = r#"[Script Info]
Title: Minimal Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello World!"#;

    let script = Script::parse(script_content).expect("Failed to parse script");

    // Basic validation - this works without any backends
    let sections = script.sections();
    assert!(!sections.is_empty());
    assert!(sections.len() >= 3); // Should have Script Info, Styles, and Events sections

    println!("✅ Minimal parsing test passed!");
}

#[test]
#[cfg(feature = "software-backend")]
fn test_basic_script_parsing() {
    // Test that we can parse a basic script
    let script_content = r#"[Script Info]
Title: Basic Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello World!"#;

    let script = Script::parse(script_content).expect("Failed to parse script");

    // Test that we can create a renderer
    let context = RenderContext::new(1920, 1080);
    let mut renderer =
        Renderer::new(BackendType::Software, context).expect("Failed to create renderer");

    // Test that we can render a frame
    let frame = renderer
        .render_frame(&script, 200)
        .expect("Failed to render frame"); // 2 seconds

    // Basic validation
    assert_eq!(frame.width(), 1920);
    assert_eq!(frame.height(), 1080);
    assert!(!frame.data().is_empty());

    println!("✅ Basic rendering test passed!");
    println!("Frame size: {}x{}", frame.width(), frame.height());
    println!("Data length: {} bytes", frame.data().len());
}
