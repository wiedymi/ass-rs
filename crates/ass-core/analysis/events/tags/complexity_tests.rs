//! Tests for tag rendering-complexity scoring.

use super::*;

#[test]
fn test_tag_complexity_basic() {
    assert_eq!(calculate_tag_complexity("b"), 1);
    assert_eq!(calculate_tag_complexity("i"), 1);
    assert_eq!(calculate_tag_complexity("c"), 1);
}

#[test]
fn test_tag_complexity_positioning() {
    assert_eq!(calculate_tag_complexity("pos"), 2);
    assert_eq!(calculate_tag_complexity("an"), 2);
    assert_eq!(calculate_tag_complexity("org"), 2);
}

#[test]
fn test_tag_complexity_animation() {
    assert_eq!(calculate_tag_complexity("move"), 3);
    assert_eq!(calculate_tag_complexity("fade"), 3);
    assert_eq!(calculate_tag_complexity("frz"), 3);
}

#[test]
fn test_tag_complexity_advanced() {
    assert_eq!(calculate_tag_complexity("t"), 4);
    assert_eq!(calculate_tag_complexity("pbo"), 4);
}

#[test]
fn test_tag_complexity_drawing() {
    assert_eq!(calculate_tag_complexity("p"), 5);
}

#[test]
fn test_tag_complexity_unknown() {
    assert_eq!(calculate_tag_complexity("unknown"), 2);
    assert_eq!(calculate_tag_complexity(""), 2);
}
