//! A8 coverage tiles for the software backend.
//!
//! A text layer's fill and outline are rasterized to 8-bit coverage once, then
//! composited with the current colour/alpha at the current screen position. This
//! separates the expensive vector rasterization (which depends only on the
//! glyph *geometry*) from the cheap per-frame compositing (which carries colour,
//! alpha and position). Layers whose geometry is unchanged between frames — the
//! common animated case (`\move`, `\fad`, colour `\t`, karaoke) — can then reuse
//! a cached tile and only re-composite, the way libass does, instead of
//! re-rasterizing every frame.

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};

use tiny_skia::{Path, PathSegment};

use crate::backends::raster::Rasterizer;

/// An 8-bit coverage tile: `width * height` alpha samples, row-major.
#[derive(Clone)]
pub struct CoverageTile {
    /// Tile width in pixels.
    pub width: u32,
    /// Tile height in pixels.
    pub height: u32,
    /// `width * height` coverage bytes (0 = empty, 255 = fully covered).
    pub data: Vec<u8>,
}

impl CoverageTile {
    /// Rasterize a screen-space `path` to a coverage tile.
    ///
    /// Returns the tile plus the integer `(x, y)` at which it must be
    /// composited. The path's sub-pixel position is baked into the coverage, so
    /// compositing at the returned integer offset reproduces the anti-aliasing
    /// of a direct fill at the original position. Returns `None` for an empty or
    /// unrasterizable path.
    #[must_use]
    pub fn rasterize(path: &Path) -> Option<(Self, i32, i32)> {
        let bounds = path.bounds();
        // Pad by one pixel so anti-aliased edges are not clipped at the border.
        let min_x = bounds.left().floor() as i32 - 1;
        let min_y = bounds.top().floor() as i32 - 1;
        let max_x = bounds.right().ceil() as i32 + 1;
        let max_y = bounds.bottom().ceil() as i32 + 1;
        let width = u32::try_from((max_x - min_x).max(1)).ok()?;
        let height = u32::try_from((max_y - min_y).max(1)).ok()?;

        // Feed the path's contours to the in-house scanline rasterizer, in tile
        // coordinates (origin at the padded bbox corner). Each contour is closed
        // (back to its start) so non-zero-winding coverage is correct.
        let ox = min_x as f32;
        let oy = min_y as f32;
        let mut raster = Rasterizer::new(width as usize, height as usize);
        let mut start = (0.0_f32, 0.0_f32);
        let mut cur = (0.0_f32, 0.0_f32);
        let mut open = false;
        for segment in path.segments() {
            match segment {
                PathSegment::MoveTo(p) => {
                    if open {
                        raster.line(cur.0, cur.1, start.0, start.1);
                    }
                    start = (p.x - ox, p.y - oy);
                    cur = start;
                    open = true;
                }
                PathSegment::LineTo(p) => {
                    let next = (p.x - ox, p.y - oy);
                    raster.line(cur.0, cur.1, next.0, next.1);
                    cur = next;
                }
                PathSegment::QuadTo(c, p) => {
                    let cc = (c.x - ox, c.y - oy);
                    let next = (p.x - ox, p.y - oy);
                    raster.quad(cur.0, cur.1, cc.0, cc.1, next.0, next.1);
                    cur = next;
                }
                PathSegment::CubicTo(c1, c2, p) => {
                    let a = (c1.x - ox, c1.y - oy);
                    let b = (c2.x - ox, c2.y - oy);
                    let next = (p.x - ox, p.y - oy);
                    raster.cubic(cur.0, cur.1, a.0, a.1, b.0, b.1, next.0, next.1);
                    cur = next;
                }
                PathSegment::Close => {
                    raster.line(cur.0, cur.1, start.0, start.1);
                    cur = start;
                }
            }
        }
        if open {
            raster.line(cur.0, cur.1, start.0, start.1);
        }

        Some((
            Self {
                width,
                height,
                data: raster.finish(),
            },
            min_x,
            min_y,
        ))
    }
}

/// Rounded fixed-point multiply-divide by 255 (`a * b / 255`).
#[inline]
fn mul255(a: u32, b: u32) -> u32 {
    (a * b + 127) / 255
}

/// Composite an A8 coverage tile onto a premultiplied-RGBA8 buffer at `(x, y)`
/// using a straight (non-premultiplied) `color`, with source-over blending.
///
/// `dst` is `dst_w * dst_h * 4` bytes in tiny-skia's premultiplied RGBA layout.
/// The paint is premultiplied once, then scaled by each pixel's coverage — the
/// same result as filling the path directly with a solid-colour paint.
pub fn composite(
    dst: &mut [u8],
    dst_w: u32,
    dst_h: u32,
    tile: &CoverageTile,
    x: i32,
    y: i32,
    color: [u8; 4],
) {
    let pa = u32::from(color[3]);
    if pa == 0 {
        return;
    }
    let pr = mul255(u32::from(color[0]), pa);
    let pg = mul255(u32::from(color[1]), pa);
    let pb = mul255(u32::from(color[2]), pa);

    let dst_w_i = dst_w as i32;
    let dst_h_i = dst_h as i32;
    for ty in 0..tile.height {
        let py = y + ty as i32;
        if py < 0 || py >= dst_h_i {
            continue;
        }
        let tile_row = (ty * tile.width) as usize;
        let dst_row = (py as u32 * dst_w) as usize * 4;
        for tx in 0..tile.width {
            let cov = u32::from(tile.data[tile_row + tx as usize]);
            if cov == 0 {
                continue;
            }
            let px = x + tx as i32;
            if px < 0 || px >= dst_w_i {
                continue;
            }
            let idx = dst_row + px as usize * 4;
            let sr = mul255(pr, cov);
            let sg = mul255(pg, cov);
            let sb = mul255(pb, cov);
            let sa = mul255(pa, cov);
            let inv = 255 - sa;
            dst[idx] = (sr + mul255(u32::from(dst[idx]), inv)) as u8;
            dst[idx + 1] = (sg + mul255(u32::from(dst[idx + 1]), inv)) as u8;
            dst[idx + 2] = (sb + mul255(u32::from(dst[idx + 2]), inv)) as u8;
            dst[idx + 3] = (sa + mul255(u32::from(dst[idx + 3]), inv)) as u8;
        }
    }
}
