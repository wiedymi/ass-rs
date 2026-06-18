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
use alloc::{sync::Arc, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::sync::Arc;

use tiny_skia::{Path, PathSegment};

use crate::backends::raster::Rasterizer;

mod blend;
mod composite;

pub use composite::{composite, composite_bitmap};

/// An 8-bit coverage tile: `width * height` alpha samples, row-major.
///
/// The coverage is `Arc`-shared so a cached tile can be emitted as a
/// [`RenderBitmap`] (or composited) without copying the buffer.
#[derive(Clone)]
pub struct CoverageTile {
    /// Tile width in pixels.
    pub width: u32,
    /// Tile height in pixels.
    pub height: u32,
    /// `width * height` coverage bytes (0 = empty, 255 = fully covered).
    pub data: Arc<Vec<u8>>,
}

/// A positioned bitmap — the renderer's libass-`ASS_Image`-style output unit. A
/// frame is a list of these; the caller (or [`composite_bitmap`]) blends them.
///
/// The common case is `Coverage` (A8 + one colour): producing it from a cached
/// tile is an `Arc` clone, so geometry-static animated layers cost almost nothing
/// per frame. Complex effects that mix colours within a layer (blur, swept
/// karaoke, clip) are pre-composited into an `Rgba` tile.
#[derive(Clone)]
pub enum RenderBitmap {
    /// An 8-bit coverage mask plus a single straight RGBA colour.
    Coverage {
        /// Bitmap width in pixels.
        width: u32,
        /// Bitmap height in pixels.
        height: u32,
        /// `width * height` A8 coverage bytes.
        coverage: Arc<Vec<u8>>,
        /// Destination x of the top-left, in frame pixels.
        x: i32,
        /// Destination y of the top-left, in frame pixels.
        y: i32,
        /// Straight (non-premultiplied) RGBA colour applied through the coverage.
        color: [u8; 4],
    },
    /// A pre-composited premultiplied-RGBA tile (`width * height * 4` bytes).
    Rgba {
        /// Bitmap width in pixels.
        width: u32,
        /// Bitmap height in pixels.
        height: u32,
        /// `width * height * 4` premultiplied RGBA bytes.
        pixels: Arc<Vec<u8>>,
        /// Destination x of the top-left, in frame pixels.
        x: i32,
        /// Destination y of the top-left, in frame pixels.
        y: i32,
    },
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
                data: Arc::new(raster.finish()),
            },
            min_x,
            min_y,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{composite, Arc, CoverageTile};

    #[test]
    fn composite_blends_known_pixel() {
        // 1x1 tile, half coverage, opaque red over opaque black.
        let tile = CoverageTile {
            width: 1,
            height: 1,
            data: Arc::new(vec![128]),
        };
        let mut dst = vec![0u8, 0, 0, 255]; // premultiplied black, opaque
        composite(&mut dst, 1, 1, &tile, 0, 0, [255, 0, 0, 255]);
        // src alpha = 255*128/255 = 128; inv = 127.
        // r = 128 + 0; a = 128 + 255*127/255 = 128 + 127 = 255.
        assert_eq!(dst[0], 128, "red");
        assert_eq!(dst[1], 0, "green");
        assert_eq!(dst[2], 0, "blue");
        assert_eq!(dst[3], 255, "alpha");
    }

    #[test]
    fn composite_clips_offscreen() {
        let tile = CoverageTile {
            width: 4,
            height: 4,
            data: Arc::new(vec![255; 16]),
        };
        let mut dst = vec![0u8; 2 * 2 * 4];
        // Place mostly off the top-left; only the bottom-right tile pixel lands.
        composite(&mut dst, 2, 2, &tile, -3, -3, [10, 20, 30, 255]);
        assert_eq!(&dst[0..3], &[10, 20, 30], "the one in-bounds pixel blended");
        assert_eq!(&dst[4..8], &[0, 0, 0, 0], "neighbours untouched");
    }
}
