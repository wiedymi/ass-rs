//! Compositing of coverage tiles and positioned bitmaps onto a frame buffer.
//!
//! Source-over blends an A8 coverage tile (with a straight colour) or a
//! [`RenderBitmap`] onto a premultiplied-RGBA8 frame, reusing the low-level
//! blend helpers in [`super::blend`].

use super::blend::{blend_row, mul255};
use super::{CoverageTile, RenderBitmap};

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
