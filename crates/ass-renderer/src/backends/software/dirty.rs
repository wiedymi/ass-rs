//! Dirty-region tracking and scratch-crop helpers for the software backend.
//!
//! When `render_to_bitmaps` collects a positioned bitmap list, vector-path
//! layers render into a reused scratch pixmap that must then be cropped to its
//! drawn pixels and cleared again for the next layer. These helpers record a
//! generous per-layer dirty bbox (via the `DIRTY_BBOX` thread-local) so the crop
//! scans only the touched region of a 4K frame rather than the whole thing, crop
//! the scratch into an RGBA bitmap, and zero the touched rectangle afterwards.

#[cfg(feature = "nostd")]
use alloc::{sync::Arc, vec};
#[cfg(not(feature = "nostd"))]
use std::sync::Arc;

use tiny_skia::{Pixmap, Transform};

use super::cache::DIRTY_BBOX;
use crate::backends::geometry::merge_transformed;

/// A generous screen-space bbox covering a text layer's vector-path output
/// (glyphs plus an outline/shadow/blur margin), or `None` if it has no geometry.
#[cfg(not(feature = "nostd"))]
pub(super) fn text_vector_dirty_bbox(
    data: &crate::pipeline::TextData,
    paths: &[tiny_skia::Path],
    base_transform: Transform,
) -> Option<(i32, i32, i32, i32)> {
    use crate::pipeline::TextEffect;
    let bounds = merge_transformed(paths, base_transform)?.bounds();
    let (mut outline, mut shadow, mut blur) = (0.0_f32, 0.0_f32, 0.0_f32);
    for effect in &data.effects {
        match effect {
            TextEffect::Outline {
                width_x, width_y, ..
            } => outline = outline.max(width_x.max(*width_y)),
            TextEffect::Shadow {
                x_offset, y_offset, ..
            } => shadow = shadow.max(x_offset.abs()).max(y_offset.abs()),
            TextEffect::Blur { radius } | TextEffect::EdgeBlur { radius } => {
                blur = blur.max(*radius)
            }
            _ => {}
        }
    }
    // Generous: box blur of radius r spreads ~r each side; ×4 leaves head-room.
    let margin = 4.0 + outline * 2.0 + shadow + blur * 4.0;
    Some((
        (bounds.left() - margin).floor() as i32,
        (bounds.top() - margin).floor() as i32,
        (bounds.right() + margin).ceil() as i32,
        (bounds.bottom() + margin).ceil() as i32,
    ))
}

/// Record a generous dirty bbox for the current vector layer (used to bound the
/// scratch crop). No-op outside `render_to_bitmaps`.
#[cfg(not(feature = "nostd"))]
pub(super) fn note_dirty_bbox(bbox: (i32, i32, i32, i32)) {
    DIRTY_BBOX.with(|b| {
        let mut slot = b.borrow_mut();
        *slot = Some(match *slot {
            None => bbox,
            Some((x0, y0, x1, y1)) => (
                x0.min(bbox.0),
                y0.min(bbox.1),
                x1.max(bbox.2),
                y1.max(bbox.3),
            ),
        });
    });
}

/// Crop a premultiplied-RGBA pixmap to the bounding box of its non-transparent
/// pixels and return it as an `Rgba` [`RenderBitmap`], or `None` if fully empty.
#[cfg(not(feature = "nostd"))]
/// Zero a rectangular region `(x, y, width, height)` of `pixmap` (clamped to its
/// bounds). Used to restore the scratch pixmap to transparent after a vector
/// layer is cropped, clearing only the touched rectangle rather than the frame.
#[cfg(not(feature = "nostd"))]
pub(super) fn clear_region(pixmap: &mut Pixmap, region: (i32, i32, u32, u32)) {
    let (rx, ry, rw, rh) = region;
    let w = pixmap.width() as i32;
    let h = pixmap.height() as i32;
    let x0 = rx.max(0);
    let y0 = ry.max(0);
    let x1 = (rx + rw as i32).min(w);
    let y1 = (ry + rh as i32).min(h);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let row_bytes = (x1 - x0) as usize * 4;
    let data = pixmap.data_mut();
    for y in y0..y1 {
        let start = (y * w + x0) as usize * 4;
        data[start..start + row_bytes].fill(0);
    }
}

#[cfg(not(feature = "nostd"))]
pub(super) fn crop_pixmap(
    pixmap: &Pixmap,
    hint: Option<(i32, i32, i32, i32)>,
) -> Option<crate::backends::coverage::RenderBitmap> {
    use crate::backends::coverage::RenderBitmap;
    let w = pixmap.width() as i32;
    let h = pixmap.height() as i32;
    let data = pixmap.data();
    // Only scan the layer's (generous) dirty region — scanning the whole 4K frame
    // per vector layer is memory-bound and dominates otherwise.
    let (scan_x0, scan_y0, scan_x1, scan_y1) = match hint {
        Some((x0, y0, x1, y1)) => (x0.max(0), y0.max(0), (x1 + 1).min(w), (y1 + 1).min(h)),
        None => (0, 0, w, h),
    };
    if scan_x1 <= scan_x0 || scan_y1 <= scan_y0 {
        return None;
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (scan_x1, scan_y1, -1_i32, -1_i32);
    for y in scan_y0..scan_y1 {
        let row = (y * w) as usize * 4;
        for x in scan_x0..scan_x1 {
            if data[row + x as usize * 4 + 3] != 0 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }
    if max_x < min_x {
        return None;
    }
    let bw = (max_x - min_x + 1) as u32;
    let bh = (max_y - min_y + 1) as u32;
    let row_bytes = bw as usize * 4;
    let mut pixels = vec![0u8; row_bytes * bh as usize];
    for ty in 0..bh as i32 {
        let src = (((min_y + ty) * w + min_x) as usize) * 4;
        let dst = ty as usize * row_bytes;
        pixels[dst..dst + row_bytes].copy_from_slice(&data[src..src + row_bytes]);
    }
    Some(RenderBitmap::Rgba {
        width: bw,
        height: bh,
        pixels: Arc::new(pixels),
        x: min_x,
        y: min_y,
    })
}
