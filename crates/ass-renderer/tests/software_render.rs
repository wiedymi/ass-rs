//! Pixel-level correctness tests for the software backend.
//!
//! These render small scripts and assert on the produced RGBA buffer, catching
//! regressions that "frame is non-empty" checks miss (e.g. text rendered fully
//! transparent or in the wrong color).
#![cfg(all(feature = "software-backend", feature = "analysis-integration"))]

use ass_core::parser::Script;
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{RenderContext, Renderer};

const HEAD: &str = "[Script Info]\nPlayResX: 1280\nPlayResY: 720\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,64,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,5,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

/// Render a single dialogue line at t=2s and return (width, height, RGBA bytes).
fn render(dialogue_text: &str) -> (usize, usize, Vec<u8>) {
    let script_text =
        format!("{HEAD}Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,{dialogue_text}\n");
    let script = Script::parse(&script_text).expect("parse");
    let ctx = RenderContext::new(1280, 720);
    let mut renderer = Renderer::new(BackendType::Software, ctx).expect("renderer");
    let frame = renderer.render_frame(&script, 200).expect("render");
    (
        frame.width() as usize,
        frame.height() as usize,
        frame.data().to_vec(),
    )
}

/// Count opaque (alpha >= 128) pixels satisfying `pred(r, g, b)`.
fn count_opaque<P: Fn(u8, u8, u8) -> bool>(data: &[u8], pred: P) -> u64 {
    data.chunks_exact(4)
        .filter(|px| px[3] >= 128 && pred(px[0], px[1], px[2]))
        .count() as u64
}

#[test]
fn inline_color_tag_renders_opaque_and_colored() {
    // Regression: `\c&Hbbggrr&` is 6-digit (no alpha); the fill must stay opaque
    // (it was previously rendered with alpha 0 => fully transparent / invisible).
    let (_, _, red) = render("{\\c&H0000FF&}RED");
    let red_px = count_opaque(&red, |r, g, b| r > 150 && g < 110 && b < 110);
    assert!(
        red_px > 200,
        "expected opaque red text, got {red_px} red pixels"
    );

    let (_, _, green) = render("{\\c&H00FF00&}GREEN");
    let green_px = count_opaque(&green, |r, g, b| g > 150 && r < 110 && b < 110);
    assert!(
        green_px > 200,
        "expected opaque green text, got {green_px} green pixels"
    );

    let (_, _, blue) = render("{\\c&HFF0000&}BLUE");
    let blue_px = count_opaque(&blue, |r, g, b| b > 150 && r < 110 && g < 110);
    assert!(
        blue_px > 200,
        "expected opaque blue text, got {blue_px} blue pixels"
    );
}

#[test]
fn style_primary_color_renders_opaque() {
    // A plain white-style line must produce opaque near-white pixels.
    let (_, _, w) = render("White Text");
    let white = count_opaque(&w, |r, g, b| r > 200 && g > 200 && b > 200);
    assert!(white > 200, "expected opaque white text, got {white}");
}
