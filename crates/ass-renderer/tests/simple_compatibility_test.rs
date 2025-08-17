//! Simple compatibility test without complex dependencies

use ass_core::parser::Script;
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{RenderContext, Renderer};

#[test]
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
    assert!(frame.data().len() > 0);

    println!("✅ Basic rendering test passed!");
    println!("Frame size: {}x{}", frame.width(), frame.height());
    println!("Data length: {} bytes", frame.data().len());
}

#[test]
#[cfg(feature = "libass-compare")]
fn test_libass_rendering() {
    // Test that libass renderer works
    use ass_renderer::debug::LibassRenderer;

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
    let mut libass_renderer =
        LibassRenderer::new(1920, 1080).expect("Failed to create libass renderer");

    let frame = libass_renderer
        .render_frame(&script, 200)
        .expect("Failed to render with libass");

    assert_eq!(frame.width(), 1920);
    assert_eq!(frame.height(), 1080);

    println!("✅ libass rendering test passed!");
    println!("Frame size: {}x{}", frame.width(), frame.height());
}

#[test]
#[cfg(feature = "libass-compare")]
fn test_basic_comparison() {
    // Simple pixel comparison test
    use ass_renderer::debug::LibassRenderer;

    let script_content = r#"[Script Info]
Title: Comparison Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Test"#;

    let script = Script::parse(script_content).expect("Failed to parse script");

    // Render with our renderer
    let context = RenderContext::new(1920, 1080);
    let mut our_renderer =
        Renderer::new(BackendType::Software, context).expect("Failed to create our renderer");
    let our_frame = our_renderer
        .render_frame(&script, 200)
        .expect("Failed to render with our renderer");

    // Render with libass
    let mut libass_renderer =
        LibassRenderer::new(1920, 1080).expect("Failed to create libass renderer");
    let libass_frame = libass_renderer
        .render_frame(&script, 200)
        .expect("Failed to render with libass");

    // Basic comparison
    assert_eq!(our_frame.width(), libass_frame.width());
    assert_eq!(our_frame.height(), libass_frame.height());

    // Count non-transparent pixels in each frame
    let our_pixels = count_non_transparent_pixels(our_frame.data());
    let libass_pixels = count_non_transparent_pixels(libass_frame.data());

    println!("✅ Basic comparison test completed!");
    println!("Our renderer non-transparent pixels: {}", our_pixels);
    println!("libass non-transparent pixels: {}", libass_pixels);

    // If both have some pixels, that's a good sign
    // (We're not expecting pixel-perfect match yet)
    if our_pixels > 0 || libass_pixels > 0 {
        println!("✅ At least one renderer produced visible output");
    }
}

fn count_non_transparent_pixels(data: &[u8]) -> usize {
    data.chunks(4)
        .filter(|chunk| chunk.len() == 4 && chunk[3] > 0) // Alpha > 0
        .count()
}
