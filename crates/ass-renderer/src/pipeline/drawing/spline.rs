//! B-spline to Bezier curve conversion

#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

/// A bezier curve represented as three control points: start, control, end
type BezierCurve = ((f32, f32), (f32, f32), (f32, f32));

/// Convert B-spline control points to Bezier curves
///
/// Parameters:
/// - points: Control points for the spline
/// - extended: Whether to use extended spline algorithm (for 'p' command)
pub(super) fn spline_to_bezier(points: &[(f32, f32)], extended: bool) -> Vec<BezierCurve> {
    let mut beziers = Vec::new();

    if points.len() < 3 {
        return beziers;
    }

    if extended {
        // Extended B-spline for 'p' command - uses Catmull-Rom interpolation
        for i in 0..points.len() - 1 {
            let p0 = if i > 0 { points[i - 1] } else { points[i] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() {
                points[i + 2]
            } else {
                points[i + 1]
            };

            // Catmull-Rom to Bezier conversion
            let tension = 0.5; // Standard Catmull-Rom tension
            let c1 = (
                p1.0 + tension * (p2.0 - p0.0) / 3.0,
                p1.1 + tension * (p2.1 - p0.1) / 3.0,
            );
            let c2 = (
                p2.0 - tension * (p3.0 - p1.0) / 3.0,
                p2.1 - tension * (p3.1 - p1.1) / 3.0,
            );

            beziers.push((c1, c2, p2));
        }
    } else {
        // Standard B-spline for 's' command
        // Uses uniform cubic B-spline to Bezier conversion
        for i in 0..points.len() - 1 {
            let p0 = if i > 0 { points[i - 1] } else { points[i] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() {
                points[i + 2]
            } else {
                points[i + 1]
            };

            // Calculate control points for cubic bezier from B-spline
            let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
            let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);

            beziers.push((c1, c2, p2));
        }
    }

    beziers
}
