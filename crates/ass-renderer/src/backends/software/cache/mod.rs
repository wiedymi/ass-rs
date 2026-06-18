//! Coverage- and blur-tile caches for the software backend.
//!
//! A text layer's rasterized geometry (glyph coverage, outline coverage) and its
//! pre-composited blurred bitmap are position- and (for coverage) colour-
//! independent, so they are cached per-thread and reused across frames. This is
//! what makes geometry-static animated layers (`\move`, `\fad`, colour `\t`,
//! karaoke colour) nearly free: the per-frame work collapses to a composite. The
//! `EMIT_SINK`/`DIRTY_BBOX` thread-locals carry the bitmap-list render mode so the
//! cached tiles can be emitted as positioned bitmaps instead of blended.

#[cfg(feature = "nostd")]
use alloc::{sync::Arc, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{sync::Arc, vec::Vec};

use tiny_skia::Transform;

mod blur_tile;
mod coverage;

/// Cache key for a text layer's coverage tiles: everything that determines the
/// rasterized *geometry* (glyph shapes + the position-independent transform +
/// outline width + shadow offset), but NOT colour, alpha or screen position —
/// those are applied when the cached tiles are composited, so a layer whose
/// geometry is unchanged between frames (e.g. `\move`, `\fad`, colour `\t`,
/// karaoke colour) reuses its tiles instead of re-rasterizing.
#[cfg(not(feature = "nostd"))]
#[derive(Clone, PartialEq, Eq, Hash)]
struct RunCoverageKey {
    text: String,
    font: String,
    size: u32,
    spacing: u32,
    bold: bool,
    italic: bool,
    outline: Option<(u32, u32)>,
    shadow: Option<(u32, u32)>,
    transform: [u32; 6],
}

/// Cache key for a blurred text layer's pre-composited bitmap. The `\blur` branch
/// rasterizes shadow + outline + fill into a temp pixmap and box-blurs it; that
/// result depends only on the glyph outlines, the blur radius and the baked
/// colours — NOT screen position (applied at composite). So the recurring letters
/// of blurred credit text reuse one rasterized+blurred bitmap, as libass does.
/// Colours are part of the key because they are baked into the bitmap.
#[cfg(not(feature = "nostd"))]
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct BlurTileKey {
    pub(super) text: String,
    pub(super) font: String,
    pub(super) size: u32,
    pub(super) spacing: u32,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) blur: u32,
    pub(super) fill: [u8; 4],
    pub(super) outline: Option<(u32, u32, [u8; 4])>,
    pub(super) shadow: Option<([u8; 4], u32, u32)>,
}

/// A cached blurred-text bitmap: premultiplied RGBA. The pixels live in an `Arc`
/// so a cache hit can emit them as a [`RenderBitmap::Rgba`] with a cheap clone.
#[cfg(not(feature = "nostd"))]
pub(super) struct BlurTile {
    pub(super) data: Arc<Vec<u8>>,
    pub(super) width: u32,
    pub(super) height: u32,
}

/// Rasterized coverage tiles for one text layer, in position-independent local
/// space. Each entry is the A8 tile plus its `(x, y)` offset from the layer
/// anchor, so compositing happens at `anchor + offset`. The shadow is not stored
/// separately: it is the fill shape, so it reuses the fill tile composited at an
/// offset (see [`composite_cached`]).
#[cfg(not(feature = "nostd"))]
struct CachedCoverage {
    fill: Option<(crate::backends::coverage::CoverageTile, i32, i32)>,
    outline: Option<(crate::backends::coverage::CoverageTile, i32, i32)>,
}

