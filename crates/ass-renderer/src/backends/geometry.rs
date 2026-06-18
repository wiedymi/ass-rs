//! Backend-agnostic geometry helpers shared across rendering backends.

use tiny_skia::Transform;

/// Merge positioned glyph outlines into a single path under one transform.
///
/// Lets a layer's glyphs be stroked and filled in one rasterizer pass instead of
/// one per glyph — the per-call setup of `fill_path`/`PathStroker` and the path
/// clones dominate per-frame cost on glyph-dense animated scenes. For
/// non-overlapping glyphs (normal text) the merged Winding fill/stroke is
/// pixel-identical to filling each glyph separately. Returns `None` if no glyph
/// produced geometry.
pub(crate) fn merge_transformed(
    paths: &[tiny_skia::Path],
    transform: Transform,
) -> Option<tiny_skia::Path> {
    let mut builder = tiny_skia::PathBuilder::new();
    for path in paths {
        if let Some(transformed) = path.clone().transform(transform) {
            builder.push_path(&transformed);
        }
    }
    builder.finish()
}

/// Stroke a glyph path to its outline, growing it outward by `wx` horizontally
/// and `wy` vertically (`\xbord`/`\ybord`). tiny-skia strokes uniformly, so an
/// asymmetric border is produced by stroking in a vertically-scaled space and
/// scaling back. The symmetric case (`wx == wy`, almost all content) is a plain
/// uniform stroke. The stroke width is doubled to match libass's outward grow.
#[cfg(not(feature = "nostd"))]
pub(crate) fn stroke_outline(path: &tiny_skia::Path, wx: f32, wy: f32) -> Option<tiny_skia::Path> {
    let mk = |w: f32| tiny_skia::Stroke {
        width: w * 2.0,
        line_cap: tiny_skia::LineCap::Square,
        line_join: tiny_skia::LineJoin::Miter,
        ..Default::default()
    };
    let mut stroker = tiny_skia::PathStroker::new();
    if (wx - wy).abs() < 0.05 || wx <= 0.0 || wy <= 0.0 {
        return stroker.stroke(path, &mk(wx.max(wy)), 1.0);
    }
    // Stroke uniformly with radius wx in a space scaled by (1, wx/wy), then undo
    // the scale: the vertical extent becomes wy while the horizontal stays wx.
    let sy = wx / wy;
    let scaled = path.clone().transform(Transform::from_scale(1.0, sy))?;
    let stroked = stroker.stroke(&scaled, &mk(wx), 1.0)?;
    stroked.transform(Transform::from_scale(1.0, 1.0 / sy))
}

/// Project a screen-space path through a 3D rotation (`\frx`/`\fry`) and a pinhole
/// perspective division about `(cx, cy)`, mirroring libass's transform matrix
/// (RX then RY about a camera at distance `dist`). `\frz` is applied beforehand as
/// a 2D rotation, matching libass's RZ->RX->RY order. Returns `None` if the
/// projected outline is empty.
#[cfg(not(feature = "nostd"))]
pub(crate) fn project_path_3d(
    path: &tiny_skia::Path,
    frx_rad: f32,
    fry_rad: f32,
    cx: f32,
    cy: f32,
    dist: f32,
) -> Option<tiny_skia::Path> {
    use tiny_skia::{PathSegment, Point};
    // libass: sx = -sin(frx), cx = cos(frx); sy = sin(fry), cy = cos(fry).
    let (sfx, cfx) = (-frx_rad.sin(), frx_rad.cos());
    let (sfy, cfy) = (fry_rad.sin(), fry_rad.cos());
    let project = |p: Point| -> Point {
        let dx = p.x - cx;
        let dy = p.y - cy;
        // The glyph starts in the z=0 plane; rotate about X then Y.
        let z3 = dy * sfx; // depth after \frx
        let y3 = dy * cfx; // y after \frx
        let x4 = dx * cfy - z3 * sfy;
        let z4 = dx * sfy + z3 * cfy;
        // Perspective divide by the camera distance (libass adds dist to z).
        let zf = (z4 + dist).max(0.1);
        Point::from_xy(cx + x4 * dist / zf, cy + y3 * dist / zf)
    };
    let mut pb = tiny_skia::PathBuilder::new();
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                let q = project(p);
                pb.move_to(q.x, q.y);
            }
            PathSegment::LineTo(p) => {
                let q = project(p);
                pb.line_to(q.x, q.y);
            }
            PathSegment::QuadTo(a, b) => {
                let (qa, qb) = (project(a), project(b));
                pb.quad_to(qa.x, qa.y, qb.x, qb.y);
            }
            PathSegment::CubicTo(a, b, c) => {
                let (qa, qb, qc) = (project(a), project(b), project(c));
                pb.cubic_to(qa.x, qa.y, qb.x, qb.y, qc.x, qc.y);
            }
            PathSegment::Close => pb.close(),
        }
    }
    pb.finish()
}

/// Screen-space shadow displacement for a local-space `(sx, sy)` offset under the
/// layer transform's linear part — so the fill tile can be reused as the shadow.
#[cfg(not(feature = "nostd"))]
pub(crate) fn shadow_delta(local: Transform, sx: f32, sy: f32) -> (i32, i32) {
    (
        (sx * local.sx + sy * local.kx).round() as i32,
        (sx * local.ky + sy * local.sy).round() as i32,
    )
}
