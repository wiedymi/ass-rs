//! Per-row coverage resolution: accumulated prefix sums → 8-bit coverage.
//!
//! Split out from the rasterizer so the SIMD and scalar implementations of the
//! non-zero-winding conversion live together; `Rasterizer::finish` calls the
//! variant selected by the `simd` feature.

/// Convert a row of accumulated prefix sums to 8-bit non-zero-winding coverage:
/// `coverage = min(|sum|, 1) * 255`.
#[cfg(feature = "simd")]
pub(super) fn convert_span(psum: &[f32], out: &mut [u8]) {
    use wide::f32x8;
    let ones = f32x8::splat(1.0);
    let scale = f32x8::splat(255.0);
    let half = f32x8::splat(0.5);

    let mut pc = psum.chunks_exact(8);
    let mut oc = out.chunks_exact_mut(8);
    for (p, o) in pc.by_ref().zip(oc.by_ref()) {
        let mut lane = [0.0_f32; 8];
        lane.copy_from_slice(p);
        let v = f32x8::new(lane);
        let cov = v.abs().min(ones) * scale + half;
        let bytes = cov.to_array();
        for (dst, b) in o.iter_mut().zip(bytes) {
            *dst = b as u8;
        }
    }
    for (dst, &s) in oc.into_remainder().iter_mut().zip(pc.remainder()) {
        *dst = (s.abs().min(1.0) * 255.0 + 0.5) as u8;
    }
}

/// Scalar fallback for [`convert_span`] when the `simd` feature is off.
#[cfg(not(feature = "simd"))]
pub(super) fn convert_span(psum: &[f32], out: &mut [u8]) {
    for (dst, &s) in out.iter_mut().zip(psum) {
        *dst = (s.abs().min(1.0) * 255.0 + 0.5) as u8;
    }
}
