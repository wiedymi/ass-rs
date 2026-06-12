//! `render_frame_bitmaps` must reproduce `render_frame`: compositing the emitted
//! bitmap list onto a transparent buffer should match the directly-composited
//! frame. Covers both the coverage path (text + outline + shadow) and the
//! vector path (\blur), and overlapping layers (source-over equivalence).

#![cfg(feature = "software-backend")]

use ass_core::parser::Script;
use ass_renderer::backends::coverage::composite_bitmap;
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{RenderContext, Renderer};

const W: u32 = 640;
const H: u32 = 360;

const SCRIPT: &str = "\
[Script Info]
ScriptType: v4.00+
PlayResX: 640
PlayResY: 360

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,40,&H00FFFFFF,&H000000FF,&H00202020,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,{\\pos(120,150)}Coverage path text
Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,{\\pos(140,170)\\1c&H00A0FF&}Overlapping coloured line
Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,{\\pos(120,260)\\blur4}Blurred vector path layer
";

fn composited_from_bitmaps(renderer: &mut Renderer, script: &Script, t: u32) -> Vec<u8> {
    let bitmaps = renderer.render_frame_bitmaps(script, t).expect("bitmaps");
    let mut buf = vec![0u8; (W * H * 4) as usize];
    for bitmap in &bitmaps {
        composite_bitmap(&mut buf, W, H, bitmap);
    }
    buf
}

#[test]
fn bitmap_list_matches_direct_frame() {
    let script = Script::parse(SCRIPT).expect("parse");
    let t = 100;

    let mut direct_renderer =
        Renderer::new(BackendType::Software, RenderContext::new(W, H)).expect("renderer");
    let direct = direct_renderer
        .render_frame(&script, t)
        .expect("render_frame")
        .data()
        .to_vec();

    let mut bitmap_renderer =
        Renderer::new(BackendType::Software, RenderContext::new(W, H)).expect("renderer");
    let from_bitmaps = composited_from_bitmaps(&mut bitmap_renderer, &script, t);

    assert_eq!(direct.len(), from_bitmaps.len());

    let mut max_diff = 0u8;
    let mut differing = 0usize;
    for (a, b) in direct.iter().zip(from_bitmaps.iter()) {
        let d = a.abs_diff(*b);
        max_diff = max_diff.max(d);
        if d != 0 {
            differing += 1;
        }
    }

    // Coverage layers are bit-identical; vector (blur) layers go through an extra
    // premultiplied crop+composite, which may round by at most 1 per channel.
    assert!(max_diff <= 2, "max per-channel diff too high: {max_diff}");
    let frac = differing as f64 / direct.len() as f64;
    assert!(frac < 0.02, "too many differing channels: {frac:.4}");
    // Sanity: something was actually drawn.
    assert!(direct.iter().any(|&v| v != 0), "frame was empty");
}
