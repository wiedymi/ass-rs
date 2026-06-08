//! Tests for cubic bezier evaluation.

use super::*;

#[test]
fn bezier_evaluation() {
    let p0 = (0.0, 0.0);
    let p1 = (0.33, 0.0);
    let p2 = (0.67, 1.0);
    let p3 = (1.0, 1.0);

    let start = eval_cubic_bezier(p0, p1, p2, p3, 0.0);
    assert_eq!(start, p0);

    let end = eval_cubic_bezier(p0, p1, p2, p3, 1.0);
    assert_eq!(end, p3);

    let mid = eval_cubic_bezier(p0, p1, p2, p3, 0.5);
    assert!(mid.0 > 0.0 && mid.0 < 1.0);
    assert!(mid.1 > 0.0 && mid.1 < 1.0);
}

#[test]
fn eval_cubic_bezier_edge_cases() {
    // Test identical control points (linear case)
    let linear_result = eval_cubic_bezier((0.0, 0.0), (0.0, 0.0), (1.0, 1.0), (1.0, 1.0), 0.5);
    assert!((linear_result.0 - 0.5).abs() < f32::EPSILON);
    assert!((linear_result.1 - 0.5).abs() < f32::EPSILON);

    // Test extreme t values
    let p0 = (0.0, 0.0);
    let p1 = (0.25, 0.5);
    let p2 = (0.75, 0.5);
    let p3 = (1.0, 1.0);

    // t = 0 should return p0
    let result_0 = eval_cubic_bezier(p0, p1, p2, p3, 0.0);
    assert!((result_0.0 - p0.0).abs() < f32::EPSILON);
    assert!((result_0.1 - p0.1).abs() < f32::EPSILON);

    // t = 1 should return p3
    let result_1 = eval_cubic_bezier(p0, p1, p2, p3, 1.0);
    assert!((result_1.0 - p3.0).abs() < f32::EPSILON);
    assert!((result_1.1 - p3.1).abs() < f32::EPSILON);

    // Test negative coordinates
    let neg_result = eval_cubic_bezier((-1.0, -1.0), (-0.5, -0.5), (0.5, 0.5), (1.0, 1.0), 0.5);
    assert!(neg_result.0 > -1.0 && neg_result.0 < 1.0);
    assert!(neg_result.1 > -1.0 && neg_result.1 < 1.0);

    // Test very small and very large coordinates
    let large_result = eval_cubic_bezier(
        (0.0, 0.0),
        (1000.0, 1000.0),
        (2000.0, 2000.0),
        (3000.0, 3000.0),
        0.5,
    );
    assert!(large_result.0 > 0.0 && large_result.0 < 3000.0);
    assert!(large_result.1 > 0.0 && large_result.1 < 3000.0);
}
