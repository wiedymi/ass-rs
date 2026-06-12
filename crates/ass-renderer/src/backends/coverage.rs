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

/// Rounded fixed-point `a * b / 255` for `a, b` in `0..=255`.
///
/// The `((t + (t >> 8)) >> 8)` form is bit-identical to `(a*b + 127) / 255` over
/// that range but maps directly onto SIMD lanes (no per-lane division), so the
/// scalar and SIMD composite paths produce identical pixels.
#[inline]
fn mul255(a: u16, b: u16) -> u16 {
    let t = a * b + 128;
    (t + (t >> 8)) >> 8
}

#[cfg(feature = "simd")]
#[inline]
fn mul255x16(a: wide::u16x16, b: wide::u16x16) -> wide::u16x16 {
    let t = a * b + 128_u16;
    (t + (t >> 8)) >> 8
}

/// Source-over blend of one tile row (premultiplied straight `color`, premult
/// channels `pr/pg/pb/pa`) onto `dst` starting at byte `dst_start`. Four pixels
/// at a time with `wide` when the `simd` feature is on; empty four-pixel groups
/// are skipped (text coverage is mostly empty).
#[cfg(feature = "simd")]
#[inline]
fn blend_row(dst: &mut [u8], dst_start: usize, cov_row: &[u8], pr: u16, pg: u16, pb: u16, pa: u16) {
    use wide::u16x16;
    let pcolor = u16x16::new([
        pr, pg, pb, pa, pr, pg, pb, pa, pr, pg, pb, pa, pr, pg, pb, pa,
    ]);
    let run = cov_row.len();
    let mut t = 0;
    while t + 4 <= run {
        let (c0, c1, c2, c3) = (cov_row[t], cov_row[t + 1], cov_row[t + 2], cov_row[t + 3]);
        if (c0 | c1 | c2 | c3) != 0 {
            let cov = u16x16::new([
                u16::from(c0),
                u16::from(c0),
                u16::from(c0),
                u16::from(c0),
                u16::from(c1),
                u16::from(c1),
                u16::from(c1),
                u16::from(c1),
                u16::from(c2),
                u16::from(c2),
                u16::from(c2),
                u16::from(c2),
                u16::from(c3),
                u16::from(c3),
                u16::from(c3),
                u16::from(c3),
            ]);
            let src = mul255x16(pcolor, cov);
            let s = src.to_array();
            let inv = u16x16::new([
                255 - s[3],
                255 - s[3],
                255 - s[3],
                255 - s[3],
                255 - s[7],
                255 - s[7],
                255 - s[7],
                255 - s[7],
                255 - s[11],
                255 - s[11],
                255 - s[11],
                255 - s[11],
                255 - s[15],
                255 - s[15],
                255 - s[15],
                255 - s[15],
            ]);
            let di = dst_start + t * 4;
            let d = &dst[di..di + 16];
            let dpix = u16x16::new([
                u16::from(d[0]),
                u16::from(d[1]),
                u16::from(d[2]),
                u16::from(d[3]),
                u16::from(d[4]),
                u16::from(d[5]),
                u16::from(d[6]),
                u16::from(d[7]),
                u16::from(d[8]),
                u16::from(d[9]),
                u16::from(d[10]),
                u16::from(d[11]),
                u16::from(d[12]),
                u16::from(d[13]),
                u16::from(d[14]),
                u16::from(d[15]),
            ]);
            let out = (src + mul255x16(dpix, inv)).to_array();
            for (slot, &v) in dst[di..di + 16].iter_mut().zip(out.iter()) {
                *slot = v as u8;
            }
        }
        t += 4;
    }
    while t < run {
        blend_pixel(
            dst,
            dst_start + t * 4,
            u16::from(cov_row[t]),
            pr,
            pg,
            pb,
            pa,
        );
        t += 1;
    }
}

/// Scalar fallback when the `simd` feature is off.
#[cfg(not(feature = "simd"))]
#[inline]
fn blend_row(dst: &mut [u8], dst_start: usize, cov_row: &[u8], pr: u16, pg: u16, pb: u16, pa: u16) {
    for (t, &c) in cov_row.iter().enumerate() {
        blend_pixel(dst, dst_start + t * 4, u16::from(c), pr, pg, pb, pa);
    }
}

/// Source-over one premultiplied RGBA pixel by coverage `cov`.
#[inline]
fn blend_pixel(dst: &mut [u8], di: usize, cov: u16, pr: u16, pg: u16, pb: u16, pa: u16) {
    if cov == 0 {
        return;
    }
    let inv = 255 - mul255(pa, cov);
    dst[di] = (mul255(pr, cov) + mul255(u16::from(dst[di]), inv)) as u8;
    dst[di + 1] = (mul255(pg, cov) + mul255(u16::from(dst[di + 1]), inv)) as u8;
    dst[di + 2] = (mul255(pb, cov) + mul255(u16::from(dst[di + 2]), inv)) as u8;
    dst[di + 3] = (mul255(pa, cov) + mul255(u16::from(dst[di + 3]), inv)) as u8;
}

