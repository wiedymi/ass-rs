//! Scanline coverage rasterizer: glyph outline → 8-bit coverage bitmap.
//!
//! This is the in-house replacement for tiny-skia's general path filler on the
//! hot text path. It uses the signed-area accumulation technique (as in
//! FreeType's smooth rasterizer and Raph Levien's font-rs): each edge deposits
//! signed area/coverage deltas into an accumulation buffer, and a single per-row
//! prefix sum yields exact-area anti-aliased coverage. Fill uses the non-zero
//! winding rule (`coverage = min(|accumulated|, 1)`), which is what glyph
//! outlines need.
//!
//! Working in coverage (A8) rather than RGBA lets a layer's geometry be
//! rasterized once and cached, then colourized and blended cheaply every frame —
//! the basis for beating libass on animation-heavy content.

mod span;

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};

use span::convert_span;

/// Accumulation-buffer scanline rasterizer producing an A8 coverage bitmap of a
/// fixed `width × height`.
pub struct Rasterizer {
    width: usize,
    height: usize,
    /// Row stride of `acc`, two cells wider than `width` so the `+1` cover writes
    /// at the right edge land in guard cells instead of the next row.
    stride: usize,
    acc: Vec<f32>,
    /// Per-row touched-column span `[row_min, row_max]`. `finish` resolves only
    /// this span and leaves the (zero-initialised) rest untouched, so the prefix
    /// sum runs over the live columns instead of the whole — usually mostly
    /// empty — tile. `row_min[y] > row_max[y]` marks an untouched row.
    row_min: Vec<u32>,
    row_max: Vec<u32>,
}

