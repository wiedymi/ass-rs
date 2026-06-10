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

#[test]
fn bord_and_shad_were_parsed_and_applied() {
    // Regression: the segmenter dropped \bord and \shad entirely. The Default
    // style has Outline=0 / Shadow=0, so these tags must visibly add coverage.
    let (_, _, plain) = render("BORD");
    let (_, _, bord) = render("{\\bord10}BORD");
    let (_, _, shad) = render("{\\shad14}BORD");
    let plain_cov = count_covered(&plain);
    assert!(plain_cov > 0, "plain text should render");
    assert!(
        count_covered(&bord) >= plain_cov * 3 / 2,
        "\\bord10 should add a thick outline ({} vs {plain_cov})",
        count_covered(&bord)
    );
    assert!(
        count_covered(&shad) > plain_cov,
        "\\shad14 should add a shadow ({} vs {plain_cov})",
        count_covered(&shad)
    );
}

#[test]
fn reset_tag_clears_inline_overrides() {
    // Regression: \r was dropped by the segmenter. \r must reset inline overrides
    // back to the style: red set before \r reverts to the white style colour.
    let (_, _, data) = render("{\\c&H0000FF&\\r}RESET");
    let red = count_opaque(&data, |r, g, b| r > 150 && g < 90 && b < 90);
    let white = count_opaque(&data, |r, g, b| r > 200 && g > 200 && b > 200);
    assert!(
        white > 200,
        "after \\r text should be the white style colour ({white})"
    );
    assert!(red < 50, "after \\r there should be ~no red left ({red})");
}

#[test]
fn complex_fade_holds_visible_then_fades_out() {
    // \fade(a1,a2,a3,t1,t2,t3,t4): invisible before t1, fade in t1..t2, fully
    // visible t2..t3, fade out t3..t4, invisible after. (The old code lerped
    // a1->a3 = 255->255, leaving the text permanently invisible.) Times in ms;
    // the event spans 0..10000ms.
    let fade = "{\\fade(255,0,255,0,1000,4000,5000)}FADE";
    let visible = |d: &[u8]| count_opaque(d, |_, _, _| true);

    // Hold region t2..t3 (t = 2000ms = 200cs): fully visible.
    let (_, _, mid) = render_at(200, fade);
    assert!(
        visible(&mid) > 500,
        "complex fade should be visible during the hold ({})",
        visible(&mid)
    );
    // After t4 (t = 6000ms = 600cs): faded out / invisible.
    let (_, _, after) = render_at(600, fade);
    assert!(
        visible(&after) < 50,
        "complex fade should be invisible after t4 ({})",
        visible(&after)
    );
}

#[test]
fn kf_sweep_shows_primary_and_secondary_together() {
    // \kf sweeps left-to-right: mid-syllable the sung (left) part is primary
    // (white) and the unsung (right) part is secondary (red) AT THE SAME TIME —
    // not one uniform blended colour. \kf100 = a 1s syllable; t=50cs is 50% in.
    let (_, _, mid) = render_at(50, "{\\kf100}KARAOKE");
    let white = count_opaque(&mid, |r, g, b| r > 200 && g > 200 && b > 200);
    let red = count_opaque(&mid, |r, g, b| r > 150 && g < 90 && b < 90);
    assert!(white > 100, "swept (sung) part should be white ({white})");
    assert!(red > 100, "un-swept part should be secondary red ({red})");
}

#[test]
fn blur_softens_outline_and_fill_together() {
    // Outlined style (Outline=4) + strong \blur: the outline must blur with the
    // fill, so almost no fully-opaque pixels remain. The old code blurred only the
    // fill and kept a sharp (fully opaque) outline ring.
    let head = "[Script Info]\nPlayResX: 1280\nPlayResY: 720\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Outlined,Arial,64,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,4,0,5,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";
    let render_outlined = |text: &str| {
        let s = format!("{head}Dialogue: 0,0:00:00.00,0:00:10.00,Outlined,,0,0,0,,{text}\n");
        let script = Script::parse(&s).expect("parse");
        let ctx = RenderContext::new(1280, 720);
        let mut r = Renderer::new(BackendType::Software, ctx).expect("renderer");
        r.render_frame(&script, 200)
            .expect("render")
            .data()
            .to_vec()
    };
    let solid = |d: &[u8]| d.chunks_exact(4).filter(|p| p[3] >= 250).count() as u64;

    let sharp = render_outlined("OUTLINE");
    let blurred = render_outlined("{\\blur20}OUTLINE");
    assert!(
        solid(&sharp) > 500,
        "sharp outlined text should have many opaque pixels ({})",
        solid(&sharp)
    );
    assert!(
        solid(&blurred) < solid(&sharp) / 5,
        "\\blur should soften the outline too ({} blurred vs {} sharp)",
        solid(&blurred),
        solid(&sharp)
    );
}

#[test]
fn blur_softens_shadow_and_fill_together() {
    // Shadowed style (Shadow=8, no outline) + strong \blur: the offset shadow
    // must blur with the fill, leaving almost no fully-opaque pixels. The old
    // code blurred only the fill and kept the sharp offset shadow block opaque.
    let head = "[Script Info]\nPlayResX: 1280\nPlayResY: 720\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Shadowed,Arial,64,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,8,5,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";
    let render_shadowed = |text: &str| {
        let s = format!("{head}Dialogue: 0,0:00:00.00,0:00:10.00,Shadowed,,0,0,0,,{text}\n");
        let script = Script::parse(&s).expect("parse");
        let ctx = RenderContext::new(1280, 720);
        let mut r = Renderer::new(BackendType::Software, ctx).expect("renderer");
        r.render_frame(&script, 200)
            .expect("render")
            .data()
            .to_vec()
    };
    let solid = |d: &[u8]| d.chunks_exact(4).filter(|p| p[3] >= 250).count() as u64;

    let sharp = render_shadowed("SHADOW");
    let blurred = render_shadowed("{\\blur20}SHADOW");
    assert!(
        solid(&sharp) > 500,
        "sharp shadowed text should have many opaque pixels ({})",
        solid(&sharp)
    );
    assert!(
        solid(&blurred) < solid(&sharp) / 5,
        "\\blur should soften the shadow too ({} blurred vs {} sharp)",
        solid(&blurred),
        solid(&sharp)
    );
}

#[test]
fn multiline_lines_spaced_by_font_size() {
    // Regression: the `\N` line advance must equal the nominal font size (libass's
    // baseline-to-baseline spacing), not the dpi-scaled glyph size (~0.9x). The old
    // code advanced by the scaled glyph size, packing lines ~11% too tight. The
    // Default style font is 64 and PlayResY == frame height, so spacing should be ~64.
    let (width, height, data) = render("Line one\\NLine two");
    let mut band_tops = Vec::new();
    let mut in_band = false;
    for y in 0..height {
        let lit = (0..width)
            .filter(|x| data[(y * width + x) * 4 + 3] >= 128)
            .count();
        let on = lit >= 3;
        if on && !in_band {
            band_tops.push(y);
            in_band = true;
        } else if !on && in_band {
            in_band = false;
        }
    }
    assert_eq!(
        band_tops.len(),
        2,
        "expected two text lines, got {} bands",
        band_tops.len()
    );
    let spacing = band_tops[1] - band_tops[0];
    assert!(
        (58..=70).contains(&spacing),
        "multi-line advance should be ~font size 64, got {spacing}px"
    );
}