/// Source-over an A8 `cov` (`cov_w * cov_h`) onto a premultiplied-RGBA8 buffer at
/// `(x, y)` using a straight (non-premultiplied) `color`.
///
/// `dst` is `dst_w * dst_h * 4` bytes in tiny-skia's premultiplied RGBA layout.
/// The paint is premultiplied once, then scaled by each pixel's coverage — the
/// same result as filling the path directly with a solid-colour paint.
fn composite_coverage(
    dst: &mut [u8],
    dst_dim: (u32, u32),
    src: (&[u8], u32, u32),
    pos: (i32, i32),
    color: [u8; 4],
) {
    let pa = u16::from(color[3]);
    if pa == 0 {
        return;
    }
    let pr = mul255(u16::from(color[0]), pa);
    let pg = mul255(u16::from(color[1]), pa);
    let pb = mul255(u16::from(color[2]), pa);

    let (dst_w, dst_h) = dst_dim;
    let (cov, cov_w, cov_h) = src;
    let (x, y) = pos;

    // Clip the tile against the destination once so the row blend is bounds-free.
    let (tw, th) = (cov_w as i32, cov_h as i32);
    let (dw, dh) = (dst_w as i32, dst_h as i32);
    let ty0 = (-y).max(0);
    let ty1 = th.min(dh - y);
    let tx0 = (-x).max(0);
    let tx1 = tw.min(dw - x);
    if ty1 <= ty0 || tx1 <= tx0 {
        return;
    }
    let run = (tx1 - tx0) as usize;
    for ty in ty0..ty1 {
        let tile_base = (ty * tw + tx0) as usize;
        let cov_row = &cov[tile_base..tile_base + run];
        let dst_start = ((y + ty) * dw + x + tx0) as usize * 4;
        blend_row(dst, dst_start, cov_row, pr, pg, pb, pa);
    }
}

/// Composite an A8 coverage tile at `(x, y)` in `color` (see
/// [`composite_coverage`]).
pub fn composite(
    dst: &mut [u8],
    dst_w: u32,
    dst_h: u32,
    tile: &CoverageTile,
    x: i32,
    y: i32,
    color: [u8; 4],
) {
    composite_coverage(
        dst,
        (dst_w, dst_h),
        (&tile.data, tile.width, tile.height),
        (x, y),
        color,
    );
}

/// Composite a [`RenderBitmap`] at its own position onto a premultiplied-RGBA8
/// frame buffer with source-over blending.
pub fn composite_bitmap(dst: &mut [u8], dst_w: u32, dst_h: u32, bmp: &RenderBitmap) {
    match bmp {
        RenderBitmap::Coverage {
            width,
            height,
            coverage,
            x,
            y,
            color,
        } => composite_coverage(
            dst,
            (dst_w, dst_h),
            (coverage, *width, *height),
            (*x, *y),
            *color,
        ),
        RenderBitmap::Rgba {
            width,
            height,
            pixels,
            x,
            y,
        } => composite_rgba(dst, (dst_w, dst_h), (pixels, *width, *height), (*x, *y)),
    }
}

/// Source-over a premultiplied-RGBA `src` tile onto a premultiplied-RGBA frame.
fn composite_rgba(dst: &mut [u8], dst_dim: (u32, u32), src: (&[u8], u32, u32), pos: (i32, i32)) {
    let (dst_w, dst_h) = dst_dim;
    let (pixels, sw, sh) = src;
    let (x, y) = pos;
    let (tw, th) = (sw as i32, sh as i32);
    let (dw, dh) = (dst_w as i32, dst_h as i32);
    let ty0 = (-y).max(0);
    let ty1 = th.min(dh - y);
    let tx0 = (-x).max(0);
    let tx1 = tw.min(dw - x);
    if ty1 <= ty0 || tx1 <= tx0 {
        return;
    }
    for ty in ty0..ty1 {
        let mut si = ((ty * tw + tx0) as usize) * 4;
        let mut di = ((y + ty) * dw + x + tx0) as usize * 4;
        for _ in tx0..tx1 {
            let sa = u16::from(pixels[si + 3]);
            if sa != 0 {
                let inv = 255 - sa;
                dst[di] = (u16::from(pixels[si]) + mul255(u16::from(dst[di]), inv)) as u8;
                dst[di + 1] =
                    (u16::from(pixels[si + 1]) + mul255(u16::from(dst[di + 1]), inv)) as u8;
                dst[di + 2] =
                    (u16::from(pixels[si + 2]) + mul255(u16::from(dst[di + 2]), inv)) as u8;
                dst[di + 3] =
                    (u16::from(pixels[si + 3]) + mul255(u16::from(dst[di + 3]), inv)) as u8;
            }
            si += 4;
            di += 4;
        }
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