impl Rasterizer {
    /// Create a rasterizer for a `width × height` coverage bitmap.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let stride = width + 2;
        let rows = height.max(1);
        Self {
            width,
            height,
            stride,
            acc: vec![0.0; stride * rows],
            row_min: vec![width as u32; rows],
            row_max: vec![0; rows],
        }
    }

    /// Add a straight edge between two points in pixel coordinates.
    ///
    /// Edges may be supplied in any order; their signed contributions combine so
    /// that a closed contour yields correct non-zero-winding coverage.
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        // Orient the edge top-to-bottom; `dir` records the original winding.
        let (dir, mut x, top_y, bot_x, bot_y) = if y0 <= y1 {
            (1.0_f32, x0, y0, x1, y1)
        } else {
            (-1.0_f32, x1, y1, x0, y0)
        };
        let dy_total = bot_y - top_y;
        if dy_total <= 0.0 {
            return; // horizontal edge: no vertical coverage
        }
        let dxdy = (bot_x - x) / dy_total;

        // Clip to the top of the bitmap.
        let mut y_top = top_y;
        if y_top < 0.0 {
            x += -y_top * dxdy;
            y_top = 0.0;
        }
        let y_bot = bot_y.min(self.height as f32);
        if y_bot <= y_top {
            return;
        }

        let w = self.width as f32;
        let first_row = y_top.floor() as usize;
        let last_row = (y_bot.ceil() as usize).min(self.height);
        for y in first_row..last_row {
            let line_start = y * self.stride;
            let dy = ((y + 1) as f32).min(y_bot) - (y as f32).max(y_top);
            if dy <= 0.0 {
                continue;
            }
            let x_next = x + dxdy * dy;
            let d = dy * dir;

            // Clamp the x-span into the bitmap; with correct padding this never
            // triggers, but it keeps a degenerate transform from indexing OOB.
            let xc = x.clamp(0.0, w);
            let xnc = x_next.clamp(0.0, w);
            let (xa, xb) = if xc < xnc { (xc, xnc) } else { (xnc, xc) };

            let xa_floor = xa.floor();
            let x0i = xa_floor as usize;
            let x1ceil = xb.ceil();
            let x1i = x1ceil as usize;

            // Record the touched column span for this row. The single-column
            // branch always writes `x0i` and `x0i + 1`; the multi-column branch
            // writes through `x1i`. `finish` resolves only `[row_min, row_max]`.
            let rmin = &mut self.row_min[y];
            *rmin = (*rmin).min(x0i as u32);
            let rmax = &mut self.row_max[y];
            *rmax = (*rmax).max(x1i.max(x0i + 1) as u32);

            if x1i <= x0i + 1 {
                // The edge stays within a single pixel column.
                let xmf = 0.5 * (xc + xnc) - xa_floor;
                let i = line_start + x0i;
                self.acc[i] += d * (1.0 - xmf);
                self.acc[i + 1] += d * xmf;
            } else {
                // The edge spans several columns: split d into the entry-cell
                // area, a constant run, and the exit-cell area (trapezoid areas).
                let s = (xb - xa).recip();
                let x0f = xa - xa_floor;
                let a_m = 1.0 - x0f;
                let am = 0.5 * s * a_m * a_m;
                let x1f = xb - x1ceil + 1.0;
                let bm = 0.5 * s * x1f * x1f;

                let i0 = line_start + x0i;
                self.acc[i0] += d * am;
                if x1i == x0i + 2 {
                    self.acc[i0 + 1] += d * (1.0 - am - bm);
                } else {
                    let a0 = s * (1.5 - x0f);
                    self.acc[i0 + 1] += d * (a0 - am);
                    for xi in (x0i + 2)..(x1i - 1) {
                        self.acc[line_start + xi] += d * s;
                    }
                    let a1 = a0 + ((x1i - x0i) as f32 - 3.0) * s;
                    self.acc[line_start + x1i - 1] += d * (1.0 - a1 - bm);
                }
                self.acc[line_start + x1i] += d * bm;
            }

            x = x_next;
        }
    }

    /// Add a quadratic Bézier, flattened to line segments.
    pub fn quad(&mut self, x0: f32, y0: f32, cx: f32, cy: f32, x1: f32, y1: f32) {
        // Subdivisions from the control-point deviation (~0.2px flatness).
        let dev = ((x0 - 2.0 * cx + x1).powi(2) + (y0 - 2.0 * cy + y1).powi(2)).sqrt();
        let n = (1 + (dev / 0.8).sqrt() as usize).clamp(1, 64);
        let (mut px, mut py) = (x0, y0);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let mt = 1.0 - t;
            let a = mt * mt;
            let b = 2.0 * mt * t;
            let c = t * t;
            let nx = a * x0 + b * cx + c * x1;
            let ny = a * y0 + b * cy + c * y1;
            self.line(px, py, nx, ny);
            px = nx;
            py = ny;
        }
    }

    /// Add a cubic Bézier, flattened to line segments.
    #[allow(clippy::too_many_arguments)]
    pub fn cubic(
        &mut self,
        x0: f32,
        y0: f32,
        c1x: f32,
        c1y: f32,
        c2x: f32,
        c2y: f32,
        x1: f32,
        y1: f32,
    ) {
        let d1 = ((x0 - 2.0 * c1x + c2x).powi(2) + (y0 - 2.0 * c1y + c2y).powi(2)).sqrt();
        let d2 = ((c1x - 2.0 * c2x + x1).powi(2) + (c1y - 2.0 * c2y + y1).powi(2)).sqrt();
        let n = (1 + ((d1 + d2) / 0.8).sqrt() as usize).clamp(1, 96);
        let (mut px, mut py) = (x0, y0);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let mt = 1.0 - t;
            let a = mt * mt * mt;
            let b = 3.0 * mt * mt * t;
            let c = 3.0 * mt * t * t;
            let e = t * t * t;
            let nx = a * x0 + b * c1x + c * c2x + e * x1;
            let ny = a * y0 + b * c1y + c * c2y + e * y1;
            self.line(px, py, nx, ny);
            px = nx;
            py = ny;
        }
    }

    /// Resolve the accumulation buffer into an A8 coverage bitmap
    /// (`width * height` bytes, row-major).
    #[must_use]
    pub fn finish(&self) -> Vec<u8> {
        let mut out = vec![0u8; self.width * self.height];
        if self.width == 0 {
            return out;
        }
        // Scratch for one row's prefix sums, reused across rows.
        let mut psum = vec![0.0_f32; self.width];
        for y in 0..self.height {
            let lo = self.row_min[y] as usize;
            // The right guard column (`width`) is never emitted, mirroring the
            // original `0..width` scan, so clamp the span to the last real pixel.
            let hi = (self.row_max[y] as usize).min(self.width - 1);
            if lo > hi {
                continue; // untouched row: stays transparent
            }
            let row = y * self.stride;
            let out_row = y * self.width;
            let psum_row = &mut psum[lo..=hi];
            let mut sum = 0.0_f32;
            for (p, &a) in psum_row.iter_mut().zip(&self.acc[row + lo..=row + hi]) {
                sum += a;
                *p = sum;
            }
            convert_span(psum_row, &mut out[out_row + lo..=out_row + hi]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::Rasterizer;
    use tiny_skia::{FillRule, Mask, PathBuilder, Transform};

    /// Fill `build` into an A8 mask with our rasterizer.
    fn ours(width: usize, height: usize, build: impl Fn(&mut Rasterizer)) -> Vec<u8> {
        let mut r = Rasterizer::new(width, height);
        build(&mut r);
        r.finish()
    }

    /// Fill the same path into an A8 mask with tiny-skia (the reference).
    fn skia(width: u32, height: u32, path: &tiny_skia::Path) -> Vec<u8> {
        let mut mask = Mask::new(width, height).unwrap();
        mask.fill_path(path, FillRule::Winding, true, Transform::identity());
        mask.data().to_vec()
    }

    /// Mean and max absolute per-pixel difference between two A8 buffers.
    fn diff(a: &[u8], b: &[u8]) -> (f64, u8) {
        let mut sum = 0u64;
        let mut max = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            let d = x.abs_diff(*y);
            sum += u64::from(d);
            max = max.max(d);
        }
        (sum as f64 / a.len() as f64, max)
    }

    #[test]
    fn solid_rect_is_fully_covered() {
        // A rectangle aligned to pixel centres should be ~255 inside.
        let cov = ours(10, 10, |r| {
            r.line(2.0, 2.0, 8.0, 2.0);
            r.line(8.0, 2.0, 8.0, 8.0);
            r.line(8.0, 8.0, 2.0, 8.0);
            r.line(2.0, 8.0, 2.0, 2.0);
        });
        assert_eq!(cov[5 * 10 + 5], 255, "centre of rect must be solid");
        assert_eq!(cov[0], 0, "outside the rect must be empty");
    }

    #[test]
    fn matches_tiny_skia_on_rect() {
        let mut pb = PathBuilder::new();
        pb.move_to(2.3, 2.7);
        pb.line_to(8.6, 2.7);
        pb.line_to(8.6, 8.1);
        pb.line_to(2.3, 8.1);
        pb.close();
        let path = pb.finish().unwrap();
        let s = skia(12, 12, &path);
        let o = ours(12, 12, |r| {
            r.line(2.3, 2.7, 8.6, 2.7);
            r.line(8.6, 2.7, 8.6, 8.1);
            r.line(8.6, 8.1, 2.3, 8.1);
            r.line(2.3, 8.1, 2.3, 2.7);
        });
        // Cross-engine AA conventions differ at edges; bound the discrepancy
        // well below a structural error (a wrong rasterizer means mean >> 10).
        let (mean, max) = diff(&o, &s);
        assert!(mean < 4.0, "mean A8 diff vs tiny-skia too high: {mean}");
        assert!(max < 40, "max A8 diff vs tiny-skia too high: {max}");
    }

    #[test]
    fn matches_tiny_skia_on_triangle() {
        let mut pb = PathBuilder::new();
        pb.move_to(4.0, 1.5);
        pb.line_to(14.2, 12.8);
        pb.line_to(1.7, 13.3);
        pb.close();
        let path = pb.finish().unwrap();
        let s = skia(16, 16, &path);
        let o = ours(16, 16, |r| {
            r.line(4.0, 1.5, 14.2, 12.8);
            r.line(14.2, 12.8, 1.7, 13.3);
            r.line(1.7, 13.3, 4.0, 1.5);
        });
        let (mean, max) = diff(&o, &s);
        assert!(mean < 4.0, "mean A8 diff vs tiny-skia too high: {mean}");
        assert!(max < 40, "max A8 diff vs tiny-skia too high: {max}");
    }

    #[test]
    fn matches_tiny_skia_with_curves() {
        // A rounded blob built from quadratics, in both engines.
        let mut pb = PathBuilder::new();
        pb.move_to(6.0, 2.0);
        pb.quad_to(14.0, 3.0, 13.0, 10.0);
        pb.quad_to(12.0, 17.0, 5.0, 16.0);
        pb.quad_to(2.0, 15.0, 3.0, 8.0);
        pb.quad_to(3.5, 3.0, 6.0, 2.0);
        pb.close();
        let path = pb.finish().unwrap();
        let s = skia(18, 18, &path);
        let o = ours(18, 18, |r| {
            r.quad(6.0, 2.0, 14.0, 3.0, 13.0, 10.0);
            r.quad(13.0, 10.0, 12.0, 17.0, 5.0, 16.0);
            r.quad(5.0, 16.0, 2.0, 15.0, 3.0, 8.0);
            r.quad(3.0, 8.0, 3.5, 3.0, 6.0, 2.0);
        });
        let (mean, max) = diff(&o, &s);
        assert!(mean < 3.0, "mean A8 diff vs tiny-skia too high: {mean}");
        assert!(max < 40, "max A8 diff vs tiny-skia too high: {max}");
    }
}
