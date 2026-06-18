//! Coverage-tile rasterization and the cached / first-miss composite fast paths
//! for the software text layer.

use tiny_skia::Transform;

use crate::backends::geometry::{merge_transformed, shadow_delta, stroke_outline};

/// Rasterize a layer's fill and outline coverage in local space.
#[cfg(not(feature = "nostd"))]
fn rasterize_run_coverage(
    paths: &[tiny_skia::Path],
    local: Transform,
    outline_width: Option<(f32, f32)>,
) -> super::CachedCoverage {
    use crate::backends::coverage::CoverageTile;

    // Merge the per-glyph paths once; both the fill and the outline derive from
    // the same merged outline (previously merged twice, redundantly).
    let merged = merge_transformed(paths, local);
    let fill = merged.as_ref().and_then(CoverageTile::rasterize);
    let outline = outline_width
        .zip(merged.as_ref())
        .and_then(|((wx, wy), merged)| {
            // libass grows the glyph outward by the per-axis border (\xbord/\ybord);
            // stroke_outline produces that (uniform for the symmetric case).
            let outlined = stroke_outline(merged, wx, wy)?;
            CoverageTile::rasterize(&outlined)
        });
    super::CachedCoverage { fill, outline }
}

impl super::super::SoftwareBackend {
    /// Render a text layer from A8 coverage tiles (shadow, outline, fill) and
    /// return `true`, or `false` when the layer uses an effect this fast path
    /// does not cover (blur, edge blur, karaoke, clip, opaque box, underline,
    /// strikethrough) so the caller falls back to the full vector path.
    ///
    /// `base_transform` already bakes rotation/scale/shear, so the rasterized
    /// coverage depends only on the layer geometry — which is what makes the
    /// per-frame work for unchanged geometry a cheap composite rather than a
    /// re-rasterization.
    /// Composite a text layer from cached coverage tiles WITHOUT shaping or
    /// building glyph paths. Returns `true` on a cache hit; `false` if the layer
    /// is ineligible or its geometry is not cached yet (the caller then builds
    /// the paths and calls [`Self::rasterize_coverage_miss`]). This is the path
    /// that makes geometry-static animated layers nearly free: the per-frame work
    /// is just the composite.
    #[cfg(not(feature = "nostd"))]
    pub(in crate::backends::software) fn coverage_hit(
        &mut self,
        data: &crate::pipeline::TextData,
        base_transform: Transform,
        baseline_y: f32,
    ) -> bool {
        let Some((key, outline, shadow, local, fill_color, karaoke_sweep)) =
            super::coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };
        let shadow_paint = shadow.map(|(c, sx, sy)| (c, shadow_delta(local, sx, sy)));
        let anchor_x = data.x.round() as i32;
        let anchor_y = baseline_y.round() as i32;
        let pixmap_w = self.pixmap.width();
        let pixmap_h = self.pixmap.height();
        let dst = self.pixmap.data_mut();
        super::RUN_COVERAGE.with(|cache| {
            let map = cache.borrow();
            let Some(cached) = map.get(&key) else {
                return false;
            };
            super::blur_tile::composite_cached(
                dst,
                pixmap_w,
                pixmap_h,
                cached,
                (anchor_x, anchor_y),
                (outline.map(|(c, _, _)| c), shadow_paint, fill_color),
                karaoke_sweep,
            );
            true
        })
    }

    /// Rasterize a text layer's coverage from already-built `paths`, cache it,
    /// and composite it. Returns `true` if the coverage path handled the layer,
    /// or `false` if it is ineligible and must take the full vector path.
    #[cfg(not(feature = "nostd"))]
    pub(in crate::backends::software) fn rasterize_coverage_miss(
        &mut self,
        data: &crate::pipeline::TextData,
        paths: &[tiny_skia::Path],
        base_transform: Transform,
        baseline_y: f32,
    ) -> bool {
        let Some((key, outline, shadow, local, fill_color, karaoke_sweep)) =
            super::coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };

        super::RUN_COVERAGE.with(|cache| {
            if !cache.borrow().contains_key(&key) {
                let cached =
                    rasterize_run_coverage(paths, local, outline.map(|(_, wx, wy)| (wx, wy)));
                let mut map = cache.borrow_mut();
                // Bound memory: continuously geometry-animated layers produce a
                // fresh key every frame, so drop the cache when it grows large
                // rather than leak; geometry-static layers re-cache once after.
                if map.len() >= 256 {
                    map.clear();
                }
                map.insert(key.clone(), cached);
            }
        });

        let shadow_paint = shadow.map(|(c, sx, sy)| (c, shadow_delta(local, sx, sy)));
        let anchor_x = data.x.round() as i32;
        let anchor_y = baseline_y.round() as i32;
        let pixmap_w = self.pixmap.width();
        let pixmap_h = self.pixmap.height();
        let dst = self.pixmap.data_mut();
        super::RUN_COVERAGE.with(|cache| {
            let map = cache.borrow();
            if let Some(cached) = map.get(&key) {
                super::blur_tile::composite_cached(
                    dst,
                    pixmap_w,
                    pixmap_h,
                    cached,
                    (anchor_x, anchor_y),
                    (outline.map(|(c, _, _)| c), shadow_paint, fill_color),
                    karaoke_sweep,
                );
            }
        });
        true
    }
}
