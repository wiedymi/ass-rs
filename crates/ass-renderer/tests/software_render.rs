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
    render_at(200, dialogue_text)
}

/// Render a single dialogue line at `time_cs` and return (width, height, RGBA bytes).
fn render_at(time_cs: u32, dialogue_text: &str) -> (usize, usize, Vec<u8>) {
    let script_text =
        format!("{HEAD}Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,{dialogue_text}\n");
    let script = Script::parse(&script_text).expect("parse");
    let ctx = RenderContext::new(1280, 720);
    let mut renderer = Renderer::new(BackendType::Software, ctx).expect("renderer");
    let frame = renderer.render_frame(&script, time_cs).expect("render");
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

/// Width in pixels of the bounding box of opaque (alpha >= 128) pixels.
fn opaque_bbox_width(data: &[u8], width: usize) -> usize {
    let (mut min_x, mut max_x) = (usize::MAX, 0usize);
    for (i, px) in data.chunks_exact(4).enumerate() {
        if px[3] >= 128 {
            let x = i % width;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }
    }
    if min_x == usize::MAX {
        0
    } else {
        max_x - min_x + 1
    }
}

/// Count pixels with any coverage (alpha > 0).
fn count_covered(data: &[u8]) -> u64 {
    data.chunks_exact(4).filter(|px| px[3] > 0).count() as u64
}

/// Left edge (min x) of the bounding box of opaque (alpha >= 128) pixels.
fn bbox_min_x(data: &[u8], width: usize) -> usize {
    let mut min_x = usize::MAX;
    for (i, px) in data.chunks_exact(4).enumerate() {
        if px[3] >= 128 {
            min_x = min_x.min(i % width);
        }
    }
    if min_x == usize::MAX {
        0
    } else {
        min_x
    }
}

/// Height in pixels of the bounding box of opaque (alpha >= 128) pixels.
fn opaque_bbox_height(data: &[u8], width: usize) -> usize {
    let (mut min_y, mut max_y) = (usize::MAX, 0usize);
    for (i, px) in data.chunks_exact(4).enumerate() {
        if px[3] >= 128 {
            let y = i / width;
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }
    if min_y == usize::MAX {
        0
    } else {
        max_y - min_y + 1
    }
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

#[test]
fn inline_tag_does_not_add_horizontal_gap() {
    // Regression: a mid-line override (e.g. `\c`) must not double-advance the pen
    // and leave a one-segment-width gap before the following run.
    let (pw, _, plain) = render("HelloWorld");
    let (sw, _, split) = render("Hello{\\c&H00FF00&}World");
    let plain_w = opaque_bbox_width(&plain, pw);
    let split_w = opaque_bbox_width(&split, sw);
    assert!(plain_w > 0 && split_w > 0, "both lines should render");
    assert!(
        split_w <= plain_w * 6 / 5,
        "inline color split widened the line ({split_w}px vs plain {plain_w}px) — gap regression"
    );
}

#[test]
fn frz_rotation_changes_geometry() {
    // Regression: `\frz` must actually rotate. tiny-skia's pre_rotate takes
    // degrees; passing radians made rotations ~flat. A wide line rotated 45°
    // spans far more vertically than the unrotated line.
    let (pw, _, plain) = render("ROTATME");
    let (rw, _, rot) = render("{\\frz45}ROTATME");
    let plain_h = opaque_bbox_height(&plain, pw);
    let rot_h = opaque_bbox_height(&rot, rw);
    assert!(plain_h > 0 && rot_h > 0, "both lines should render");
    assert!(
        rot_h >= plain_h * 3 / 2,
        "expected \\frz45 to increase vertical extent (rotated {rot_h}px vs plain {plain_h}px)"
    );
}

#[test]
fn karaoke_uses_primary_and_secondary_not_yellow() {
    // Default style: primary white, secondary red (&H000000FF). A `\k` syllable
    // is the secondary colour until sung, then the primary colour — and never the
    // old hard-coded yellow.
    let is_yellow = |r: u8, g: u8, b: u8| r > 150 && g > 150 && b < 100;

    // Before the syllable's time: secondary (red).
    let (_, _, early) = render_at(0, "{\\k100}KARAOKE");
    assert_eq!(count_opaque(&early, is_yellow), 0, "no yellow karaoke fill");
    let red = count_opaque(&early, |r, g, b| r > 150 && g < 110 && b < 110);
    assert!(
        red > 200,
        "unsung karaoke should be secondary red, got {red}"
    );

    // After the syllable's time: primary (white).
    let (_, _, late) = render_at(500, "{\\k100}KARAOKE");
    assert_eq!(count_opaque(&late, is_yellow), 0, "no yellow karaoke fill");
    let white = count_opaque(&late, |r, g, b| r > 200 && g > 200 && b > 200);
    assert!(
        white > 200,
        "sung karaoke should be primary white, got {white}"
    );
}

#[test]
fn blur_tag_spreads_coverage() {
    // Regression: `\blur` was silently dropped by the segmenter (output identical
    // to no blur). A strong blur must spread coverage well beyond the sharp text.
    let (_, _, plain) = render("BLURME");
    let (_, _, blurred) = render("{\\blur15}BLURME");
    let plain_cov = count_covered(&plain);
    let blur_cov = count_covered(&blurred);
    assert!(plain_cov > 0, "plain text should render");
    assert!(
        blur_cov >= plain_cov * 3 / 2,
        "\\blur should spread coverage (blurred {blur_cov}px vs plain {plain_cov}px)"
    );
}

#[test]
fn clip_and_iclip_partition_the_text() {
    // Regression: `\clip`/`\iclip` were dropped by the segmenter (no-op), and the
    // inverse flag only toggled mask anti-aliasing. `\clip` keeps pixels inside the
    // rect, `\iclip` keeps those outside; together they reconstruct the full text.
    // The Default style is centre-aligned (an5), so it straddles (640,360).
    let (_, _, full) = render("CLIPPED");
    let (_, _, clipped) = render("{\\clip(0,0,640,360)}CLIPPED");
    let (_, _, iclipped) = render("{\\iclip(0,0,640,360)}CLIPPED");
    let full_cov = count_covered(&full);
    let clip_cov = count_covered(&clipped);
    let iclip_cov = count_covered(&iclipped);

    assert!(full_cov > 0, "unclipped text should render");
    assert!(
        clip_cov > 0 && clip_cov < full_cov,
        "clip should remove some pixels ({clip_cov} of {full_cov})"
    );
    assert!(
        iclip_cov > 0 && iclip_cov < full_cov,
        "iclip should remove some pixels ({iclip_cov} of {full_cov})"
    );
    assert_ne!(clip_cov, iclip_cov, "clip and iclip must differ");
    let sum = clip_cov + iclip_cov;
    assert!(
        sum.abs_diff(full_cov) <= full_cov / 20,
        "clip ({clip_cov}) + iclip ({iclip_cov}) should reconstruct full ({full_cov})"
    );
}

#[test]
fn frx_fry_rotation_does_not_vanish() {
    // Regression: \frx/\fry sheared around the screen origin, flinging text
    // off-frame (it vanished entirely for angles >= ~30deg). They now shear around
    // the text centre, so the glyphs stay on screen.
    let (pw, _, plain) = render("FLIPME");
    let (_, _, frx) = render("{\\frx55}FLIPME");
    let (_, _, fry) = render("{\\fry55}FLIPME");
    let plain_h = opaque_bbox_height(&plain, pw);

    assert!(count_covered(&frx) > 0, "\\frx55 text vanished off-screen");
    assert!(count_covered(&fry) > 0, "\\fry55 text vanished off-screen");
    let frx_h = opaque_bbox_height(&frx, pw);
    assert!(
        frx_h > plain_h,
        "\\frx should add vertical skew ({frx_h}px vs unrotated {plain_h}px)"
    );
}

#[test]
fn fax_shear_applies_and_stays_centered() {
    // Regression: \fax/\fay were dropped by the segmenter, and once applied the
    // shear ran around the screen origin, shoving the text hundreds of px sideways.
    let (pw, _, plain) = render("SHEARME");
    let (sw, _, fax) = render("{\\fax0.6}SHEARME");
    assert!(count_covered(&fax) > 0, "\\fax text vanished");
    let plain_w = opaque_bbox_width(&plain, pw);
    let fax_w = opaque_bbox_width(&fax, sw);
    assert!(
        fax_w > plain_w,
        "\\fax should widen the line via slant ({fax_w}px vs {plain_w}px)"
    );
    let dx = bbox_min_x(&plain, pw).abs_diff(bbox_min_x(&fax, sw));
    assert!(
        dx < 150,
        "\\fax shifted the line by {dx}px (origin-shear regression)"
    );
}

#[test]
fn borderstyle3_draws_opaque_box() {
    // Regression: BorderStyle 3 was ignored (rendered as an outline). It must fill
    // an opaque box behind the text in the outline colour (here blue, &H00FF0000).
    let script_text = "[Script Info]\nPlayResX: 1280\nPlayResY: 720\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Box,Arial,64,&H00FFFFFF,&H000000FF,&H00FF0000,&H00FF0000,0,0,0,0,100,100,0,0,3,6,0,5,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:10.00,Box,,0,0,0,,BOX\n";
    let script = Script::parse(script_text).expect("parse");
    let ctx = RenderContext::new(1280, 720);
    let mut renderer = Renderer::new(BackendType::Software, ctx).expect("renderer");
    let frame = renderer.render_frame(&script, 200).expect("render");
    let data = frame.data();

    let blue_box = count_opaque(data, |r, g, b| b > 150 && r < 120 && g < 120);
    assert!(
        blue_box > 3000,
        "BorderStyle 3 should fill a large opaque box, got {blue_box} blue px"
    );
    let white = count_opaque(data, |r, g, b| r > 200 && g > 200 && b > 200);
    assert!(white > 200, "expected white text over the box, got {white}");
}

#[test]
fn org_changes_rotation_pivot() {
    // Regression: \org was dropped by the segmenter and rotation always used the
    // text's own centre. Rotating around an explicit far-off origin displaces the
    // text differently than rotating around its centre.
    let (w, _, centre) = render("{\\frz40}PIVOT");
    let (_, _, orged) = render("{\\org(300,200)\\frz40}PIVOT");
    assert!(count_covered(&orged) > 0, "\\org text vanished");
    let dx = bbox_min_x(&centre, w).abs_diff(bbox_min_x(&orged, w));
    assert!(
        dx > 30,
        "\\org should move the rotation pivot (delta min_x = {dx}px)"
    );
}
