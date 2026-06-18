//! Software-raster Gaussian blur used by the CPU backend.
//!
//! The sigma/`blur_scale` mapping is backend-agnostic (it mirrors libass's
//! `blur_radius_scale`); only the separable-kernel rasterization here is
//! specific to the software backend.

use tiny_skia::Pixmap;

/// Apply a separable Gaussian blur of standard deviation `sigma` (screen pixels)
/// to a pixmap.
///
/// Callers map their source blur amount to `sigma`: a `\blur` of `b` at a
/// frame/PlayRes ratio `s` becomes `sigma = b * s * 2/sqrt(ln 256)` (libass
/// `blur_radius_scale`, ass_render.c:2539, where `restore_blur` returns the
/// variance), so `\blur4` at a 1:1 render is sigma ~= 3.4px; `\be` edge
/// softening uses a gentler factor. A flat box blur would lower the peak and
/// wash the glyph centre out at larger radii; the Gaussian keeps a bright
/// centre with a soft falloff. Applied to glyph-sized temp pixmaps, so the
/// per-tap cost stays small.
pub(crate) fn apply_gaussian_blur(pixmap: &mut Pixmap, sigma: f32) {
    if sigma <= 0.0 {
        return;
    }
    let radius = (sigma * 3.0).ceil() as i32;
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    if radius < 1 || width == 0 || height == 0 {
        return;
    }

    // Normalised 1D Gaussian kernel.
    let inv_two_sigma_sq = 1.0 / (2.0 * sigma * sigma);
    let mut kernel = vec![0f32; (2 * radius + 1) as usize];
    let mut sum = 0f32;
    for (i, k) in kernel.iter_mut().enumerate() {
        let x = i as i32 - radius;
        let v = (-((x * x) as f32) * inv_two_sigma_sq).exp();
        *k = v;
        sum += v;
    }
    for k in &mut kernel {
        *k /= sum;
    }

    let stride = width as usize * 4;
    let data = pixmap.data_mut();

    // Blur in premultiplied space. Blurring straight-alpha RGBA mixes each colour
    // channel independently of coverage, so a white-on-transparent edge averages
    // toward black as alpha falls — narrowing and dimming the glow (a `\blur20`
    // box kept only ~65% of libass's mass). libass blurs coverage, so premultiply
    // (colour *= alpha) before the passes and un-premultiply after.
    for px in data.chunks_exact_mut(4) {
        let a = u32::from(px[3]);
        px[0] = ((u32::from(px[0]) * a + 127) / 255) as u8;
        px[1] = ((u32::from(px[1]) * a + 127) / 255) as u8;
        px[2] = ((u32::from(px[2]) * a + 127) / 255) as u8;
    }

    let mut temp = vec![0u8; data.len()];

    // Horizontal pass (data -> temp).
    for y in 0..height {
        let row = y as usize * stride;
        for x in 0..width {
            let mut acc = [0f32; 4];
            for (ki, &kw) in kernel.iter().enumerate() {
                let sx = (x + ki as i32 - radius).clamp(0, width - 1) as usize;
                let i = row + sx * 4;
                for (a, &v) in acc.iter_mut().zip(&data[i..i + 4]) {
                    *a += kw * f32::from(v);
                }
            }
            let o = row + x as usize * 4;
            for (dst, &a) in temp[o..o + 4].iter_mut().zip(&acc) {
                *dst = a.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    // Vertical pass (temp -> data).
    for x in 0..width {
        let col = x as usize * 4;
        for y in 0..height {
            let mut acc = [0f32; 4];
            for (ki, &kw) in kernel.iter().enumerate() {
                let sy = (y + ki as i32 - radius).clamp(0, height - 1) as usize;
                let i = sy * stride + col;
                for (a, &v) in acc.iter_mut().zip(&temp[i..i + 4]) {
                    *a += kw * f32::from(v);
                }
            }
            let o = y as usize * stride + col;
            for (dst, &a) in data[o..o + 4].iter_mut().zip(&acc) {
                *dst = a.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    // Un-premultiply (colour /= alpha) back to straight-alpha RGBA; a fully
    // transparent pixel (alpha 0) has no colour to restore.
    for px in data.chunks_exact_mut(4) {
        let a = u32::from(px[3]);
        for c in &mut px[0..3] {
            *c = (u32::from(*c) * 255).checked_div(a).unwrap_or(0).min(255) as u8;
        }
    }
}
