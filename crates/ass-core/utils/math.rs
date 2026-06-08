//! Math helpers for drawing commands and animation curves.
//!
//! Implements bezier evaluation directly without external dependencies.

/// Evaluate cubic bezier curve at parameter t
///
/// Used for drawing command evaluation and animation curves.
/// No external dependencies - implements bezier math directly.
///
/// # Arguments
///
/// * `p0, p1, p2, p3` - Control points as (x, y) tuples
/// * `t` - Parameter from 0.0 to 1.0
///
/// # Returns
///
/// Point on curve as (x, y) tuple
#[must_use]
pub fn eval_cubic_bezier(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    let x = t3.mul_add(
        p3.0,
        (3.0 * mt * t2).mul_add(p2.0, mt3.mul_add(p0.0, 3.0 * mt2 * t * p1.0)),
    );
    let y = t3.mul_add(
        p3.1,
        (3.0 * mt * t2).mul_add(p2.1, mt3.mul_add(p0.1, 3.0 * mt2 * t * p1.1)),
    );

    (x, y)
}
