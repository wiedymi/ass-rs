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
use alloc::{sync::Arc, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{sync::Arc, vec::Vec};

use tiny_skia::Transform;

use crate::backends::geometry::{merge_transformed, shadow_delta, stroke_outline};

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

/// Rasterize a layer's fill and outline coverage in local space.
#[cfg(not(feature = "nostd"))]
fn rasterize_run_coverage(
    paths: &[tiny_skia::Path],
    local: Transform,
    outline_width: Option<(f32, f32)>,
) -> CachedCoverage {
    use crate::backends::coverage::CoverageTile;

    // Merge the per-glyph paths once; both the fill and the outline derive from
    // the same merged outline (previously merged twice, redundantly).
    let merged = merge_transformed(paths, local);
    let fill = merged.as_ref().and_then(CoverageTile::rasterize);
    let outline = outline_width
        .zip(merged.as_ref())
        .and_then(|((wx, wy), merged)| {
            // libass grows the glyph outward by the per-axis border (\xbord/\ybord);
            // stroke_outline produces that (uniform for the symmetric case).
            let outlined = stroke_outline(merged, wx, wy)?;
            CoverageTile::rasterize(&outlined)
        });
    CachedCoverage { fill, outline }
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

/// Emit a layer's cached coverage as positioned [`RenderBitmap`]s (shadow, then
/// outline, then fill), applying the current colours at the rounded screen
/// anchor. Producing each is an `Arc` clone of the cached tile, so a
/// geometry-static layer costs almost nothing — this is the libass-style output.
#[cfg(not(feature = "nostd"))]
fn emit_cached(
    cached: &CachedCoverage,
    anchor: (i32, i32),
    colors: super::LayerColors,
    karaoke_sweep: Option<(f32, [u8; 4])>,
) -> Vec<crate::backends::coverage::RenderBitmap> {
    use crate::backends::coverage::RenderBitmap;
    let (anchor_x, anchor_y) = anchor;
    let (outline_color, shadow, fill_color) = colors;
    let mut out = Vec::new();
    let bitmap =
        |tile: &crate::backends::coverage::CoverageTile, x: i32, y: i32, color: [u8; 4]| {
            RenderBitmap::Coverage {
                width: tile.width,
                height: tile.height,
                coverage: tile.data.clone(),
                x,
                y,
                color,
            }
        };
    // Shadow: the fill shape in the shadow colour, displaced. Reuses the fill
    // tile rather than a separately rasterized one.
    if let (Some((color, (dx, dy))), Some((tile, ox, oy))) = (shadow, &cached.fill) {
        out.push(bitmap(tile, anchor_x + ox + dx, anchor_y + oy + dy, color));
    }
    if let (Some(color), Some((tile, ox, oy))) = (outline_color, &cached.outline) {
        out.push(bitmap(tile, anchor_x + ox, anchor_y + oy, color));
    }
    if let Some((tile, ox, oy)) = &cached.fill {
        let (x, y) = (anchor_x + ox, anchor_y + oy);
        match karaoke_sweep {
            // Swept `\K`/`\kf`: not-yet-sung syllables are wholly secondary,
            // fully-sung wholly primary (both reuse the shared tile, no copy);
            // only the one syllable mid-sweep needs a secondary base plus a
            // primary fill cropped to the advancing boundary.
            Some((progress, secondary)) if progress <= 0.0 => {
                out.push(bitmap(tile, x, y, secondary));
            }
            Some((progress, _)) if progress >= 1.0 => {
                out.push(bitmap(tile, x, y, fill_color));
            }
            Some((progress, secondary)) => {
                out.push(bitmap(tile, x, y, secondary));
                let cols = (1.0 + progress * (tile.width as f32 - 2.0))
                    .round()
                    .clamp(0.0, tile.width as f32) as u32;
                if cols > 0 {
                    out.push(RenderBitmap::Coverage {
                        width: cols,
                        height: tile.height,
                        coverage: crop_coverage_columns(tile, cols),
                        x,
                        y,
                        color: fill_color,
                    });
                }
            }
            None => out.push(bitmap(tile, x, y, fill_color)),
        }
    }
    out
}

/// Copy the leftmost `cols` columns of a coverage tile into a new buffer — the
/// "sung" portion of a swept karaoke syllable.
#[cfg(not(feature = "nostd"))]
fn crop_coverage_columns(
    tile: &crate::backends::coverage::CoverageTile,
    cols: u32,
) -> std::sync::Arc<Vec<u8>> {
    let cols = cols.min(tile.width) as usize;
    let width = tile.width as usize;
    let mut data = vec![0u8; cols * tile.height as usize];
    for y in 0..tile.height as usize {
        let src = y * width;
        let dst = y * cols;
        data[dst..dst + cols].copy_from_slice(&tile.data[src..src + cols]);
    }
    std::sync::Arc::new(data)
}

/// Composite cached coverage tiles (shadow, then outline, then fill) onto the
/// premultiplied buffer at the rounded screen anchor — emits the layer's bitmaps
/// and blends them in order.
#[cfg(not(feature = "nostd"))]
fn composite_cached(
    dst: &mut [u8],
    pixmap_w: u32,
    pixmap_h: u32,
    cached: &CachedCoverage,
    anchor: (i32, i32),
    colors: super::LayerColors,
    karaoke_sweep: Option<(f32, [u8; 4])>,
) {
    use crate::backends::coverage::composite_bitmap;
    let bitmaps = emit_cached(cached, anchor, colors, karaoke_sweep);
    EMIT_SINK.with(|sink| {
        let mut sink = sink.borrow_mut();
        if let Some(sink) = sink.as_mut() {
            // Collecting the bitmap list: hand the layer's bitmaps over instead of
            // blending them into a frame buffer.
            sink.extend(bitmaps);
        } else {
            for bitmap in &bitmaps {
                composite_bitmap(dst, pixmap_w, pixmap_h, bitmap);
            }
        }
    });
}

impl super::SoftwareBackend {
    /// Render a text layer from A8 coverage tiles (shadow, outline, fill) and
    /// return `true`, or `false` when the layer uses an effect this fast path
    /// does not cover (blur, edge blur, karaoke, clip, opaque box, underline,
    /// strikethrough) so the caller falls back to the full vector path.
    ///
    /// `base_transform` already bakes rotation/scale/shear, so the rasterized
    /// coverage depends only on the layer geometry — which is what makes the
    /// per-frame work for unchanged geometry a cheap composite rather than a
    /// re-rasterization.
    /// Composite a text layer from cached coverage tiles WITHOUT shaping or
    /// building glyph paths. Returns `true` on a cache hit; `false` if the layer
    /// is ineligible or its geometry is not cached yet (the caller then builds
    /// the paths and calls [`Self::rasterize_coverage_miss`]). This is the path
    /// that makes geometry-static animated layers nearly free: the per-frame work
    /// is just the composite.
    #[cfg(not(feature = "nostd"))]
    pub(super) fn coverage_hit(
        &mut self,
        data: &crate::pipeline::TextData,
        base_transform: Transform,
        baseline_y: f32,
    ) -> bool {
        let Some((key, outline, shadow, local, fill_color, karaoke_sweep)) =
            coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };
        let shadow_paint = shadow.map(|(c, sx, sy)| (c, shadow_delta(local, sx, sy)));
        let anchor_x = data.x.round() as i32;
        let anchor_y = baseline_y.round() as i32;
        let pixmap_w = self.pixmap.width();
        let pixmap_h = self.pixmap.height();
        let dst = self.pixmap.data_mut();
        RUN_COVERAGE.with(|cache| {
            let map = cache.borrow();
            let Some(cached) = map.get(&key) else {
                return false;
            };
            composite_cached(
                dst,
                pixmap_w,
                pixmap_h,
                cached,
                (anchor_x, anchor_y),
                (outline.map(|(c, _, _)| c), shadow_paint, fill_color),
                karaoke_sweep,
            );
            true
        })
    }

    /// Rasterize a text layer's coverage from already-built `paths`, cache it,
    /// and composite it. Returns `true` if the coverage path handled the layer,
    /// or `false` if it is ineligible and must take the full vector path.
    #[cfg(not(feature = "nostd"))]
    pub(super) fn rasterize_coverage_miss(
        &mut self,
        data: &crate::pipeline::TextData,
        paths: &[tiny_skia::Path],
        base_transform: Transform,
        baseline_y: f32,
    ) -> bool {
        let Some((key, outline, shadow, local, fill_color, karaoke_sweep)) =
            coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };

        RUN_COVERAGE.with(|cache| {
            if !cache.borrow().contains_key(&key) {
                let cached =
                    rasterize_run_coverage(paths, local, outline.map(|(_, wx, wy)| (wx, wy)));
                let mut map = cache.borrow_mut();
                // Bound memory: continuously geometry-animated layers produce a
                // fresh key every frame, so drop the cache when it grows large
                // rather than leak; geometry-static layers re-cache once after.
                if map.len() >= 256 {
                    map.clear();
                }
                map.insert(key.clone(), cached);
            }
        });

        let shadow_paint = shadow.map(|(c, sx, sy)| (c, shadow_delta(local, sx, sy)));
        let anchor_x = data.x.round() as i32;
        let anchor_y = baseline_y.round() as i32;
        let pixmap_w = self.pixmap.width();
        let pixmap_h = self.pixmap.height();
        let dst = self.pixmap.data_mut();
        RUN_COVERAGE.with(|cache| {
            let map = cache.borrow();
            if let Some(cached) = map.get(&key) {
                composite_cached(
                    dst,
                    pixmap_w,
                    pixmap_h,
                    cached,
                    (anchor_x, anchor_y),
                    (outline.map(|(c, _, _)| c), shadow_paint, fill_color),
                    karaoke_sweep,
                );
            }
        });
        true
    }

    /// Composite a blurred text layer directly from the cached blurred bitmap
    /// (see [`BlurTileKey`]), skipping the font lookup and glyph-path build.
    /// Returns `true` on a hit. Eligible only when the cached bitmap is the
    /// layer's *entire* output: `\blur` is present and every effect is one the
    /// bitmap captures (outline/shadow) or that the blur branch ignores
    /// (bold/italic/rotation/scale/shear). A clip, opaque box, underline,
    /// strikethrough, edge blur or karaoke draws beyond the tile, so those layers
    /// fall through to the full path.
    #[cfg(not(feature = "nostd"))]
    pub(super) fn blur_tile_hit(
        &mut self,
        data: &crate::pipeline::TextData,
        bold: bool,
        italic: bool,
        baseline_y: f32,
        ascent: f32,
    ) -> bool {
        use crate::pipeline::TextEffect;

        let eligible = data.effects.iter().all(|e| {
            matches!(
                e,
                TextEffect::Blur { .. }
                    | TextEffect::Outline { .. }
                    | TextEffect::Shadow { .. }
                    | TextEffect::Bold
                    | TextEffect::Italic
                    | TextEffect::Rotation { .. }
                    | TextEffect::Scale { .. }
                    | TextEffect::Shear { .. }
            )
        });
        if !eligible {
            return false;
        }

        // Extract blur/outline/shadow with the same first-match semantics the blur
        // branch uses, so the key is identical to the one it stored.
        let Some(radius) = data.effects.iter().find_map(|e| match e {
            TextEffect::Blur { radius } => Some(*radius),
            _ => None,
        }) else {
            return false;
        };
        let outline_info = data.effects.iter().find_map(|e| match e {
            TextEffect::Outline {
                color,
                width_x,
                width_y,
            } => Some((*color, *width_x, *width_y)),
            _ => None,
        });
        let shadow_info = data.effects.iter().find_map(|e| match e {
            TextEffect::Shadow {
                color,
                x_offset,
                y_offset,
            } => Some((*color, *x_offset, *y_offset)),
            _ => None,
        });

        let key = BlurTileKey {
            text: data.text.clone(),
            font: data.font_family.clone(),
            size: data.font_size.to_bits(),
            spacing: data.spacing.to_bits(),
            bold,
            italic,
            blur: radius.to_bits(),
            fill: data.color,
            outline: outline_info.map(|(c, wx, wy)| (wx.to_bits(), wy.to_bits(), c)),
            shadow: shadow_info.map(|(c, x, y)| (c, x.to_bits(), y.to_bits())),
        };
        let Some(tile) = BLUR_TILES.with(|c| c.borrow().get(&key).cloned()) else {
            return false;
        };

        let blur_size = (radius * 3.0).ceil();
        let x = data.x - blur_size;
        // The tile's baseline sits at `blur_size + ascent` from its top (see the
        // blur branch's temp_transform), so the tile origin lands here.
        let y = baseline_y - blur_size - ascent;

        // Bitmap-list mode: the cached tile IS this layer's entire output (the
        // eligibility check guarantees nothing else is drawn), so emit it directly
        // as a positioned bitmap — skipping the full-frame scratch render + crop +
        // clear the generic vector path would do. `composite_rgba` places it at
        // integer (x, y); the sharp `\blur` path's nearest-filter `draw_pixmap`
        // lands at the same rounded position, so this stays frame-equivalent.
        let emitted = EMIT_SINK.with(|sink| {
            if let Some(list) = sink.borrow_mut().as_mut() {
                list.push(crate::backends::coverage::RenderBitmap::Rgba {
                    width: tile.width,
                    height: tile.height,
                    pixels: tile.data.clone(),
                    x: x.round() as i32,
                    y: y.round() as i32,
                });
                true
            } else {
                false
            }
        });
        if emitted {
            return true;
        }

        // Composite mode: draw the tile into the frame at the same fractional
        // origin and SourceOver blend the blur branch uses on a hit.
        if let Some(pixref) =
            tiny_skia::PixmapRef::from_bytes(tile.data.as_slice(), tile.width, tile.height)
        {
            let paint = tiny_skia::PixmapPaint {
                blend_mode: tiny_skia::BlendMode::SourceOver,
                ..Default::default()
            };
            self.pixmap
                .draw_pixmap(0, 0, pixref, &paint, Transform::from_translate(x, y), None);
        }
        true
    }
}
