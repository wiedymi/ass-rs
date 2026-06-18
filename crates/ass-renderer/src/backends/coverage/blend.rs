//! Low-level source-over blend helpers for coverage compositing.
//!
//! Fixed-point `a * b / 255` scaling plus per-row and per-pixel source-over
//! blends, with a `wide`-accelerated row blend behind the `simd` feature and a
//! scalar fallback. Shared by the compositing routines in
//! [`super::composite`].

/// Rounded fixed-point `a * b / 255` for `a, b` in `0..=255`.
///
/// The `((t + (t >> 8)) >> 8)` form is bit-identical to `(a*b + 127) / 255` over
/// that range but maps directly onto SIMD lanes (no per-lane division), so the
/// scalar and SIMD composite paths produce identical pixels.
#[inline]
pub(super) fn mul255(a: u16, b: u16) -> u16 {
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
pub(super) fn blend_row(
    dst: &mut [u8],
    dst_start: usize,
    cov_row: &[u8],
    pr: u16,
    pg: u16,
    pb: u16,
    pa: u16,
) {
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
pub(super) fn blend_row(
    dst: &mut [u8],
    dst_start: usize,
    cov_row: &[u8],
    pr: u16,
    pg: u16,
    pb: u16,
    pa: u16,
) {
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
