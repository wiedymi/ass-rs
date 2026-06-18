//! Positioned-bitmap emit / blend of cached coverage tiles, plus the
//! blurred-text bitmap cache hit, for the software text layer.

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

use tiny_skia::Transform;

/// Emit a layer's cached coverage as positioned [`RenderBitmap`]s (shadow, then
/// outline, then fill), applying the current colours at the rounded screen
/// anchor. Producing each is an `Arc` clone of the cached tile, so a
/// geometry-static layer costs almost nothing — this is the libass-style output.
#[cfg(not(feature = "nostd"))]
fn emit_cached(
    cached: &super::CachedCoverage,
    anchor: (i32, i32),
    colors: super::super::LayerColors,
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
pub(super) fn composite_cached(
    dst: &mut [u8],
    pixmap_w: u32,
    pixmap_h: u32,
    cached: &super::CachedCoverage,
    anchor: (i32, i32),
    colors: super::super::LayerColors,
    karaoke_sweep: Option<(f32, [u8; 4])>,
) {
    use crate::backends::coverage::composite_bitmap;
    let bitmaps = emit_cached(cached, anchor, colors, karaoke_sweep);
    super::EMIT_SINK.with(|sink| {
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

impl super::super::SoftwareBackend {
    /// Composite a blurred text layer directly from the cached blurred bitmap
    /// (see [`BlurTileKey`]), skipping the font lookup and glyph-path build.
    /// Returns `true` on a hit. Eligible only when the cached bitmap is the
    /// layer's *entire* output: `\blur` is present and every effect is one the
    /// bitmap captures (outline/shadow) or that the blur branch ignores
    /// (bold/italic/rotation/scale/shear). A clip, opaque box, underline,
    /// strikethrough, edge blur or karaoke draws beyond the tile, so those layers
    /// fall through to the full path.
    #[cfg(not(feature = "nostd"))]
    pub(in crate::backends::software) fn blur_tile_hit(
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

        let key = super::BlurTileKey {
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
        let Some(tile) = super::BLUR_TILES.with(|c| c.borrow().get(&key).cloned()) else {
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
        let emitted = super::EMIT_SINK.with(|sink| {
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