// Per-thread cache of rasterized coverage tiles, shared by the hit and miss
// paths and persistent across frames.
#[cfg(not(feature = "nostd"))]
std::thread_local! {
    static RUN_COVERAGE: std::cell::RefCell<std::collections::HashMap<RunCoverageKey, CachedCoverage>> =
        std::cell::RefCell::new(std::collections::HashMap::new());

    /// Per-thread cache of blurred text bitmaps (see [`BlurTileKey`]), persistent
    /// across frames so recurring blurred glyphs are rasterized+blurred once.
    pub(super) static BLUR_TILES: std::cell::RefCell<std::collections::HashMap<BlurTileKey, std::sync::Arc<BlurTile>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());

    /// When `Some`, coverage-path layers append their bitmaps here instead of
    /// compositing — this is how `render_to_bitmaps` collects the libass-style
    /// output while reusing the normal layer-rendering code unchanged.
    pub(super) static EMIT_SINK: std::cell::RefCell<Option<Vec<crate::backends::coverage::RenderBitmap>>> =
        const { std::cell::RefCell::new(None) };

    /// A generous screen-space `(min_x, min_y, max_x, max_y)` of a vector layer's
    /// drawn pixels, set during `render_to_bitmaps` so the scratch crop scans only
    /// that region instead of the whole (4K) frame.
    pub(super) static DIRTY_BBOX: std::cell::RefCell<Option<(i32, i32, i32, i32)>> =
        const { std::cell::RefCell::new(None) };
}

/// Build the coverage cache key (and extract the outline/shadow paints and the
/// position-independent transform) for a text layer, or `None` if the layer uses
/// an effect the coverage path does not handle (blur, edge blur, karaoke, clip,
/// opaque box, underline, strikethrough) and must take the full vector path.
#[cfg(not(feature = "nostd"))]
#[allow(clippy::type_complexity)]
fn coverage_key(
    data: &crate::pipeline::TextData,
    base_transform: Transform,
    baseline_y: f32,
) -> Option<(
    RunCoverageKey,
    Option<([u8; 4], f32, f32)>,
    Option<([u8; 4], f32, f32)>,
    Transform,
    [u8; 4],
    Option<(f32, [u8; 4])>,
)> {
    use crate::pipeline::TextEffect;

    let mut outline: Option<([u8; 4], f32, f32)> = None;
    let mut shadow: Option<([u8; 4], f32, f32)> = None;
    let mut bold = false;
    let mut italic = false;
    // The fill colour is normally the primary colour. Karaoke leaves the glyph
    // GEOMETRY unchanged, so it stays cacheable: binary `\k` just flips the whole
    // syllable's fill colour (primary once sung, else secondary), and swept
    // `\K`/`\kf` is `karaoke_sweep = (progress, secondary)` — applied at composite
    // as a secondary base plus a primary fill cropped to the sweep boundary.
    let mut fill_color = data.color;
    let mut karaoke_sweep: Option<(f32, [u8; 4])> = None;
    for effect in &data.effects {
        match effect {
            TextEffect::Outline {
                color,
                width_x,
                width_y,
            } => outline = Some((*color, *width_x, *width_y)),
            TextEffect::Shadow {
                color,
                x_offset,
                y_offset,
            } => shadow = Some((*color, *x_offset, *y_offset)),
            TextEffect::Bold => bold = true,
            TextEffect::Italic => italic = true,
            TextEffect::Rotation { .. } | TextEffect::Scale { .. } | TextEffect::Shear { .. } => {}
            TextEffect::Karaoke {
                progress,
                style,
                secondary,
            } => {
                if *style == 0 {
                    fill_color = if *progress > 0.0 {
                        data.color
                    } else {
                        *secondary
                    };
                } else {
                    karaoke_sweep = Some((*progress, *secondary));
                }
            }
            _ => return None,
        }
    }

    // Strip the screen translation so the coverage depends only on geometry and
    // can be reused at any screen position / colour.
    let local = base_transform.post_translate(-data.x, -baseline_y);
    let key = RunCoverageKey {
        text: data.text.clone(),
        font: data.font_family.clone(),
        size: data.font_size.to_bits(),
        spacing: data.spacing.to_bits(),
        bold,
        italic,
        outline: outline.map(|(_, wx, wy)| (wx.to_bits(), wy.to_bits())),
        shadow: shadow.map(|(_, x, y)| (x.to_bits(), y.to_bits())),
        transform: [
            local.sx.to_bits(),
            local.kx.to_bits(),
            local.ky.to_bits(),
            local.sy.to_bits(),
            local.tx.to_bits(),
            local.ty.to_bits(),
        ],
    };
    Some((key, outline, shadow, local, fill_color, karaoke_sweep))
}
